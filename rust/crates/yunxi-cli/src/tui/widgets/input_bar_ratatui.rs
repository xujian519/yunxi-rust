use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::ui_palette;

pub(crate) struct InputBarWidget<'a> {
    pub(crate) content: &'a str,
    pub(crate) slash_completion_count: usize,
    pub(crate) slash_completion: Option<&'a crate::tui::slash_complete::SlashCompletion>,
}

impl Widget for InputBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line> = Vec::new();
        let muted = Color::Rgb(
            ui_palette::TEXT_MUTED.0,
            ui_palette::TEXT_MUTED.1,
            ui_palette::TEXT_MUTED.2,
        );
        let primary = Color::Rgb(
            ui_palette::TEXT_PRIMARY.0,
            ui_palette::TEXT_PRIMARY.1,
            ui_palette::TEXT_PRIMARY.2,
        );
        let accent = Color::Rgb(
            ui_palette::BRAND_YUNXI.0,
            ui_palette::BRAND_YUNXI.1,
            ui_palette::BRAND_YUNXI.2,
        );
        let tertiary_bg = Color::Rgb(
            ui_palette::BG_TERTIARY.0,
            ui_palette::BG_TERTIARY.1,
            ui_palette::BG_TERTIARY.2,
        );
        let border = Color::Rgb(
            ui_palette::BORDER.0,
            ui_palette::BORDER.1,
            ui_palette::BORDER.2,
        );

        if let Some(menu) = self.slash_completion {
            for (i, (display, _)) in menu.matches.iter().take(6).enumerate() {
                if i == menu.selected {
                    lines.push(Line::from(Span::styled(
                        format!("▸ {display}"),
                        Style::default()
                            .fg(Color::Rgb(232, 232, 237))
                            .bg(Color::Rgb(74, 74, 106)),
                    )));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("  {display}"),
                        Style::default().fg(muted),
                    )));
                }
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(tertiary_bg));

        let prompt = Span::styled(
            "❯ ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        );

        let content_span = if self.content.is_empty() {
            Span::styled("Ask anything...", Style::default().fg(muted))
        } else {
            Span::styled(self.content, Style::default().fg(primary))
        };

        let mut spans = vec![prompt, content_span];

        if self.slash_completion_count > 0 && self.slash_completion.is_none() {
            let hint = Span::styled(
                format!(
                    " ({} commands available, Tab to complete)",
                    self.slash_completion_count
                ),
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            );
            spans.push(hint);
        }

        let input_line = Line::from(spans);
        lines.push(input_line);

        let hint_text = self
            .slash_completion
            .map(|m| {
                let (display, _) = &m.matches[m.selected];
                format!("Tab apply · ↑↓ select · {}", display)
            })
            .unwrap_or_else(|| {
                "Enter send · Shift+Enter newline · / commands · Tab complete".to_string()
            });

        lines.push(Line::from(Span::styled(
            hint_text,
            Style::default().fg(muted),
        )));

        let text = ratatui::text::Text::from(lines);
        let inner = block.inner(area);
        block.render(area, buf);
        Paragraph::new(text).render(inner, buf);
    }
}
