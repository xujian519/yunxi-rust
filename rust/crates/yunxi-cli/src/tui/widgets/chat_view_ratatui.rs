use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::tui::components::chat_view::{ChatRole, ChatView};

pub(crate) struct ChatViewWidget<'a> {
    pub(crate) chat: &'a ChatView,
    pub(crate) thinking: bool,
    pub(crate) spinner_frame: usize,
}

impl Widget for ChatViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default().style(Style::default());

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
            let body = Span::styled(
                &entry.text,
                Style::default().fg(Color::Indexed(252)),
            );

            lines.push(Line::from(vec![role_label, colon, body]));
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

        let text = Text::from(lines);
        Paragraph::new(text)
            .block(block)
            .scroll((self.chat.scroll_offset() as u16, 0))
            .render(area, buf);
    }
}