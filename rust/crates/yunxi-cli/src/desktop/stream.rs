//! AssistantEvent → 前端 StreamEvent 转换与 Tauri emit。

use runtime::{AssistantEvent, TokenUsage, TurnSummary};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// 与前端 `StreamEvent` 对齐的流式事件载荷。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    TextDelta {
        content: String,
    },
    ReasoningDelta {
        content: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: String,
    },
    ToolResult {
        id: String,
        output: String,
        is_error: bool,
    },
    PermissionRequest {
        request_id: String,
        tool: String,
        input: String,
    },
    Usage {
        input_tokens: u32,
        output_tokens: u32,
    },
    MessageStop,
    Error {
        message: String,
    },
}

impl From<AssistantEvent> for StreamEvent {
    fn from(event: AssistantEvent) -> Self {
        match event {
            AssistantEvent::TextDelta(content) => Self::TextDelta { content },
            AssistantEvent::ReasoningDelta(content) => Self::ReasoningDelta { content },
            AssistantEvent::ToolUse { id, name, input } => Self::ToolUse { id, name, input },
            AssistantEvent::Usage(usage) => Self::Usage {
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
            },
            AssistantEvent::MessageStop => Self::MessageStop,
        }
    }
}

pub fn stream_channel(session_id: &str) -> String {
    format!("yunxi://stream/{session_id}")
}

pub fn emit_stream(app: &AppHandle, session_id: &str, event: StreamEvent) {
    let channel = stream_channel(session_id);
    let _ = app.emit(&channel, event);
}

pub fn emit_assistant_event(app: &AppHandle, session_id: &str, event: AssistantEvent) {
    emit_stream(app, session_id, event.into());
}

pub fn emit_tool_results_from_summary(app: &AppHandle, session_id: &str, summary: &TurnSummary) {
    use runtime::ContentBlock;
    for message in &summary.tool_results {
        for block in &message.blocks {
            if let ContentBlock::ToolResult {
                tool_use_id,
                output,
                is_error,
                ..
            } = block
            {
                emit_stream(
                    app,
                    session_id,
                    StreamEvent::ToolResult {
                        id: tool_use_id.clone(),
                        output: output.clone(),
                        is_error: *is_error,
                    },
                );
            }
        }
    }
}

pub fn emit_usage(app: &AppHandle, session_id: &str, usage: TokenUsage) {
    emit_stream(
        app,
        session_id,
        StreamEvent::Usage {
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
        },
    );
}
