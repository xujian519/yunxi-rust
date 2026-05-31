use std::collections::{BTreeMap, BTreeSet};

use api::{
    AnthropicClient, ContentBlockDelta, InputContentBlock, InputMessage, MessageRequest,
    MessageResponse, OutputContentBlock, StreamEvent as ApiStreamEvent, ToolChoice, ToolDefinition,
    ToolResultContentBlock,
};
use runtime::{
    load_system_prompt, ApiClient, ApiRequest, AssistantEvent, ContentBlock, ConversationMessage,
    ConversationRuntime, MessageRole, RuntimeError, Session, TokenUsage, ToolError, ToolExecutor,
};
use serde::{Deserialize, Serialize};

use super::execute_tool;
use crate::agent_helpers::{
    agent_store_dir, iso8601_now, make_agent_id, normalize_subagent_type, slugify_agent_name,
    tool_specs_for_allowed_tools,
};

// --- Constants ---

const DEFAULT_AGENT_MODEL: &str = "deepseek-v4-pro";
const DEFAULT_AGENT_SYSTEM_DATE: &str = "2026-03-31";
const DEFAULT_AGENT_MAX_ITERATIONS: usize = 32;

// --- Input/Output types ---

#[derive(Debug, Deserialize)]
pub struct AgentInput {
    pub description: String,
    pub prompt: String,
    pub subagent_type: Option<String>,
    pub name: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    #[serde(rename = "agentId")]
    pub agent_id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "subagentType")]
    pub subagent_type: Option<String>,
    pub model: Option<String>,
    pub status: String,
    #[serde(rename = "outputFile")]
    pub output_file: String,
    #[serde(rename = "manifestFile")]
    pub manifest_file: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "startedAt", skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(rename = "completedAt", skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentJob {
    pub manifest: AgentOutput,
    pub prompt: String,
    pub system_prompt: Vec<String>,
    pub allowed_tools: BTreeSet<String>,
}

// --- Public functions ---

pub(crate) fn execute_agent(input: AgentInput) -> Result<AgentOutput, String> {
    execute_agent_with_spawn(input, spawn_agent_job)
}

/// 使用自定义 spawn 函数执行智能体
///
/// # Errors
///
/// - 如果描述为空,返回错误
/// - 如果提示为空,返回错误
/// - 如果代理存储目录创建失败,返回错误
/// - 如果智能体执行失败,返回错误
pub fn execute_agent_with_spawn<F>(input: AgentInput, spawn_fn: F) -> Result<AgentOutput, String>
where
    F: FnOnce(AgentJob) -> Result<(), String>,
{
    if input.description.trim().is_empty() {
        return Err(String::from("description must not be empty"));
    }
    if input.prompt.trim().is_empty() {
        return Err(String::from("prompt must not be empty"));
    }

    let agent_id = make_agent_id();
    let output_dir = agent_store_dir()?;
    std::fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let output_file = output_dir.join(format!("{agent_id}.md"));
    let manifest_file = output_dir.join(format!("{agent_id}.json"));
    let normalized_subagent_type = normalize_subagent_type(input.subagent_type.as_deref());
    let model = resolve_agent_model(input.model.as_deref(), Some(&normalized_subagent_type));
    let agent_name = input
        .name
        .as_deref()
        .map(slugify_agent_name)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| slugify_agent_name(&input.description));
    let created_at = iso8601_now();
    let system_prompt = build_agent_system_prompt(&normalized_subagent_type)?;
    let allowed_tools = allowed_tools_for_subagent(&normalized_subagent_type);

    let output_contents = format!(
        "# Agent Task

- id: {}
- name: {}
- description: {}
- subagent_type: {}
- created_at: {}

## Prompt

{}
",
        agent_id, agent_name, input.description, normalized_subagent_type, created_at, input.prompt
    );
    std::fs::write(&output_file, output_contents).map_err(|error| error.to_string())?;

    let manifest = AgentOutput {
        agent_id,
        name: agent_name,
        description: input.description,
        subagent_type: Some(normalized_subagent_type),
        model: Some(model),
        status: String::from("running"),
        output_file: output_file.display().to_string(),
        manifest_file: manifest_file.display().to_string(),
        created_at: created_at.clone(),
        started_at: Some(created_at),
        completed_at: None,
        error: None,
    };
    write_agent_manifest(&manifest)?;

    let manifest_for_spawn = manifest.clone();
    let job = AgentJob {
        manifest: manifest_for_spawn,
        prompt: input.prompt,
        system_prompt,
        allowed_tools,
    };
    if let Err(error) = spawn_fn(job) {
        let error = format!("failed to spawn sub-agent: {error}");
        persist_agent_terminal_state(&manifest, "failed", None, Some(error.clone()))?;
        return Err(error);
    }

    Ok(manifest)
}

pub fn allowed_tools_for_subagent(subagent_type: &str) -> BTreeSet<String> {
    // 优先尝试匹配专业代理角色
    if let Some(role) = crate::agent_roles::AgentRole::from_str_opt(subagent_type) {
        return role.allowed_tools();
    }

    let tools = match subagent_type {
        "Explore" => vec![
            "read_file",
            "glob_search",
            "grep_search",
            "WebFetch",
            "WebSearch",
            "ToolSearch",
            "Skill",
            "StructuredOutput",
        ],
        "Plan" => vec![
            "read_file",
            "glob_search",
            "grep_search",
            "WebFetch",
            "WebSearch",
            "ToolSearch",
            "Skill",
            "TodoWrite",
            "StructuredOutput",
            "SendUserMessage",
        ],
        "Verification" => vec![
            "bash",
            "read_file",
            "glob_search",
            "grep_search",
            "WebFetch",
            "WebSearch",
            "ToolSearch",
            "TodoWrite",
            "StructuredOutput",
            "SendUserMessage",
            "PowerShell",
        ],
        "yunxi-guide" => vec![
            "read_file",
            "glob_search",
            "grep_search",
            "WebFetch",
            "WebSearch",
            "ToolSearch",
            "Skill",
            "StructuredOutput",
            "SendUserMessage",
        ],
        "statusline-setup" => vec![
            "bash",
            "read_file",
            "write_file",
            "edit_file",
            "glob_search",
            "grep_search",
            "ToolSearch",
        ],
        _ => vec![
            "bash",
            "read_file",
            "write_file",
            "edit_file",
            "glob_search",
            "grep_search",
            "WebFetch",
            "WebSearch",
            "TodoWrite",
            "Skill",
            "ToolSearch",
            "NotebookEdit",
            "Sleep",
            "SendUserMessage",
            "Config",
            "StructuredOutput",
            "REPL",
            "PowerShell",
        ],
    };
    tools.into_iter().map(str::to_string).collect()
}

#[must_use]
pub fn agent_permission_policy() -> runtime::PermissionPolicy {
    crate::agent_helpers::agent_permission_policy()
}

/// 持久化智能体终端状态
///
/// # Errors
///
/// - 如果文件操作失败,返回错误
pub fn persist_agent_terminal_state(
    manifest: &AgentOutput,
    status: &str,
    result: Option<&str>,
    error: Option<String>,
) -> Result<(), String> {
    append_agent_output(
        &manifest.output_file,
        &format_agent_terminal_output(status, result, error.as_deref()),
    )?;
    let mut next_manifest = manifest.clone();
    next_manifest.status = status.to_string();
    next_manifest.completed_at = Some(iso8601_now());
    next_manifest.error = error;
    write_agent_manifest(&next_manifest)
}

#[must_use]
pub fn final_assistant_text(summary: &runtime::TurnSummary) -> String {
    summary
        .assistant_messages
        .last()
        .map(|message| {
            message
                .blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_default()
}

// --- Internal functions ---

fn spawn_agent_job(job: AgentJob) -> Result<(), String> {
    let thread_name = format!("yunxi-agent-{}", job.manifest.agent_id);
    std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_agent_job(&job)));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    let _ =
                        persist_agent_terminal_state(&job.manifest, "failed", None, Some(error));
                }
                Err(_) => {
                    let _ = persist_agent_terminal_state(
                        &job.manifest,
                        "failed",
                        None,
                        Some(String::from("sub-agent thread panicked")),
                    );
                }
            }
        })
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn run_agent_job(job: &AgentJob) -> Result<(), String> {
    let mut runtime = build_agent_runtime(job)?.with_max_iterations(DEFAULT_AGENT_MAX_ITERATIONS);
    let summary = runtime
        .run_turn(job.prompt.clone(), None)
        .map_err(|error| error.to_string())?;
    let final_text = final_assistant_text(&summary);
    persist_agent_terminal_state(&job.manifest, "completed", Some(final_text.as_str()), None)
}

fn build_agent_runtime(
    job: &AgentJob,
) -> Result<ConversationRuntime<MessagesRuntimeClient, SubagentToolExecutor>, String> {
    let model = job
        .manifest
        .model
        .as_deref()
        .unwrap_or(DEFAULT_AGENT_MODEL)
        .to_string();
    let allowed_tools = job.allowed_tools.clone();
    let api_client = MessagesRuntimeClient::new(model, allowed_tools.clone())?;
    let tool_executor = SubagentToolExecutor::new(allowed_tools);
    Ok(ConversationRuntime::new(
        Session::new(),
        api_client,
        tool_executor,
        agent_permission_policy(),
        job.system_prompt.clone(),
    ))
}

fn build_agent_system_prompt(subagent_type: &str) -> Result<Vec<String>, String> {
    let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
    let mut prompt = load_system_prompt(
        cwd,
        DEFAULT_AGENT_SYSTEM_DATE.to_string(),
        std::env::consts::OS,
        "unknown",
    )
    .map_err(|error| error.to_string())?;

    crate::system_prompt::append_athena_capabilities(&mut prompt);

    if let Some(role) = crate::agent_roles::AgentRole::from_str_opt(subagent_type) {
        let role_prompt = role.system_prompt();
        // 如果 XML 角色定义中包含 <include> 标签，尝试展开
        let cargo_manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let skills_dir = cargo_manifest.join("../../../assets/skills");
        let resolved =
            crate::skill::resolve_includes(&role_prompt, &skills_dir).unwrap_or(role_prompt);
        prompt.push(resolved);
    } else {
        prompt.push(format!(
            "You are a background sub-agent of type `{subagent_type}`. Work only on the delegated task, use only the tools available to you, do not ask the user questions, and finish with a concise result."
        ));
    }
    Ok(prompt)
}

fn resolve_agent_model(model: Option<&str>, subagent_type: Option<&str>) -> String {
    if let Some(m) = model.map(str::trim).filter(|m| !m.is_empty()) {
        return m.to_string();
    }
    if let Some(role) = subagent_type.and_then(crate::agent_roles::AgentRole::from_str_opt) {
        return role.preferred_model().to_string();
    }
    DEFAULT_AGENT_MODEL.to_string()
}

fn write_agent_manifest(manifest: &AgentOutput) -> Result<(), String> {
    std::fs::write(
        &manifest.manifest_file,
        serde_json::to_string_pretty(manifest).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn append_agent_output(path: &str, suffix: &str) -> Result<(), String> {
    use std::io::Write as _;

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(suffix.as_bytes())
        .map_err(|error| error.to_string())
}

fn format_agent_terminal_output(status: &str, result: Option<&str>, error: Option<&str>) -> String {
    let mut sections = vec![format!("\n## Result\n\n- status: {status}\n")];
    if let Some(result) = result.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("\n### Final response\n\n{}\n", result.trim()));
    }
    if let Some(error) = error.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("\n### Error\n\n{}\n", error.trim()));
    }
    sections.join("")
}

// --- Provider detection ---

#[derive(Debug, Clone)]
enum SubagentProvider {
    Anthropic,
    OpenAiCompatible { base_url: String, api_key_env: String },
}

fn detect_provider(model: &str) -> SubagentProvider {
    let lower = model.to_ascii_lowercase();
    if lower.starts_with("claude-")
        || lower.contains("opus")
        || lower.contains("sonnet")
        || lower.contains("haiku")
    {
        return SubagentProvider::Anthropic;
    }
    if lower.contains("deepseek") || lower.starts_with("ds-") {
        return SubagentProvider::OpenAiCompatible {
            base_url: "https://api.deepseek.com".to_string(),
            api_key_env: "DEEPSEEK_API_KEY".to_string(),
        };
    }
    if lower.contains("qwen") {
        return SubagentProvider::OpenAiCompatible {
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key_env: "QWEN_API_KEY".to_string(),
        };
    }
    if lower.contains("kimi") || lower.contains("moonshot") {
        return SubagentProvider::OpenAiCompatible {
            base_url: "https://api.moonshot.cn/v1".to_string(),
            api_key_env: "MOONSHOT_API_KEY".to_string(),
        };
    }
    if lower.contains("glm") || lower.contains("chatglm") {
        return SubagentProvider::OpenAiCompatible {
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            api_key_env: "GLM_API_KEY".to_string(),
        };
    }
    if lower.starts_with("gpt-")
        || lower.starts_with("o1-")
        || lower.starts_with("o3-")
        || lower.starts_with("o4-")
        || lower.contains("chatgpt")
    {
        return SubagentProvider::OpenAiCompatible {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
        };
    }
    SubagentProvider::Anthropic
}

fn resolve_api_key(env_var: &str) -> String {
    if let Ok(val) = std::env::var(env_var) {
        if !val.is_empty() {
            return val;
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        if let Ok(cfg) = runtime::ConfigLoader::default_for(cwd).load() {
            if let Some(env_obj) = cfg.get("env").and_then(|v| v.as_object()) {
                if let Some(val) = env_obj.get(env_var).and_then(|v| v.as_str()) {
                    if !val.is_empty() {
                        return val.to_string();
                    }
                }
            }
        }
    }
    String::new()
}

// --- Runtime client ---

struct MessagesRuntimeClient {
    provider: SubagentProvider,
    model: String,
    allowed_tools: BTreeSet<String>,
}

impl MessagesRuntimeClient {
    fn new(model: String, allowed_tools: BTreeSet<String>) -> Result<Self, String> {
        let provider = detect_provider(&model);

        match &provider {
            SubagentProvider::Anthropic => {
                AnthropicClient::from_env().map_err(|error| {
                    format!("Anthropic API key not configured: {error}")
                })?;
            }
            SubagentProvider::OpenAiCompatible { api_key_env, .. } => {
                let api_key = resolve_api_key(api_key_env);
                if api_key.is_empty() {
                    return Err(format!(
                        "{} is not set; export it before using sub-agents with model '{}'",
                        api_key_env, model
                    ));
                }
            }
        }

        Ok(Self {
            provider,
            model,
            allowed_tools,
        })
    }

    fn block_on<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| handle.block_on(future))
        } else {
            let rt = tokio::runtime::Runtime::new()
                .expect("failed to create tokio runtime — system resource exhaustion");
            rt.block_on(future)
        }
    }

    fn stream_anthropic(
        &self,
        client: AnthropicClient,
        request: &ApiRequest,
        mut on_event: Option<&mut dyn FnMut(AssistantEvent)>,
    ) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let tools = tool_specs_for_allowed_tools(Some(&self.allowed_tools))
            .into_iter()
            .map(|spec| ToolDefinition {
                name: spec.name.to_string(),
                description: Some(spec.description.to_string()),
                input_schema: spec.input_schema,
            })
            .collect::<Vec<_>>();
        let message_request = MessageRequest {
            model: self.model.clone(),
            max_tokens: 32_000,
            messages: convert_messages(&request.messages),
            system: (!request.system_prompt.is_empty()).then(|| request.system_prompt.join("\n\n")),
            tools: (!tools.is_empty()).then_some(tools),
            tool_choice: (!self.allowed_tools.is_empty()).then_some(ToolChoice::Auto),
            stream: true,
        };

        self.block_on(async {
            let mut stream = client
                .stream_message(&message_request)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            let mut events = Vec::new();
            let mut pending_tool: Option<(String, String, String)> = None;
            let mut saw_stop = false;

            while let Some(event) = stream
                .next_event()
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?
            {
                match event {
                    ApiStreamEvent::MessageStart(start) => {
                        for block in start.message.content {
                            push_output_block(block, &mut events, &mut pending_tool, true);
                        }
                    }
                    ApiStreamEvent::ContentBlockStart(start) => {
                        push_output_block(
                            start.content_block,
                            &mut events,
                            &mut pending_tool,
                            true,
                        );
                    }
                    ApiStreamEvent::ContentBlockDelta(delta) => match delta.delta {
                        ContentBlockDelta::TextDelta { text } => {
                            if !text.is_empty() {
                                let event = AssistantEvent::TextDelta(text);
                                if let Some(cb) = on_event.as_deref_mut() {
                                    cb(event.clone());
                                }
                                events.push(event);
                            }
                        }
                        ContentBlockDelta::InputJsonDelta { partial_json } => {
                            if let Some((_, _, input)) = &mut pending_tool {
                                input.push_str(&partial_json);
                            }
                        }
                    },
                    ApiStreamEvent::ContentBlockStop(_) => {
                        if let Some((id, name, input)) = pending_tool.take() {
                            let event = AssistantEvent::ToolUse { id, name, input };
                            if let Some(cb) = on_event.as_deref_mut() {
                                cb(event.clone());
                            }
                            events.push(event);
                        }
                    }
                    ApiStreamEvent::MessageDelta(delta) => {
                        let event = AssistantEvent::Usage(TokenUsage {
                            input_tokens: delta.usage.input_tokens,
                            output_tokens: delta.usage.output_tokens,
                            cache_creation_input_tokens: 0,
                            cache_read_input_tokens: 0,
                        });
                        if let Some(cb) = on_event.as_deref_mut() {
                            cb(event.clone());
                        }
                        events.push(event);
                    }
                    ApiStreamEvent::MessageStop(_) => {
                        saw_stop = true;
                        let event = AssistantEvent::MessageStop;
                        if let Some(cb) = on_event.as_deref_mut() {
                            cb(event.clone());
                        }
                        events.push(event);
                    }
                }
            }

            if !saw_stop
                && events.iter().any(|event| {
                    matches!(event, AssistantEvent::TextDelta(text) if !text.is_empty())
                        || matches!(event, AssistantEvent::ToolUse { .. })
                })
            {
                let event = AssistantEvent::MessageStop;
                if let Some(cb) = on_event.as_deref_mut() {
                    cb(event.clone());
                }
                events.push(event);
            }

            if events
                .iter()
                .any(|event| matches!(event, AssistantEvent::MessageStop))
            {
                return Ok(events);
            }

            let response = client
                .send_message(&MessageRequest {
                    stream: false,
                    ..message_request.clone()
                })
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;
            Ok(response_to_events(response))
        })
    }

    fn stream_openai(
        &self,
        api_key: String,
        base_url: String,
        request: &ApiRequest,
        mut on_event: Option<&mut dyn FnMut(AssistantEvent)>,
    ) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let tools = tool_specs_for_allowed_tools(Some(&self.allowed_tools));

        let messages = build_openai_messages(&request.system_prompt, &request.messages);
        let openai_tools: Option<Vec<serde_json::Value>> = (!tools.is_empty()).then(|| {
            tools
                .iter()
                .map(|spec| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": spec.name,
                            "description": spec.description,
                            "parameters": spec.input_schema,
                        }
                    })
                })
                .collect()
        });

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "max_tokens": 32_000,
            "stream": true,
            "tools": openai_tools,
            "tool_choice": (!self.allowed_tools.is_empty()).then_some("auto"),
        });

        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        self.block_on(async {
            let http = reqwest::Client::new();
            let response = http
                .post(&url)
                .header("Authorization", format!("Bearer {api_key}"))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| RuntimeError::new(e.to_string()))?;

            let text = response
                .text()
                .await
                .map_err(|e| RuntimeError::new(e.to_string()))?;

            let mut events: Vec<AssistantEvent> = Vec::new();
            let mut pending_tools: BTreeMap<u32, (String, String, String)> = BTreeMap::new();

            for line in text.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("data: ") {
                    continue;
                }
                let data = &trimmed[6..];
                if data == "[DONE]" {
                    break;
                }
                let parsed: serde_json::Value =
                    serde_json::from_str(data).map_err(|e| RuntimeError::new(e.to_string()))?;
                process_openai_chunk(
                    &parsed,
                    &mut events,
                    &mut pending_tools,
                    &mut on_event,
                )?;
            }

            let remaining = std::mem::take(&mut pending_tools);
            for (_idx, (id, name, input)) in remaining {
                let event = AssistantEvent::ToolUse { id, name, input };
                if let Some(cb) = on_event.as_deref_mut() {
                    cb(event.clone());
                }
                events.push(event);
            }

            if !events
                .iter()
                .any(|e| matches!(e, AssistantEvent::MessageStop))
            {
                events.push(AssistantEvent::MessageStop);
            }

            Ok(events)
        })
    }
}

impl ApiClient for MessagesRuntimeClient {
    fn stream(
        &mut self,
        request: ApiRequest,
        on_event: Option<&mut dyn FnMut(AssistantEvent)>,
    ) -> Result<Vec<AssistantEvent>, RuntimeError> {
        match &self.provider {
            SubagentProvider::Anthropic => {
                let client = AnthropicClient::from_env()
                    .map_err(|e| RuntimeError::new(e.to_string()))?;
                self.stream_anthropic(client, &request, on_event)
            }
            SubagentProvider::OpenAiCompatible {
                base_url,
                api_key_env,
            } => {
                let api_key = resolve_api_key(api_key_env);
                self.stream_openai(api_key, base_url.clone(), &request, on_event)
            }
        }
    }
}

// --- Subagent tool executor ---

pub struct SubagentToolExecutor {
    allowed_tools: BTreeSet<String>,
}

impl SubagentToolExecutor {
    #[must_use]
    pub fn new(allowed_tools: BTreeSet<String>) -> Self {
        Self { allowed_tools }
    }
}

impl ToolExecutor for SubagentToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError> {
        if !self.allowed_tools.contains(tool_name) {
            return Err(ToolError::new(format!(
                "tool `{tool_name}` is not enabled for this sub-agent"
            )));
        }
        let value = serde_json::from_str(input)
            .map_err(|error| ToolError::new(format!("invalid tool input JSON: {error}")))?;
        execute_tool(tool_name, &value).map_err(ToolError::new)
    }
}

// --- Message conversion helpers ---

fn convert_messages(messages: &[ConversationMessage]) -> Vec<InputMessage> {
    messages
        .iter()
        .filter_map(|message| {
            let role = match message.role {
                MessageRole::System | MessageRole::User | MessageRole::Tool => "user",
                MessageRole::Assistant => "assistant",
            };
            let content = message
                .blocks
                .iter()
                .map(|block| match block {
                    ContentBlock::Text { text } => InputContentBlock::Text { text: text.clone() },
                    ContentBlock::Reasoning { .. } => InputContentBlock::Text {
                        text: String::new(),
                    },
                    ContentBlock::ToolUse { id, name, input } => InputContentBlock::ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: serde_json::from_str(input)
                            .unwrap_or_else(|_| serde_json::json!({ "raw": input })),
                    },
                    ContentBlock::ToolResult {
                        tool_use_id,
                        output,
                        is_error,
                        ..
                    } => InputContentBlock::ToolResult {
                        tool_use_id: tool_use_id.clone(),
                        content: vec![ToolResultContentBlock::Text {
                            text: output.clone(),
                        }],
                        is_error: *is_error,
                    },
                })
                .collect::<Vec<_>>();
            (!content.is_empty()).then(|| InputMessage {
                role: role.to_string(),
                content,
            })
        })
        .collect()
}

fn push_output_block(
    block: OutputContentBlock,
    events: &mut Vec<AssistantEvent>,
    pending_tool: &mut Option<(String, String, String)>,
    streaming_tool_input: bool,
) {
    match block {
        OutputContentBlock::Text { text } => {
            if !text.is_empty() {
                events.push(AssistantEvent::TextDelta(text));
            }
        }
        OutputContentBlock::ToolUse { id, name, input } => {
            let initial_input = if streaming_tool_input
                && input.is_object()
                && input.as_object().is_some_and(serde_json::Map::is_empty)
            {
                String::new()
            } else {
                input.to_string()
            };
            *pending_tool = Some((id, name, initial_input));
        }
    }
}

fn response_to_events(response: MessageResponse) -> Vec<AssistantEvent> {
    let mut events = Vec::new();
    let mut pending_tool = None;

    for block in response.content {
        push_output_block(block, &mut events, &mut pending_tool, false);
        if let Some((id, name, input)) = pending_tool.take() {
            events.push(AssistantEvent::ToolUse { id, name, input });
        }
    }

    events.push(AssistantEvent::Usage(TokenUsage {
        input_tokens: response.usage.input_tokens,
        output_tokens: response.usage.output_tokens,
        cache_creation_input_tokens: response.usage.cache_creation_input_tokens,
        cache_read_input_tokens: response.usage.cache_read_input_tokens,
    }));
    events.push(AssistantEvent::MessageStop);
    events
}

// --- OpenAI streaming helpers ---

fn build_openai_messages(
    system_prompt: &[String],
    messages: &[ConversationMessage],
) -> Vec<serde_json::Value> {
    let mut result = Vec::new();
    if !system_prompt.is_empty() {
        result.push(serde_json::json!({
            "role": "system",
            "content": system_prompt.join("\n\n"),
        }));
    }
    for msg in messages {
        let role = match msg.role {
            MessageRole::System | MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        };
        let mut content_parts: Vec<String> = Vec::new();
        let mut tool_calls = Vec::new();
        let mut tool_call_id: Option<&str> = None;
        for block in &msg.blocks {
            match block {
                ContentBlock::Text { text } => content_parts.push(text.clone()),
                ContentBlock::Reasoning { .. } => {}
                ContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(serde_json::json!({
                        "id": id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": input,
                        }
                    }));
                }
                ContentBlock::ToolResult {
                    tool_use_id,
                    output,
                    ..
                } => {
                    tool_call_id = Some(tool_use_id.as_str());
                    content_parts.push(output.clone());
                }
            }
        }
        let m = match (role, tool_call_id) {
            ("tool", Some(tcid)) => {
                serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tcid,
                    "content": content_parts.join(""),
                })
            }
            ("assistant", _) if !tool_calls.is_empty() => {
                serde_json::json!({
                    "role": "assistant",
                    "content": content_parts.join(""),
                    "tool_calls": tool_calls,
                })
            }
            _ => {
                let content = content_parts.join("");
                serde_json::json!({
                    "role": role,
                    "content": if content.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(content) },
                })
            }
        };
        result.push(m);
    }
    result
}

fn process_openai_chunk(
    chunk: &serde_json::Value,
    events: &mut Vec<AssistantEvent>,
    pending_tools: &mut BTreeMap<u32, (String, String, String)>,
    on_event: &mut Option<&mut dyn FnMut(AssistantEvent)>,
) -> Result<(), RuntimeError> {
    if let Some(choices) = chunk.get("choices").and_then(|v| v.as_array()) {
        for choice in choices {
            let delta = &choice["delta"];
            if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                if !content.is_empty() {
                    let ev = AssistantEvent::TextDelta(content.to_string());
                    if let Some(cb) = on_event.as_deref_mut() {
                        cb(ev.clone());
                    }
                    events.push(ev);
                }
            }
            if let Some(reasoning) = delta.get("reasoning_content").and_then(|v| v.as_str()) {
                if !reasoning.is_empty() {
                    let ev = AssistantEvent::ReasoningDelta(reasoning.to_string());
                    if let Some(cb) = on_event.as_deref_mut() {
                        cb(ev.clone());
                    }
                    events.push(ev);
                }
            }
            if let Some(tcs) = delta.get("tool_calls").and_then(|v| v.as_array()) {
                for tc in tcs {
                    let idx = tc["index"].as_u64().unwrap_or(0) as u32;
                    let id = tc["id"].as_str().unwrap_or_default().to_string();
                    let name = tc["function"]["name"].as_str().unwrap_or_default().to_string();
                    let args = tc["function"]["arguments"].as_str().unwrap_or_default().to_string();
                    if !id.is_empty() || !name.is_empty() {
                        pending_tools.insert(idx, (id, name, args));
                    } else if let Some(entry) = pending_tools.get_mut(&idx) {
                        entry.2.push_str(&args);
                    }
                }
            }
            if let Some(finish) = choice.get("finish_reason").and_then(|v| v.as_str()) {
                let completed: Vec<u32> = pending_tools.keys().copied().collect();
                for idx in completed {
                    if let Some((id, name, input)) = pending_tools.remove(&idx) {
                        if !id.is_empty() || !name.is_empty() {
                            let ev = AssistantEvent::ToolUse { id, name, input };
                            if let Some(cb) = on_event.as_deref_mut() {
                                cb(ev.clone());
                            }
                            events.push(ev);
                        }
                    }
                }
                if finish == "stop" || finish == "tool_calls" {
                    events.push(AssistantEvent::MessageStop);
                }
            }
        }
    }
    if let Some(usage) = chunk.get("usage") {
        let ev = AssistantEvent::Usage(TokenUsage {
            input_tokens: usage["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: usage["completion_tokens"].as_u64().unwrap_or(0) as u32,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        });
        if let Some(cb) = on_event.as_deref_mut() {
            cb(ev.clone());
        }
        events.push(ev);
    }
    Ok(())
}
