pub mod client;
pub mod types;

use std::collections::BTreeSet;
use std::io::Write;

use runtime::{
    AssistantEvent, ContentBlock, ConversationMessage, MessageRole, RuntimeError, TokenUsage,
};
use tools::ToolSpec;

use types::{
    ChatCompletionChunk, ChatCompletionRequest, ChatMessage, FunctionDefinition, OpenAITool,
    OpenAIToolCall, StreamOptions,
};

use crate::error::LlmError;

/// 将运行时消息转换为 `OpenAI` Chat Completion 格式
#[must_use]
pub fn convert_to_openai_messages(messages: &[ConversationMessage]) -> Vec<ChatMessage> {
    let mut result = Vec::new();

    for message in messages {
        let mut text_parts = Vec::new();
        let mut reasoning_parts = Vec::new();
        let mut tool_calls = Vec::new();
        let mut tool_results = Vec::new();

        for block in &message.blocks {
            match block {
                ContentBlock::Text { text } => text_parts.push(text.clone()),
                ContentBlock::Reasoning { text } => reasoning_parts.push(text.clone()),
                ContentBlock::ToolUse { id, name, input } => {
                    tool_calls.push(OpenAIToolCall {
                        id: id.clone(),
                        call_type: "function".to_string(),
                        function: types::FunctionCall {
                            name: name.clone(),
                            arguments: input.clone(),
                        },
                    });
                }
                ContentBlock::ToolResult {
                    tool_use_id,
                    output,
                    ..
                } => {
                    tool_results.push(ChatMessage::tool_result(tool_use_id, output));
                }
            }
        }

        match message.role {
            MessageRole::System => {
                if !text_parts.is_empty() {
                    result.push(ChatMessage::system(text_parts.join("")));
                }
            }
            MessageRole::User => {
                if !tool_results.is_empty() {
                    result.extend(tool_results);
                } else if !text_parts.is_empty() {
                    result.push(ChatMessage::user(text_parts.join("")));
                }
            }
            MessageRole::Assistant => {
                let text = if text_parts.is_empty() {
                    None
                } else {
                    Some(text_parts.join(""))
                };
                let reasoning_content = if reasoning_parts.is_empty() {
                    None
                } else {
                    Some(reasoning_parts.join(""))
                };
                result.push(ChatMessage::assistant_with_tools(
                    text,
                    tool_calls,
                    reasoning_content,
                ));
            }
            MessageRole::Tool => {
                result.extend(tool_results);
            }
        }
    }

    result
}

/// 将系统提示词加入消息列表头部
#[must_use]
pub fn build_openai_request(
    model: &str,
    system_prompt: &[String],
    messages: &[ConversationMessage],
    tools: Option<&[ToolSpec]>,
    max_tokens: u32,
) -> ChatCompletionRequest {
    let mut openai_messages = Vec::new();

    // 系统提示词作为 system message
    if !system_prompt.is_empty() {
        openai_messages.push(ChatMessage::system(system_prompt.join("\n\n")));
    }

    openai_messages.extend(convert_to_openai_messages(messages));

    ChatCompletionRequest {
        model: model.to_string(),
        messages: openai_messages,
        max_tokens: Some(max_tokens),
        stream: true,
        tools: tools.map(|specs| {
            specs
                .iter()
                .map(|spec| OpenAITool {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: spec.name.to_string(),
                        description: Some(spec.description.to_string()),
                        parameters: spec.input_schema.clone(),
                    },
                })
                .collect()
        }),
        tool_choice: tools.map(|_| types::OpenAIToolChoice::Auto),
        // DeepSeek 等兼容端点不接受 OpenAI 的 stream_options，否则会 400
        stream_options: if model.to_ascii_lowercase().contains("deepseek") {
            None
        } else {
            Some(StreamOptions {
                include_usage: true,
            })
        },
    }
}

/// 增量处理 OpenAI SSE chunk，供流式回调与最终汇总复用。
#[derive(Default)]
pub struct OpenAiStreamState {
    pending_tools: BTreeSet<u32>,
    tool_id: std::collections::HashMap<u32, String>,
    tool_name: std::collections::HashMap<u32, String>,
    tool_input: std::collections::HashMap<u32, String>,
}

impl OpenAiStreamState {
    /// 处理单个 chunk，返回本 chunk 产生的事件。
    pub fn push_chunk(
        &mut self,
        chunk: ChatCompletionChunk,
        out: &mut (impl Write + ?Sized),
    ) -> Result<Vec<AssistantEvent>, RuntimeError> {
        let mut events = Vec::new();
        for choice in chunk.choices {
            if let Some(content) = &choice.delta.content {
                if !content.is_empty() {
                    events.push(AssistantEvent::TextDelta(content.clone()));
                }
            }

            if let Some(reasoning_delta) = &choice.delta.reasoning_content {
                if !reasoning_delta.is_empty() {
                    events.push(AssistantEvent::ReasoningDelta(reasoning_delta.clone()));
                }
            }

            if let Some(tool_call_deltas) = &choice.delta.tool_calls {
                for tc in tool_call_deltas {
                    let idx = tc.index;
                    if tc.id.is_some() {
                        self.pending_tools.insert(idx);
                    }
                    if let Some(id) = &tc.id {
                        self.tool_id.insert(idx, id.clone());
                    }
                    if let Some(func) = &tc.function {
                        if let Some(name) = &func.name {
                            self.tool_name.insert(idx, name.clone());
                        }
                        if let Some(args) = &func.arguments {
                            self.tool_input.entry(idx).or_default().push_str(args);
                        }
                    }
                }
            }

            if let Some(finish_reason) = &choice.finish_reason {
                let completed_indices: Vec<u32> = self.pending_tools.iter().copied().collect();
                for idx in completed_indices {
                    if let (Some(id), Some(name)) =
                        (self.tool_id.remove(&idx), self.tool_name.remove(&idx))
                    {
                        let input = self.tool_input.remove(&idx).unwrap_or_default();
                        writeln!(out, "\n  tool: {name}")
                            .and_then(|()| out.flush())
                            .map_err(|e| RuntimeError::new(e.to_string()))?;
                        events.push(AssistantEvent::ToolUse { id, name, input });
                    }
                    self.pending_tools.remove(&idx);
                }

                if finish_reason == "stop" || finish_reason == "tool_calls" {
                    events.push(AssistantEvent::MessageStop);
                }
            }
        }

        if let Some(usage) = chunk.usage {
            events.push(AssistantEvent::Usage(TokenUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                cache_creation_input_tokens: 0,
                cache_read_input_tokens: 0,
            }));
        }

        Ok(events)
    }
}

/// 将 `OpenAI` 流式 chunks 转换为 `AssistantEvent`
///
/// # Errors
///
/// - 如果写入失败,返回运行时错误
pub fn stream_chunks_to_events(
    chunks: Vec<ChatCompletionChunk>,
    out: &mut (impl Write + ?Sized),
) -> Result<Vec<AssistantEvent>, RuntimeError> {
    let mut state = OpenAiStreamState::default();
    let mut events = Vec::new();
    for chunk in chunks {
        events.extend(state.push_chunk(chunk, out)?);
    }
    Ok(events)
}

/// 读取 API Key
///
/// # Errors
///
/// - 如果环境变量未设置,返回 Llm 错误
pub fn read_api_key(env_var: &str) -> Result<String, LlmError> {
    std::env::var(env_var)
        .map_err(|_| LlmError::auth(format!("environment variable {env_var} is not set")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::{ContentBlock, ConversationMessage, MessageRole};

    fn make_user_message(text: &str) -> ConversationMessage {
        ConversationMessage {
            role: MessageRole::User,
            blocks: vec![ContentBlock::Text {
                text: text.to_string(),
            }],
            usage: None,
        }
    }

    fn make_assistant_message(blocks: Vec<ContentBlock>) -> ConversationMessage {
        ConversationMessage {
            role: MessageRole::Assistant,
            blocks,
            usage: None,
        }
    }

    fn make_tool_result(tool_use_id: &str, output: &str) -> ConversationMessage {
        ConversationMessage {
            role: MessageRole::Tool,
            blocks: vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.to_string(),
                tool_name: "test_tool".to_string(),
                output: output.to_string(),
                is_error: false,
            }],
            usage: None,
        }
    }

    #[test]
    fn converts_user_message() {
        let messages = vec![make_user_message("hello")];
        let result = convert_to_openai_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "user");
        assert_eq!(
            result[0].content.as_ref().unwrap().as_str().unwrap(),
            "hello"
        );
    }

    #[test]
    fn converts_assistant_with_tool_use() {
        let messages = vec![make_assistant_message(vec![
            ContentBlock::Text {
                text: "let me check".to_string(),
            },
            ContentBlock::ToolUse {
                id: "call-1".to_string(),
                name: "read_file".to_string(),
                input: r#"{"path":"test.rs"}"#.to_string(),
            },
        ])];
        let result = convert_to_openai_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "assistant");
        let tool_calls = result[0].tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call-1");
        assert_eq!(tool_calls[0].function.name, "read_file");
        assert_eq!(tool_calls[0].function.arguments, r#"{"path":"test.rs"}"#);
    }

    #[test]
    fn converts_tool_result_to_tool_role() {
        let messages = vec![make_tool_result("call-1", "file contents")];
        let result = convert_to_openai_messages(&messages);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, "tool");
        assert_eq!(result[0].tool_call_id.as_deref(), Some("call-1"));
        assert_eq!(
            result[0].content.as_ref().unwrap().as_str().unwrap(),
            "file contents"
        );
    }

    #[test]
    fn builds_request_with_system_prompt() {
        let request = build_openai_request(
            "deepseek-chat",
            &["you are helpful".to_string()],
            &[make_user_message("hi")],
            None,
            4096,
        );
        assert_eq!(request.model, "deepseek-chat");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[1].role, "user");
        assert!(request.stream);
    }

    #[test]
    fn converts_stream_chunks_to_events() {
        use std::io::Cursor;
        let mut buffer = Cursor::new(Vec::new());

        let chunks = vec![
            ChatCompletionChunk {
                choices: vec![types::ChunkChoice {
                    index: 0,
                    delta: types::ChunkDelta {
                        role: Some("assistant".to_string()),
                        content: Some("Hello".to_string()),
                        tool_calls: None,
                        reasoning_content: None,
                    },
                    finish_reason: None,
                }],
                usage: None,
            },
            ChatCompletionChunk {
                choices: vec![types::ChunkChoice {
                    index: 0,
                    delta: types::ChunkDelta {
                        role: None,
                        content: Some(" world".to_string()),
                        tool_calls: None,
                        reasoning_content: None,
                    },
                    finish_reason: Some("stop".to_string()),
                }],
                usage: None,
            },
        ];

        let events = stream_chunks_to_events(chunks, &mut buffer).unwrap();

        assert!(events
            .iter()
            .any(|e| matches!(e, AssistantEvent::TextDelta(t) if t == "Hello")));
        assert!(events
            .iter()
            .any(|e| matches!(e, AssistantEvent::TextDelta(t) if t == " world")));
        assert!(events
            .iter()
            .any(|e| matches!(e, AssistantEvent::MessageStop)));
    }
}
