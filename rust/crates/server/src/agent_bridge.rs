//! HTTP/WebSocket Agent 运行时桥接（独立于 yunxi-cli，避免循环依赖）。

use std::path::PathBuf;
use std::sync::Arc;

use llm::LlmClient;
use memory::{build_context_section, DEFAULT_CONTEXT_LIMIT};
use runtime::{
    AssistantEvent, ConfigLoader, ContentBlock, ConversationRuntime, PermissionMode, PermissionPolicy,
    PermissionPrompter, Session, ToolError, ToolExecutor, TurnSummary,
};
use serde_json::Value;
use tools::{execute_tool, mvp_tool_specs};

use mcp_bridge::McpRuntime;

use crate::settings_store::{load_merged_config, resolve_permission_mode};

const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

pub struct ServerToolExecutor {
    mcp: Arc<McpRuntime>,
}

impl ToolExecutor for ServerToolExecutor {
    fn execute(&mut self, tool_name: &str, input: &str) -> Result<String, ToolError> {
        if McpRuntime::is_mcp_tool(tool_name) {
            return self
                .mcp
                .call_tool(tool_name, input)
                .map_err(ToolError::new);
        }
        let value: Value = serde_json::from_str(input)
            .map_err(|error| ToolError::new(format!("invalid tool input JSON: {error}")))?;
        execute_tool(tool_name, &value).map_err(ToolError::new)
    }
}

fn permission_policy(mode: PermissionMode) -> PermissionPolicy {
    mvp_tool_specs()
        .into_iter()
        .fold(PermissionPolicy::new(mode), |policy, spec| {
            policy.with_tool_requirement(spec.name, spec.required_permission)
        })
}

fn memory_context_section() -> String {
    build_context_section(DEFAULT_CONTEXT_LIMIT).unwrap_or_default()
}

/// 解析工作区根目录（环境变量 `YUNXI_WORKSPACE` 或当前目录）。
#[must_use]
pub fn workspace_root() -> PathBuf {
    std::env::var("YUNXI_WORKSPACE")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn build_system_prompt(root: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let working_date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut sections = runtime::load_system_prompt(
        root.clone(),
        &working_date,
        std::env::consts::OS,
        "unknown",
    )?;
    let memory = memory_context_section();
    if !memory.is_empty() {
        sections.push(memory);
    }
    Ok(sections)
}

pub fn build_runtime(
    session: Session,
    model: &str,
    workspace_root: PathBuf,
) -> Result<ConversationRuntime<LlmClient, ServerToolExecutor>, String> {
    let runtime_config = ConfigLoader::default_for(workspace_root.clone())
        .load()
        .map_err(|e| e.to_string())?;
    let mcp = Arc::new(McpRuntime::try_from_config(&runtime_config));
    let system_prompt = build_system_prompt(&workspace_root).map_err(|e| e.to_string())?;
    let llm = LlmClient::new(model, true, false, None)
        .map_err(|e| e.to_string())?
        .with_extra_tools(mcp.extra_tool_definitions().to_vec());
    let features = runtime_config.feature_config().clone();
    Ok(ConversationRuntime::new_with_features(
        session,
        llm,
        ServerToolExecutor {
            mcp: Arc::clone(&mcp),
        },
        permission_policy(resolve_permission_mode()),
        system_prompt,
        features,
    ))
}

/// 执行一轮 Agent 对话，通过回调推送流式事件。
pub fn run_agent_turn<F>(
    session: &mut Session,
    user_input: &str,
    model: Option<&str>,
    workspace_root: PathBuf,
    mut on_event: F,
    prompter: Option<&mut dyn PermissionPrompter>,
) -> Result<TurnSummary, String>
where
    F: FnMut(AssistantEvent) + Send + 'static,
{
    let model = model.unwrap_or(DEFAULT_MODEL);
    let _ = load_merged_config();
    let mut runtime = build_runtime(session.clone(), model, workspace_root)?;

    let summary = runtime
        .run_turn_with_stream(
            user_input,
            prompter,
            Some(Box::new(move |event| on_event(event))),
        )
        .map_err(|e| e.to_string())?;

    *session = runtime.session().clone();
    Ok(summary)
}

pub fn final_assistant_text(summary: &TurnSummary) -> String {
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
