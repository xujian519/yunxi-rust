use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::ui_palette;

const BAR_WIDTH: usize = 10;

pub(crate) struct StatusBarWidget<'a> {
    pub(crate) model: &'a str,
    pub(crate) permission_mode: &'a str,
    pub(crate) input_tokens: u32,
    pub(crate) output_tokens: u32,
    pub(crate) active_tool: Option<&'a str>,
}

impl Widget for StatusBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let brand = Color::Rgb(
            ui_palette::BRAND_YUNXI.0,
            ui_palette::BRAND_YUNXI.1,
            ui_palette::BRAND_YUNXI.2,
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
        let usage_fill = Color::Rgb(
            ui_palette::USAGE_FILL.0,
            ui_palette::USAGE_FILL.1,
            ui_palette::USAGE_FILL.2,
        );
        let usage_empty = Color::Rgb(
            ui_palette::USAGE_EMPTY.0,
            ui_palette::USAGE_EMPTY.1,
            ui_palette::USAGE_EMPTY.2,
        );

        let mut spans: Vec<Span> = Vec::new();

        spans.push(Span::styled(
            " yunxi ",
            Style::default().fg(brand).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled("| ", Style::default().fg(muted)));

        spans.push(Span::styled(self.model, Style::default().fg(secondary)));

        let total_tokens = self.input_tokens.saturating_add(self.output_tokens).max(1);
        let ctx_limit = 200_000u32;
        let percent = (total_tokens as f32 / ctx_limit as f32 * 100.0).min(100.0);
        let filled = ((percent / 100.0) * BAR_WIDTH as f32).round() as usize;
        let filled = filled.min(BAR_WIDTH);
        let empty = BAR_WIDTH.saturating_sub(filled);

        let filled_bar = "█".repeat(filled);
        let empty_bar = "░".repeat(empty);

        spans.push(Span::styled(" | ", Style::default().fg(muted)));
        spans.push(Span::styled(&filled_bar, Style::default().fg(usage_fill)));
        spans.push(Span::styled(&empty_bar, Style::default().fg(usage_empty)));
        spans.push(Span::styled(
            format!(" {:3.0}%", percent),
            Style::default().fg(secondary),
        ));

        if !self.permission_mode.is_empty() {
            spans.push(Span::styled(" | ", Style::default().fg(muted)));
            spans.push(Span::styled(
                self.permission_mode,
                Style::default().fg(secondary),
            ));
        }

        if let Some(tool) = self.active_tool {
            spans.push(Span::styled(" | ", Style::default().fg(muted)));
            spans.push(Span::styled(
                format!("▶ {tool}"),
                Style::default().fg(brand),
            ));
        }

        let full_text: String = spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<Vec<_>>()
            .concat();
        let line = Line::from(spans);
        Paragraph::new(line).render(
            Rect::new(
                area.x,
                area.y,
                area.width.min(full_text.len() as u16 + 2),
                1,
            ),
            buf,
        );
    }
}
