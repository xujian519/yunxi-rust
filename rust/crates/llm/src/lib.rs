pub mod config;
pub mod error;
pub mod openai;
pub mod provider;

use std::collections::BTreeSet;
use std::io::{self, Write};

use config::LlmConfig;
use openai::client::OpenAiClient;
use provider::Provider;

use api::{
    read_base_url, AnthropicClient, AuthSource, ContentBlockDelta, InputContentBlock, InputMessage,
    MessageRequest, OutputContentBlock, StreamEvent as ApiStreamEvent, ToolChoice, ToolDefinition,
    ToolResultContentBlock,
};
use runtime::{
    ApiClient, ApiRequest, AssistantEvent, ContentBlock, ConversationMessage, MessageRole,
    RuntimeError, TokenUsage,
};

type AllowedToolSet = BTreeSet<String>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum LlmClientInner {
    Anthropic,
    OpenAi { base_url: String, api_key: String },
}

pub struct LlmClient {
    inner: LlmClientInner,
    model: String,
    enable_tools: bool,
    emit_output: bool,
    allowed_tools: Option<AllowedToolSet>,
    runtime: tokio::runtime::Runtime,
}

impl LlmClient {
    /// 创建新的 LLM 客户端
    ///
    /// # Errors
    ///
    /// - 如果运行时创建失败,返回错误
    pub fn new(
        model: &str,
        enable_tools: bool,
        emit_output: bool,
        allowed_tools: Option<AllowedToolSet>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = LlmConfig::load().unwrap_or_default();

        // 先解析别名
        let resolved_model = config.resolve_alias(model);

        // 检测 provider
        let provider = Provider::detect(&resolved_model, &config.providers);

        let inner = match &provider {
            Provider::Anthropic => LlmClientInner::Anthropic,
            Provider::OpenAICompatible {
                base_url,
                api_key_env,
            } => {
                let api_key = std::env::var(api_key_env).unwrap_or_else(|_| String::new());
                LlmClientInner::OpenAi {
                    base_url: base_url.clone(),
                    api_key,
                }
            }
        };

        Ok(Self {
            inner,
            model: resolved_model,
            enable_tools,
            emit_output,
            allowed_tools,
            runtime: tokio::runtime::Runtime::new()?,
        })
    }

    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }
}

fn max_tokens_for_model(model: &str) -> u32 {
    let lower = model.to_ascii_lowercase();
    if lower.contains("opus") {
        32_000
    } else if lower.contains("deepseek") || lower.contains("qwen") {
        8_192
    } else {
        64_000
    }
}

impl ApiClient for LlmClient {
    #[allow(clippy::too_many_lines, clippy::needless_pass_by_value)]
    fn stream(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        match &self.inner {
            LlmClientInner::Anthropic => self.stream_anthropic(request),
            LlmClientInner::OpenAi { .. } => self.stream_openai(request),
        }
    }
}

impl LlmClient {
    #[allow(clippy::needless_pass_by_value)]
    fn stream_anthropic(
        &mut self,
        request: ApiRequest,
    ) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let anthropic_client = Self::build_anthropic_client()?;
        let message_request = self.build_anthropic_request(&request);

        self.runtime.block_on(async {
            let mut stream = anthropic_client
                .stream_message(&message_request)
                .await
                .map_err(|error| RuntimeError::new(error.to_string()))?;

            let mut stdout = io::stdout();
            let mut sink = io::sink();
            let out: &mut dyn Write = if self.emit_output {
                &mut stdout
            } else {
                &mut sink
            };

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
                            push_anthropic_output_block(
                                block,
                                out,
                                &mut events,
                                &mut pending_tool,
                            )?;
                        }
                    }
                    ApiStreamEvent::ContentBlockStart(start) => {
                        push_anthropic_output_block(
                            start.content_block,
                            out,
                            &mut events,
                            &mut pending_tool,
                        )?;
                    }
                    ApiStreamEvent::ContentBlockDelta(delta) => match delta.delta {
                        ContentBlockDelta::TextDelta { text } => {
                            if !text.is_empty() {
                                write!(out, "{text}")
                                    .and_then(|()| out.flush())
                                    .map_err(|error| RuntimeError::new(error.to_string()))?;
                                events.push(AssistantEvent::TextDelta(text));
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
                            writeln!(out, "\n  tool: {name}")
                                .and_then(|()| out.flush())
                                .map_err(|error| RuntimeError::new(error.to_string()))?;
                            events.push(AssistantEvent::ToolUse { id, name, input });
                        }
                    }
                    ApiStreamEvent::MessageDelta(delta) => {
                        events.push(AssistantEvent::Usage(TokenUsage {
                            input_tokens: delta.usage.input_tokens,
                            output_tokens: delta.usage.output_tokens,
                            cache_creation_input_tokens: 0,
                            cache_read_input_tokens: 0,
                        }));
                    }
                    ApiStreamEvent::MessageStop(_) => {
                        saw_stop = true;
                        events.push(AssistantEvent::MessageStop);
                    }
                }
            }

            if !saw_stop
                && events.iter().any(|event| {
                    matches!(event, AssistantEvent::TextDelta(text) if !text.is_empty())
                        || matches!(event, AssistantEvent::ToolUse { .. })
                })
            {
                events.push(AssistantEvent::MessageStop);
            }

            Ok(events)
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    fn stream_openai(&mut self, request: ApiRequest) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let (base_url, api_key) = match &self.inner {
            LlmClientInner::OpenAi { base_url, api_key } => (base_url.clone(), api_key.clone()),
            LlmClientInner::Anthropic => return Err(RuntimeError::new("expected OpenAI provider")),
        };

        let tools = if self.enable_tools {
            Some(
                tools::mvp_tool_specs()
                    .into_iter()
                    .filter(|spec| {
                        self.allowed_tools
                            .as_ref()
                            .is_none_or(|allowed| allowed.contains(spec.name))
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };

        let openai_request = openai::build_openai_request(
            &self.model,
            &request.system_prompt,
            &request.messages,
            tools.as_deref(),
            max_tokens_for_model(&self.model),
        );

        let client = OpenAiClient::new(base_url, api_key);

        self.runtime.block_on(async {
            let mut stream = client
                .stream_chat(&openai_request)
                .await
                .map_err(|e| RuntimeError::new(e.to_string()))?;

            let mut chunks = Vec::new();
            while let Some(chunk) = stream
                .next_chunk()
                .await
                .map_err(|e| RuntimeError::new(e.to_string()))?
            {
                chunks.push(chunk);
            }

            let mut stdout = io::stdout();
            let mut sink = io::sink();
            let out: &mut dyn Write = if self.emit_output {
                &mut stdout
            } else {
                &mut sink
            };

            openai::stream_chunks_to_events(chunks, out)
        })
    }

    fn build_anthropic_client() -> Result<AnthropicClient, RuntimeError> {
        let auth = resolve_auth_source()?;
        Ok(AnthropicClient::from_auth(auth).with_base_url(read_base_url()))
    }

    fn build_anthropic_request(&self, request: &ApiRequest) -> MessageRequest {
        MessageRequest {
            model: self.model.clone(),
            max_tokens: max_tokens_for_model(&self.model),
            messages: convert_messages_anthropic(&request.messages),
            system: (!request.system_prompt.is_empty()).then(|| request.system_prompt.join("\n\n")),
            tools: self.enable_tools.then(|| {
                tools::mvp_tool_specs()
                    .into_iter()
                    .filter(|spec| {
                        self.allowed_tools
                            .as_ref()
                            .is_none_or(|allowed| allowed.contains(spec.name))
                    })
                    .map(|spec| ToolDefinition {
                        name: spec.name.to_string(),
                        description: Some(spec.description.to_string()),
                        input_schema: spec.input_schema,
                    })
                    .collect()
            }),
            tool_choice: self.enable_tools.then_some(ToolChoice::Auto),
            stream: true,
        }
    }
}

fn resolve_auth_source() -> Result<AuthSource, RuntimeError> {
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        if !api_key.is_empty() {
            if let Ok(token) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
                if !token.is_empty() {
                    return Ok(AuthSource::ApiKeyAndBearer {
                        api_key,
                        bearer_token: token,
                    });
                }
            }
            return Ok(AuthSource::ApiKey(api_key));
        }
    }
    if let Ok(token) = std::env::var("ANTHROPIC_AUTH_TOKEN") {
        if !token.is_empty() {
            return Ok(AuthSource::BearerToken(token));
        }
    }
    match api::resolve_startup_auth_source(|| {
        let cwd = std::env::current_dir().map_err(|e| api::ApiError::Auth(e.to_string()))?;
        let config = runtime::ConfigLoader::default_for(&cwd)
            .load()
            .map_err(|e| api::ApiError::Auth(e.to_string()))?;
        Ok(config.oauth().cloned())
    }) {
        Ok(auth) => Ok(auth),
        Err(_) => Err(RuntimeError::new(
            "no Anthropic API key or OAuth token found",
        )),
    }
}

fn convert_messages_anthropic(messages: &[ConversationMessage]) -> Vec<InputMessage> {
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

fn push_anthropic_output_block(
    block: OutputContentBlock,
    out: &mut (impl Write + ?Sized),
    events: &mut Vec<AssistantEvent>,
    pending_tool: &mut Option<(String, String, String)>,
) -> Result<(), RuntimeError> {
    match block {
        OutputContentBlock::Text { text } => {
            if !text.is_empty() {
                write!(out, "{text}")
                    .and_then(|()| out.flush())
                    .map_err(|error| RuntimeError::new(error.to_string()))?;
                events.push(AssistantEvent::TextDelta(text));
            }
        }
        OutputContentBlock::ToolUse { id, name, input } => {
            let initial_input =
                if input.is_object() && input.as_object().is_some_and(serde_json::Map::is_empty) {
                    String::new()
                } else {
                    input.to_string()
                };
            *pending_tool = Some((id, name, initial_input));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_client_construction_anthropic() {
        std::env::set_var("ANTHROPIC_API_KEY", "test_key");

        tokio::task::spawn_blocking(|| {
            let client = LlmClient::new("claude-opus-4-6", true, false, None)
                .expect("should construct Anthropic LLM client");

            assert_eq!(client.model(), "claude-opus-4-6");

            std::env::remove_var("ANTHROPIC_API_KEY");
        })
        .await
        .expect("spawn_blocking task should complete");
    }
}
