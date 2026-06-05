use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::components::chat_view::{ChatRole, ChatView};
use crate::tui::ui_palette;
use crate::tui::widgets::message_bubble::MessageBubble;

/// 计算消息在指定宽度下的渲染高度（行数）。
///
/// 估算逻辑：
/// - 标签行：1 行
/// - 思考过程：reasoning 文本按行数计算
/// - 内容文本：按换行符分割后，每行根据宽度折行
/// - 内边距：上下各 1 行
/// - 分隔线：1 行（如果有 reasoning）
fn estimate_entry_height(entry: &crate::tui::components::chat_view::ChatEntry, width: u16) -> u16 {
    if width == 0 {
        return 1;
    }

    let bubble_width = if entry.role == ChatRole::User {
        ((width as f32) * 0.85) as u16
    } else {
        ((width as f32) * 0.90) as u16
    }
    .min(width)
    .max(10);

    let inner_width = bubble_width.saturating_sub(4).max(1);
    let mut height = 2u16;

    if let Some(reasoning) = &entry.reasoning {
        if !reasoning.is_empty() {
            height += 2;
            for line in reasoning.lines() {
                height += estimate_wrapped_lines(line, inner_width);
            }
        }
    }

    for line in entry.text.lines() {
        height += estimate_wrapped_lines(line, inner_width);
    }

    height + 1
}

/// 估算单行文本在指定宽度下折行后的行数。
fn estimate_wrapped_lines(line: &str, width: u16) -> u16 {
    if line.is_empty() {
        return 1;
    }
    let width = width as usize;
    let mut count = 0usize;
    let mut current_width = 0usize;

    for ch in line.chars() {
        let char_width = if ch.is_ascii() { 1 } else { 2 };
        if current_width + char_width > width && current_width > 0 {
            count += 1;
            current_width = char_width;
        } else {
            current_width += char_width;
        }
    }

    if current_width > 0 {
        count += 1;
    }

    count as u16
}

/// 计算所有消息的累积高度，返回每个消息的起始 Y 偏移。
fn compute_cumulative_heights(
    entries: &[crate::tui::components::chat_view::ChatEntry],
    width: u16,
) -> Vec<u16> {
    let mut offsets = Vec::with_capacity(entries.len());
    let mut current_y = 0u16;

    for entry in entries {
        offsets.push(current_y);
        current_y += estimate_entry_height(entry, width);
    }

    offsets
}

/// 找到第一个可见消息的索引。
fn find_first_visible(scroll_offset: usize, cumulative_heights: &[u16]) -> usize {
    let scroll = scroll_offset as u16;
    cumulative_heights
        .iter()
        .enumerate()
        .find(|&(_, &offset)| offset >= scroll)
        .map(|(idx, _)| idx)
        .unwrap_or(cumulative_heights.len().saturating_sub(1))
}

pub(crate) struct ChatViewWidget<'a> {
    pub(crate) chat: &'a ChatView,
    pub(crate) thinking: bool,
    pub(crate) spinner_frame: usize,
}

impl Widget for ChatViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let entries = self.chat.entries();
        let entry_count = entries.len();

        if entry_count == 0 {
            // 如果正在思考但消息列表为空，显示 spinner
            if self.thinking {
                render_spinner(area, buf, self.spinner_frame);
            }
            return;
        }

        let cumulative_heights = compute_cumulative_heights(entries, area.width);
        let total_height = cumulative_heights
            .last()
            .copied()
            .unwrap_or(0)
            .saturating_add(estimate_entry_height(
                &entries[entries.len() - 1],
                area.width,
            ));

        let scroll_offset = self.chat.scroll_offset() as u16;
        let start_idx = find_first_visible(self.chat.scroll_offset(), &cumulative_heights);

        let mut current_y = area.y;
        let max_y = area.y + area.height;

        for idx in start_idx..entry_count {
            let entry = &entries[idx];

            if entry.text.is_empty()
                && matches!(entry.role, ChatRole::Assistant)
                && entry.reasoning.is_none()
            {
                continue;
            }

            let estimated_height = estimate_entry_height(entry, area.width);

            // 检查是否超出显示区域
            if current_y >= max_y {
                break;
            }

            let available_height = max_y.saturating_sub(current_y);
            let msg_height = estimated_height.min(available_height);

            let msg_area = Rect {
                x: area.x,
                y: current_y,
                width: area.width,
                height: msg_height,
            };

            // 判断是否是最后一条且正在流式
            let is_last = idx == entry_count - 1;
            let is_streaming = is_last && self.thinking;

            MessageBubble {
                role: entry.role,
                content: &entry.text,
                is_streaming,
                reasoning: entry.reasoning.as_deref(),
            }
            .render(msg_area, buf);

            current_y += estimated_height;
        }
    }
}

fn render_spinner(area: Rect, buf: &mut ratatui::buffer::Buffer, spinner_frame: usize) {
    let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner_char = spinner_chars[spinner_frame % spinner_chars.len()];
    let t = (spinner_frame % 8) as f32 / 8.0;
    let r = (ui_palette::active::brand_yunxi().0 as f32
        + t * (ui_palette::active::brand_yunxi_shimmer().0 as f32
            - ui_palette::active::brand_yunxi().0 as f32)) as u8;
    let g = (ui_palette::active::brand_yunxi().1 as f32
        + t * (ui_palette::active::brand_yunxi_shimmer().1 as f32
            - ui_palette::active::brand_yunxi().1 as f32)) as u8;
    let b = (ui_palette::active::brand_yunxi().2 as f32
        + t * (ui_palette::active::brand_yunxi_shimmer().2 as f32
            - ui_palette::active::brand_yunxi().2 as f32)) as u8;
    let gradient = Color::Rgb(r, g, b);
    let muted = Color::Rgb(
        ui_palette::active::text_muted().0,
        ui_palette::active::text_muted().1,
        ui_palette::active::text_muted().2,
    );
    let spinner_line = Line::from(vec![
        Span::styled("┃ ", Style::default().fg(gradient)),
        Span::styled(spinner_char, Style::default().fg(gradient)),
        Span::styled(" thinking...", Style::default().fg(muted)),
    ]);
    Paragraph::new(Text::from(vec![spinner_line])).render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::components::chat_view::{ChatEntry, ChatRole};

    #[test]
    fn estimate_wrapped_lines_empty() {
        assert_eq!(estimate_wrapped_lines("", 80), 1);
    }

    #[test]
    fn estimate_wrapped_lines_single_line() {
        assert_eq!(estimate_wrapped_lines("hello", 80), 1);
    }

    #[test]
    fn estimate_wrapped_lines_wraps_at_width() {
        assert_eq!(estimate_wrapped_lines("abcdefghij", 5), 2);
    }

    #[test]
    fn estimate_wrapped_lines_cjk_width() {
        assert_eq!(estimate_wrapped_lines("中文测试", 4), 2);
        assert_eq!(estimate_wrapped_lines("中文测试", 2), 4);
    }

    #[test]
    fn estimate_entry_height_basic() {
        let entry = ChatEntry {
            role: ChatRole::User,
            text: "Hello".to_string(),
            reasoning: None,
        };
        let height = estimate_entry_height(&entry, 80);
        assert!(height >= 3);
    }

    #[test]
    fn estimate_entry_height_with_reasoning() {
        let entry = ChatEntry {
            role: ChatRole::Assistant,
            text: "Answer".to_string(),
            reasoning: Some("Step 1\nStep 2".to_string()),
        };
        let height_no_reasoning = estimate_entry_height(&entry, 80);
        let entry_no_reasoning = ChatEntry {
            role: ChatRole::Assistant,
            text: "Answer".to_string(),
            reasoning: None,
        };
        let height_plain = estimate_entry_height(&entry_no_reasoning, 80);
        assert!(height_no_reasoning > height_plain);
    }

    #[test]
    fn compute_cumulative_heights_basic() {
        let entries = vec![
            ChatEntry {
                role: ChatRole::User,
                text: "Hi".to_string(),
                reasoning: None,
            },
            ChatEntry {
                role: ChatRole::Assistant,
                text: "Hello".to_string(),
                reasoning: None,
            },
        ];
        let heights = compute_cumulative_heights(&entries, 80);
        assert_eq!(heights.len(), 2);
        assert_eq!(heights[0], 0);
        assert!(heights[1] > heights[0]);
    }

    #[test]
    fn find_first_visible_basic() {
        let heights = vec![0, 5, 10, 15];
        assert_eq!(find_first_visible(0, &heights), 0);
        assert_eq!(find_first_visible(3, &heights), 1);
        assert_eq!(find_first_visible(5, &heights), 1);
        assert_eq!(find_first_visible(12, &heights), 3);
    }

    #[test]
    fn find_first_visible_exceeds_all() {
        let heights = vec![0, 5, 10];
        assert_eq!(find_first_visible(20, &heights), 2);
    }

    #[test]
    fn chat_view_widget_renders_with_scroll_offset() {
        let mut chat = ChatView::new();
        for i in 0..20 {
            chat.push(ChatEntry {
                role: ChatRole::User,
                text: format!("Message {i}"),
                reasoning: None,
            });
        }
        chat.scroll_to_bottom(10);

        let backend = ratatui::backend::TestBackend::new(80, 20);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                ChatViewWidget {
                    chat: &chat,
                    thinking: false,
                    spinner_frame: 0,
                }
                .render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn chat_view_widget_renders_empty_with_thinking() {
        let chat = ChatView::new();
        let backend = ratatui::backend::TestBackend::new(80, 10);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                ChatViewWidget {
                    chat: &chat,
                    thinking: true,
                    spinner_frame: 0,
                }
                .render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn chat_view_widget_renders_narrow_width() {
        let mut chat = ChatView::new();
        chat.push(ChatEntry {
            role: ChatRole::Assistant,
            text: "This is a long message that should wrap at narrow widths".to_string(),
            reasoning: None,
        });

        let backend = ratatui::backend::TestBackend::new(20, 10);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                ChatViewWidget {
                    chat: &chat,
                    thinking: false,
                    spinner_frame: 0,
                }
                .render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn chat_view_widget_renders_zero_height() {
        let mut chat = ChatView::new();
        chat.push(ChatEntry {
            role: ChatRole::User,
            text: "Hello".to_string(),
            reasoning: None,
        });

        let backend = ratatui::backend::TestBackend::new(80, 0);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                ChatViewWidget {
                    chat: &chat,
                    thinking: false,
                    spinner_frame: 0,
                }
                .render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn estimate_entry_height_empty_text() {
        let entry = ChatEntry {
            role: ChatRole::Assistant,
            text: "".to_string(),
            reasoning: None,
        };
        let height = estimate_entry_height(&entry, 80);
        assert!(height >= 3);
    }

    #[test]
    fn estimate_entry_height_reasoning_only() {
        let entry = ChatEntry {
            role: ChatRole::Assistant,
            text: "".to_string(),
            reasoning: Some("Thinking process".to_string()),
        };
        let height = estimate_entry_height(&entry, 80);
        assert!(height >= 5);
    }

    #[test]
    fn find_first_visible_empty() {
        let heights: Vec<u16> = vec![];
        assert_eq!(find_first_visible(0, &heights), 0);
    }
}
