use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::ui_palette::{dim_color, input_bg_color, input_text_color, user_role_color};

pub(crate) struct InputBarWidget<'a> {
    pub(crate) content: &'a str,
    pub(crate) slash_completion_count: usize,
    pub(crate) slash_completion: Option<&'a crate::tui::slash_complete::SlashCompletion>,
}

impl Widget for InputBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line> = Vec::new();

        if let Some(menu) = self.slash_completion {
            for (i, (display, _)) in menu.matches.iter().take(6).enumerate() {
                if i == menu.selected {
                    lines.push(Line::from(Span::styled(
                        format!("▸ {display}"),
                        Style::default()
                            .fg(Color::Indexed(231))
                            .bg(Color::Indexed(25)),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("  {display}"),
                        Style::default().fg(Color::Indexed(dim_color())),
                    )));
                }
            }
        }

        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Indexed(dim_color())))
            .style(Style::default().bg(Color::Indexed(input_bg_color())));

        let prompt = Span::styled(
            "❯ ",
            Style::default()
                .fg(Color::Indexed(user_role_color()))
                .add_modifier(Modifier::BOLD),
        );

        let content_span = if self.content.is_empty() {
            Span::styled(
                "在此输入消息…",
                Style::default().fg(Color::Indexed(dim_color())),
            )
        } else {
            Span::styled(
                self.content,
                Style::default().fg(Color::Indexed(input_text_color())),
            )
        };

        let mut spans = vec![prompt, content_span];

        if self.slash_completion_count > 0 && self.slash_completion.is_none() {
            let hint = Span::styled(
                format!(" ({} 补全)", self.slash_completion_count),
                Style::default().fg(Color::Indexed(dim_color())),
            );
            spans.push(hint);
        }

        let input_line = Line::from(spans);
        lines.push(input_line);

        let hint_text = self
            .slash_completion
            .map(|m| {
                let (display, _) = &m.matches[m.selected];
                format!("Tab 应用 · ↑↓ 选择 · {}", display)
            })
            .unwrap_or_else(|| "Shift+Enter 换行 · Enter 发送 · Tab 补全".to_string());

        lines.push(Line::from(Span::styled(
            hint_text,
            Style::default().fg(Color::Indexed(dim_color())),
        )));

        let text = ratatui::text::Text::from(lines);
        let inner = block.inner(area);
        block.render(area, buf);
        Paragraph::new(text).render(inner, buf);
    }
}
