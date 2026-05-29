use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

const BRAND_COLOR: Color = Color::Indexed(183);
const ACCENT_COLOR: Color = Color::Indexed(213);
const DIM_COLOR: Color = Color::Indexed(245);

pub(crate) struct TitleBar<'a> {
    model: &'a str,
    version: &'a str,
    patent_mode: bool,
}

impl<'a> TitleBar<'a> {
    pub(crate) fn new(model: &'a str, version: &'a str, patent_mode: bool) -> Self {
        Self {
            model,
            version,
            patent_mode,
        }
    }
}

impl Widget for TitleBar<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let brand = Span::styled(
            "云熙智能体",
            Style::default().fg(BRAND_COLOR),
        );
        let flower = Span::styled("🌸", Style::default().fg(ACCENT_COLOR));
        let ver = Span::styled(
            format!(" v{}", self.version),
            Style::default().fg(DIM_COLOR),
        );
        let mode = if self.patent_mode {
            Span::styled(" [专利]", Style::default().fg(ACCENT_COLOR))
        } else {
            Span::styled("", Style::default())
        };
        let model = Span::styled(
            format!(" {}", self.model),
            Style::default().fg(DIM_COLOR),
        );
        let line = Line::from(vec![brand, flower, ver, mode, model]);
        Paragraph::new(line).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;

    #[test]
    fn title_bar_renders_model_name() {
        let widget = TitleBar::new("deepseek-v4", "0.1.0", false);
        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<Vec<&str>>()
            .join("");
        assert!(content.contains("云") && content.contains("熙") && content.contains("智") && content.contains("能") && content.contains("体"));
        assert!(content.contains("deepseek-v4"));
    }

    #[test]
    fn patent_mode_shows_label() {
        let widget = TitleBar::new("gpt-4", "0.1.0", true);
        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect::<Vec<&str>>()
            .join("");
        assert!(content.contains("专") && content.contains("利"));
    }
}