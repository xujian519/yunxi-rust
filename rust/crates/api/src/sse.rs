use crate::error::ApiError;
use crate::types::StreamEvent;

/// SSE（Server-Sent Events）流式解析器
///
/// 用于解析 Anthropic API 返回的 SSE 格式流数据，支持跨数据块的帧解析。
#[derive(Debug, Default)]
pub struct SseParser {
    buffer: Vec<u8>,
}

impl SseParser {
    /// 创建新的 `SseParser` 实例
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use api::sse::SseParser;
    ///
    /// let parser = SseParser::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 推入一个数据块并解析所有完整的帧
    ///
    /// 将新的数据块添加到内部缓冲区，并解析出所有完整的 SSE 帧。
    /// 跨越多个数据块的帧会被正确处理。
    ///
    /// # 参数
    ///
    /// * `chunk` - 从网络流接收的数据块
    ///
    /// # 返回
    ///
    /// 返回解析出的 `StreamEvent` 列表
    ///
    /// # Errors
    ///
    /// 如果帧解析失败或数据格式不正确，返回 `ApiError`
    pub fn push(&mut self, chunk: &[u8]) -> Result<Vec<StreamEvent>, ApiError> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();

        while let Some(frame) = self.next_frame() {
            if let Some(event) = parse_frame(&frame)? {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// 完成流解析并处理剩余的未完成帧
    ///
    /// 当流结束时调用此方法，处理缓冲区中剩余的数据。
    /// 如果缓冲区为空，返回空列表。
    ///
    /// # 返回
    ///
    /// 返回从剩余数据解析出的 `StreamEvent` 列表
    ///
    /// # Errors
    ///
    /// 如果剩余数据无法解析为有效帧，返回 `ApiError`
    pub fn finish(&mut self) -> Result<Vec<StreamEvent>, ApiError> {
        if self.buffer.is_empty() {
            return Ok(Vec::new());
        }

        let trailing = std::mem::take(&mut self.buffer);
        match parse_frame(&String::from_utf8_lossy(&trailing))? {
            Some(event) => Ok(vec![event]),
            None => Ok(Vec::new()),
        }
    }

    fn next_frame(&mut self) -> Option<String> {
        let separator = self
            .buffer
            .windows(2)
            .position(|window| window == b"\n\n")
            .map(|position| (position, 2))
            .or_else(|| {
                self.buffer
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|position| (position, 4))
            })?;

        let (position, separator_len) = separator;
        let frame = self
            .buffer
            .drain(..position + separator_len)
            .collect::<Vec<_>>();
        let frame_len = frame.len().saturating_sub(separator_len);
        Some(String::from_utf8_lossy(&frame[..frame_len]).into_owned())
    }
}

/// 解析单个 SSE 帧
///
/// 解析一个完整的 SSE 帧字符串，提取事件类型和数据负载。
/// 会忽略注释行（以 `:` 开头）和 ping 事件。
///
/// # 参数
///
/// * `frame` - SSE 帧字符串
///
/// # 返回
///
/// 返回解析出的 `StreamEvent`，如果帧为空或应被忽略则返回 `None`
///
/// # Errors
///
/// 如果 JSON 解析失败，返回 `ApiError`
pub fn parse_frame(frame: &str) -> Result<Option<StreamEvent>, ApiError> {
    let trimmed = frame.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let mut data_lines = Vec::new();
    let mut event_name: Option<&str> = None;

    for line in trimmed.lines() {
        if line.starts_with(':') {
            continue;
        }
        if let Some(name) = line.strip_prefix("event:") {
            event_name = Some(name.trim());
            continue;
        }
        if let Some(data) = line.strip_prefix("data:") {
            data_lines.push(data.trim_start());
        }
    }

    if matches!(event_name, Some("ping")) {
        return Ok(None);
    }

    if data_lines.is_empty() {
        return Ok(None);
    }

    let payload = data_lines.join("\n");
    if payload == "[DONE]" {
        return Ok(None);
    }

    serde_json::from_str::<StreamEvent>(&payload)
        .map(Some)
        .map_err(ApiError::from)
}

#[cfg(test)]
mod tests {
    use super::{parse_frame, SseParser};
    use crate::types::{ContentBlockDelta, MessageDelta, OutputContentBlock, StreamEvent, Usage};

    #[test]
    fn parses_single_frame() {
        let frame = concat!(
            "event: content_block_start\n",
            "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"Hi\"}}\n\n"
        );

        let event = parse_frame(frame).expect("frame should parse");
        assert_eq!(
            event,
            Some(StreamEvent::ContentBlockStart(
                crate::types::ContentBlockStartEvent {
                    index: 0,
                    content_block: OutputContentBlock::Text {
                        text: "Hi".to_string(),
                    },
                },
            ))
        );
    }

    #[test]
    fn parses_chunked_stream() {
        let mut parser = SseParser::new();
        let first = b"event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hel";
        let second = b"lo\"}}\n\n";

        assert!(parser
            .push(first)
            .expect("first chunk should buffer")
            .is_empty());
        let events = parser.push(second).expect("second chunk should parse");

        assert_eq!(
            events,
            vec![StreamEvent::ContentBlockDelta(
                crate::types::ContentBlockDeltaEvent {
                    index: 0,
                    delta: ContentBlockDelta::TextDelta {
                        text: "Hello".to_string(),
                    },
                }
            )]
        );
    }

    #[test]
    fn ignores_ping_and_done() {
        let mut parser = SseParser::new();
        let payload = concat!(
            ": keepalive\n",
            "event: ping\n",
            "data: {\"type\":\"ping\"}\n\n",
            "event: message_delta\n",
            "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":1,\"output_tokens\":2}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n",
            "data: [DONE]\n\n"
        );

        let events = parser
            .push(payload.as_bytes())
            .expect("parser should succeed");
        assert_eq!(
            events,
            vec![
                StreamEvent::MessageDelta(crate::types::MessageDeltaEvent {
                    delta: MessageDelta {
                        stop_reason: Some("tool_use".to_string()),
                        stop_sequence: None,
                    },
                    usage: Usage {
                        input_tokens: 1,
                        cache_creation_input_tokens: 0,
                        cache_read_input_tokens: 0,
                        output_tokens: 2,
                    },
                }),
                StreamEvent::MessageStop(crate::types::MessageStopEvent {}),
            ]
        );
    }

    #[test]
    fn ignores_data_less_event_frames() {
        let frame = "event: ping\n\n";
        let event = parse_frame(frame).expect("frame without data should be ignored");
        assert_eq!(event, None);
    }

    #[test]
    fn parses_split_json_across_data_lines() {
        let frame = concat!(
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,\n",
            "data: \"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n"
        );

        let event = parse_frame(frame).expect("frame should parse");
        assert_eq!(
            event,
            Some(StreamEvent::ContentBlockDelta(
                crate::types::ContentBlockDeltaEvent {
                    index: 0,
                    delta: ContentBlockDelta::TextDelta {
                        text: "Hello".to_string(),
                    },
                }
            ))
        );
    }
}
