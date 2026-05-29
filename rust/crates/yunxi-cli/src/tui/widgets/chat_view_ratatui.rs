use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::components::chat_view::{ChatRole, ChatView};
use crate::tui::markdown;

pub(crate) struct ChatViewWidget<'a> {
    pub(crate) chat: &'a ChatView,
    pub(crate) thinking: bool,
    pub(crate) spinner_frame: usize,
}

impl Widget for ChatViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line> = Vec::new();

        for entry in self.chat.entries() {
            let role_label = match entry.role {
                ChatRole::User => Span::styled(
                    "你",
                    Style::default()
                        .fg(Color::Indexed(214))
                        .add_modifier(Modifier::BOLD),
                ),
                ChatRole::Assistant => Span::styled(
                    "云熙",
                    Style::default()
                        .fg(Color::Indexed(183))
                        .add_modifier(Modifier::BOLD),
                ),
                ChatRole::System => Span::styled(
                    "系统",
                    Style::default().fg(Color::Indexed(245)),
                ),
            };

            let colon = Span::styled(": ", Style::default());

            let body = match entry.role {
                ChatRole::Assistant | ChatRole::System => {
                    markdown::markdown_to_text(&entry.text)
                }
                ChatRole::User => Text::from(Line::from(Span::styled(
                    entry.text.clone(),
                    Style::default().fg(Color::Indexed(252)),
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
            lines.push(Line::from(""));
        }

        if self.thinking {
            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner_char = spinner_chars[self.spinner_frame % spinner_chars.len()];
            lines.push(Line::from(Span::styled(
                format!("{spinner_char} 思考中..."),
                Style::default()
                    .fg(Color::Indexed(183))
                    .add_modifier(Modifier::ITALIC),
            )));
        }

        Paragraph::new(Text::from(lines))
            .scroll((self.chat.scroll_offset() as u16, 0))
            .render(area, buf);
    }
}
