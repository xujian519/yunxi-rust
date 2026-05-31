use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::ui_palette;

pub(crate) struct TitleBar<'a> {
    model: &'a str,
    version: &'a str,
}

impl<'a> TitleBar<'a> {
    pub(crate) fn new(model: &'a str, version: &'a str) -> Self {
        Self { model, version }
    }
}

impl Widget for TitleBar<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let bg = Color::Rgb(
            ui_palette::BG_SECONDARY.0,
            ui_palette::BG_SECONDARY.1,
            ui_palette::BG_SECONDARY.2,
        );
        let muted = Color::Rgb(
            ui_palette::TEXT_MUTED.0,
            ui_palette::TEXT_MUTED.1,
            ui_palette::TEXT_MUTED.2,
        );
        let secondary = Color::Rgb(
            ui_palette::TEXT_SECONDARY.0,
            ui_palette::TEXT_SECONDARY.1,
            ui_palette::TEXT_SECONDARY.2,
        );
        let primary = Color::Rgb(
            ui_palette::TEXT_PRIMARY.0,
            ui_palette::TEXT_PRIMARY.1,
            ui_palette::TEXT_PRIMARY.2,
        );
        let brand = Color::Rgb(
            ui_palette::BRAND_YUNXI.0,
            ui_palette::BRAND_YUNXI.1,
            ui_palette::BRAND_YUNXI.2,
        );

        let brand_icon = Span::styled(
            "✢ ",
            Style::default().fg(brand).add_modifier(Modifier::BOLD),
        );

        let brand_name = Span::styled(
            "yunxi",
            Style::default().fg(primary).add_modifier(Modifier::BOLD),
        );

        let version_text = Span::styled(format!(" v{}", self.version), Style::default().fg(muted));

        let left_spans = vec![brand_icon, brand_name, version_text];
        let left_text: String = left_spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<Vec<_>>()
            .concat();

        let help_icon = Span::styled("?", Style::default().fg(muted));
        let help_text = Span::styled(" help", Style::default().fg(secondary));
        let right_text_str: String = vec!["? help".to_string()].concat();

        let pad = area
            .width
            .saturating_sub(
                left_text.chars().count() as u16 + right_text_str.chars().count() as u16,
            )
            .saturating_sub(1);
        let padding = Span::styled(" ".repeat(pad as usize), Style::default().bg(bg));

        let mut all_spans = left_spans;
        all_spans.push(padding);
        all_spans.push(help_icon);
        all_spans.push(help_text);

        let title_line = Line::from(all_spans);
        Paragraph::new(title_line)
            .style(Style::default().bg(bg))
            .render(
                Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: 1,
                },
                buf,
            );

        if area.height >= 2 {
            let sep_area = Rect {
                x: area.x,
                y: area.y + 1,
                width: area.width,
                height: 1,
            };
            let sep = Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb(
                    ui_palette::BORDER.0,
                    ui_palette::BORDER.1,
                    ui_palette::BORDER.2,
                )));
            sep.render(sep_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_title_line() {
        let backend = TestBackend::new(60, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                TitleBar::new("deepseek-v4-pro", "0.1.0").render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }
}
