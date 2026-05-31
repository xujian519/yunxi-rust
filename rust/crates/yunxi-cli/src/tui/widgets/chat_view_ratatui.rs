use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::components::chat_view::{ChatRole, ChatView};
use crate::tui::markdown;
use crate::tui::ui_palette::{
    assistant_role_color, content_color, highlight, system_role_color, user_role_color,
};

pub(crate) struct ChatViewWidget<'a> {
    pub(crate) chat: &'a ChatView,
    pub(crate) thinking: bool,
    pub(crate) spinner_frame: usize,
}

impl Widget for ChatViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line> = Vec::new();

        for entry in self.chat.entries() {
            if entry.text.is_empty() && matches!(entry.role, ChatRole::Assistant) {
                continue;
            }
            let is_error = entry.text.starts_with("Error")
                || entry.text.starts_with("error")
                || entry.text.contains("Unauthorized")
                || entry.text.contains("401")
                || entry.text.contains("403")
                || entry.text.contains("500");

            let role_label = match entry.role {
                ChatRole::User => Span::styled(
                    "你",
                    Style::default()
                        .fg(Color::Indexed(highlight()))
                        .add_modifier(Modifier::BOLD),
                ),
                ChatRole::Assistant => Span::styled(
                    "云熙",
                    Style::default()
                        .fg(Color::Indexed(user_role_color()))
                        .add_modifier(Modifier::BOLD),
                ),
                ChatRole::System if is_error => Span::styled(
                    "⚠",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                ChatRole::System => Span::styled(
                    "系统",
                    Style::default().fg(Color::Indexed(system_role_color())),
                ),
            };

            let colon = Span::styled(": ", Style::default());

            let body = match entry.role {
                ChatRole::System if is_error => Text::from(Line::from(Span::styled(
                    entry.text.clone(),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ))),
                ChatRole::Assistant | ChatRole::System => markdown::markdown_to_text(&entry.text),
                ChatRole::User => Text::from(Line::from(Span::styled(
                    entry.text.clone(),
                    Style::default().fg(Color::Indexed(content_color())),
                ))),
            };

            if body.lines.len() == 1 {
                let mut spans = vec![role_label, colon];
                spans.extend(body.lines.into_iter().flat_map(|l| l.spans));
                lines.push(Line::from(spans));
            } else {
                lines.push(Line::from(vec![role_label, colon]));
                for line in body.lines {
                    let indent = Span::styled("  ", Style::default());
                    let mut spans = vec![indent];
                    spans.extend(line.spans);
                    lines.push(Line::from(spans));
                }
            }
            lines.push(Line::from(Span::styled(
                "─".repeat(area.width as usize),
                Style::default().fg(Color::DarkGray),
            )));
        }

        if self.thinking {
            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner_char = spinner_chars[self.spinner_frame % spinner_chars.len()];
            lines.push(Line::from(Span::styled(
                format!("{spinner_char} 思考中..."),
                Style::default()
                    .fg(Color::Indexed(assistant_role_color()))
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        Paragraph::new(Text::from(lines))
            .scroll((self.chat.scroll_offset() as u16, 0))
            .render(area, buf);
    }
}
