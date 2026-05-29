use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

const BRAND_COLOR: Color = Color::Indexed(183);
const DIM_COLOR: Color = Color::Indexed(245);

pub(crate) struct StatusBarWidget<'a> {
    pub(crate) model: &'a str,
    pub(crate) permission_mode: &'a str,
    pub(crate) session_id: &'a str,
    pub(crate) input_tokens: u32,
    pub(crate) output_tokens: u32,
    pub(crate) cost_usd: f64,
    pub(crate) active_tool: Option<&'a str>,
}

impl Widget for StatusBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut spans = Vec::new();

        spans.push(Span::styled(
            self.model,
            Style::default().fg(BRAND_COLOR).add_modifier(Modifier::BOLD),
        ));

        spans.push(Span::styled(" | ", Style::default().fg(DIM_COLOR)));
        spans.push(Span::styled(
            self.permission_mode,
            Style::default().fg(DIM_COLOR),
        ));

        if !self.session_id.is_empty() {
            spans.push(Span::styled(" | ", Style::default().fg(DIM_COLOR)));
            spans.push(Span::styled(
                format!("会话: {}", self.session_id),
                Style::default().fg(DIM_COLOR),
            ));
        }

        if self.input_tokens > 0 || self.output_tokens > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(DIM_COLOR)));
            spans.push(Span::styled(
                format!("Token: {}入/{}出", self.input_tokens, self.output_tokens),
                Style::default().fg(DIM_COLOR),
            ));
        }

        if let Some(tool) = self.active_tool {
            spans.push(Span::styled(" | ", Style::default().fg(DIM_COLOR)));
            spans.push(Span::styled(
                format!("工具: {tool}"),
                Style::default().fg(Color::Indexed(214)),
            ));
        }

        let cost_text = if self.cost_usd > 0.0 {
            format!(" ${:.4}", self.cost_usd)
        } else {
            String::new()
        };

        let cost_width = if cost_text.is_empty() {
            0u16
        } else {
            cost_text.len() as u16
        };

        let left_width = area.width.saturating_sub(cost_width);
        let left_line = Line::from(spans);
        Paragraph::new(left_line).render(
            Rect::new(area.x, area.y, left_width, 1),
            buf,
        );

        if !cost_text.is_empty() {
            Paragraph::new(Line::from(Span::styled(
                cost_text,
                Style::default().fg(DIM_COLOR),
            )))
            .render(Rect::new(area.x + left_width, area.y, cost_width, 1), buf);
        }
    }
}
