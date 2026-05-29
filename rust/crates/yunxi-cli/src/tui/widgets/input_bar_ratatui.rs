use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub(crate) struct InputBarWidget<'a> {
    pub(crate) content: &'a str,
    pub(crate) slash_completion_count: usize,
}

impl Widget for InputBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Indexed(245)));

        let prompt = Span::styled(
            "❯ ",
            Style::default()
                .fg(Color::Indexed(183))
                .add_modifier(Modifier::BOLD),
        );

        let content_span = Span::styled(
            self.content,
            Style::default().fg(Color::Indexed(252)),
        );

        let mut spans = vec![prompt, content_span];

        if self.slash_completion_count > 0 {
            let hint = Span::styled(
                format!(" ({} 补全)", self.slash_completion_count),
                Style::default().fg(Color::Indexed(245)),
            );
            spans.push(hint);
        }

        let line = Line::from(spans);
        Paragraph::new(line).block(block).render(
            Rect::new(area.x, area.y, area.width, 4),
            buf,
        );
    }
}