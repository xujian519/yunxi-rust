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
        let selected_bg = Color::Rgb(74, 74, 106);
        let description_color = Color::Rgb(160, 160, 176);

        if let Some(menu) = self.slash_completion {
            for (i, item) in menu.matches.iter().take(6).enumerate() {
                let is_selected = i == menu.selected;
                let icon_style = if is_selected {
                    Style::default().fg(accent)
                } else {
                    Style::default().fg(muted)
                };
                let name_style = if is_selected {
                    Style::default()
                        .fg(Color::Rgb(232, 232, 237))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(primary)
                };
                let desc_style = if is_selected {
                    Style::default().fg(description_color)
                } else {
                    Style::default().fg(muted)
                };

                let prefix = if is_selected { "▸ " } else { "  " };
                let mut spans = vec![
                    Span::styled(prefix, name_style),
                    Span::styled(format!("{} ", item.icon), icon_style),
                    Span::styled(item.display.clone(), name_style),
                ];

                if !item.description.is_empty() {
                    spans.push(Span::styled(format!("  {}", item.description), desc_style));
                }

                let line_style = if is_selected {
                    Style::default().bg(selected_bg)
                } else {
                    Style::default()
                };

                lines.push(Line::from(spans).style(line_style));
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
                let item = &m.matches[m.selected];
                format!("Tab apply · ↑↓ select · {}", item.display)
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
