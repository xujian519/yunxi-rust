//! WebSocket 流式事件（与桌面端 `StreamEvent` JSON 格式对齐）。

use runtime::{AssistantEvent, ContentBlock, TurnSummary};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    TextDelta { content: String },
    ReasoningDelta { content: String },
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
    Usage {
        input_tokens: u32,
        output_tokens: u32,
    },
    MessageStop,
    Error { message: String },
    PermissionRequest {
        request_id: String,
        tool: String,
        input: String,
    },
    /// 非流式：完整 assistant 文本（HTTP server 回退）
    AssistantMessage { content: String },
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

pub fn tool_results_from_summary(summary: &TurnSummary) -> Vec<StreamEvent> {
    let mut out = Vec::new();
    for message in &summary.tool_results {
        for block in &message.blocks {
            if let ContentBlock::ToolResult {
                tool_use_id,
                output,
                is_error,
                ..
            } = block
            {
                out.push(StreamEvent::ToolResult {
                    id: tool_use_id.clone(),
                    output: output.clone(),
                    is_error: *is_error,
                });
            }
        }
    }
    out
}

pub fn stream_event_json(event: &StreamEvent) -> String {
    serde_json::to_string(event).unwrap_or_else(|_| {
        serde_json::json!({"type":"error","message":"serialize stream event failed"}).to_string()
    })
}
