use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::components::chat_view::{ChatRole, ChatView};
use crate::tui::ui_palette;
use crate::tui::widgets::message_bubble::MessageBubble;

pub(crate) struct ChatViewWidget<'a> {
    pub(crate) chat: &'a ChatView,
    pub(crate) thinking: bool,
    pub(crate) spinner_frame: usize,
}

impl Widget for ChatViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut current_y = area.y;
        let entry_count = self.chat.entries().len();

        for (idx, entry) in self.chat.entries().iter().enumerate() {
            if entry.text.is_empty() && matches!(entry.role, ChatRole::Assistant) {
                continue;
            }

            // 估算消息高度（标签1行 + 内容行数 + 内边距2行）
            let content_lines = entry.text.lines().count().max(1);
            let estimated_height = (content_lines + 3) as u16;

            // 检查是否超出显示区域
            if current_y + estimated_height > area.y + area.height {
                break;
            }

            let msg_area = Rect {
                x: area.x,
                y: current_y,
                width: area.width,
                height: estimated_height.min(area.y + area.height - current_y),
            };

            // 判断是否是最后一条且正在流式
            let is_last = idx == entry_count - 1;
            let is_streaming = is_last && self.thinking;

            MessageBubble {
                role: entry.role,
                content: &entry.text,
                is_streaming,
            }
            .render(msg_area, buf);

            current_y += estimated_height;
        }

        // 如果正在思考但消息列表为空，显示 spinner
        if self.thinking && entry_count == 0 {
            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner_char = spinner_chars[self.spinner_frame % spinner_chars.len()];
            let t = (self.spinner_frame % 8) as f32 / 8.0;
            let r = (ui_palette::BRAND_YUNXI.0 as f32
                + t * (ui_palette::BRAND_YUNXI_SHIMMER.0 as f32 - ui_palette::BRAND_YUNXI.0 as f32))
                as u8;
            let g = (ui_palette::BRAND_YUNXI.1 as f32
                + t * (ui_palette::BRAND_YUNXI_SHIMMER.1 as f32 - ui_palette::BRAND_YUNXI.1 as f32))
                as u8;
            let b = (ui_palette::BRAND_YUNXI.2 as f32
                + t * (ui_palette::BRAND_YUNXI_SHIMMER.2 as f32 - ui_palette::BRAND_YUNXI.2 as f32))
                as u8;
            let gradient = Color::Rgb(r, g, b);
            let muted = Color::Rgb(
                ui_palette::TEXT_MUTED.0,
                ui_palette::TEXT_MUTED.1,
                ui_palette::TEXT_MUTED.2,
            );
            let spinner_line = Line::from(vec![
                Span::styled("┃ ", Style::default().fg(gradient)),
                Span::styled(spinner_char, Style::default().fg(gradient)),
                Span::styled(" thinking...", Style::default().fg(muted)),
            ]);
            Paragraph::new(Text::from(vec![spinner_line])).render(area, buf);
        }
    }
}
