use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::ui_palette::{accent, dim_color, user_role_color};

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
        let brand = Span::styled(
            "云熙智能体",
            Style::default().fg(Color::Indexed(user_role_color())),
        );
        let flower = Span::styled("🌸", Style::default().fg(Color::Indexed(accent())));
        let ver = Span::styled(
            format!(" v{}", self.version),
            Style::default().fg(Color::Indexed(dim_color())),
        );
        let model = Span::styled(
            format!("  {}", self.model),
            Style::default().fg(Color::Indexed(accent())),
        );
        let line = Line::from(vec![brand, flower, ver, model]);
        Paragraph::new(line).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_title_line() {
        let backend = TestBackend::new(60, 1);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                TitleBar::new("deepseek-v4-pro", "0.1.0").render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }
}
