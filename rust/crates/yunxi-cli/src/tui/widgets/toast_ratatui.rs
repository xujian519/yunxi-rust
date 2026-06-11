//! Toast notification widget — transient, auto-dismissing overlays.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use crate::tui::core::app::ui::{ToastData, ToastLevel};
use crate::tui::ui_palette;

pub(crate) struct ToastWidget<'a> {
    pub toast: &'a ToastData,
}

impl<'a> ToastWidget<'a> {
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let (icon, fg) = match self.toast.level {
            ToastLevel::Info => ("ℹ", Color::Cyan),
            ToastLevel::Success => ("✓", Color::Green),
            ToastLevel::Warning => ("⚠", Color::Yellow),
            ToastLevel::Error => ("✗", Color::Red),
        };

        let bg_c = ui_palette::active::bg_primary();
        let bg = Color::Rgb(bg_c.0, bg_c.1, bg_c.2);

        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", icon),
                Style::default().fg(fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {} ", self.toast.message),
                Style::default().fg(Color::White).bg(bg),
            ),
        ]);

        let paragraph =
            Paragraph::new(ratatui::text::Text::from(line)).style(Style::default().bg(bg));

        // Render at bottom-center of the screen, above the input bar
        let width = (self.toast.message.len() + 6).min(area.width as usize) as u16;
        let toast_area = Rect {
            x: area.x + (area.width.saturating_sub(width)) / 2,
            y: area.y + area.height.saturating_sub(5),
            width: width.min(area.width),
            height: 1,
        };

        frame.render_widget(Clear, toast_area);
        frame.render_widget(paragraph, toast_area);
    }
}
