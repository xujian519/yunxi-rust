use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::components::tool_panel::ToolPanel;

pub(crate) struct ToolPanelWidget<'a> {
    pub(crate) tools: &'a ToolPanel,
}

impl Widget for ToolPanelWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .title(" 工具 ")
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::Indexed(245)));

        let mut lines: Vec<Line> = Vec::new();

        for entry in self.tools.entries() {
            let name = Span::styled(
                &entry.name,
                Style::default()
                    .fg(Color::Indexed(214))
                    .add_modifier(Modifier::BOLD),
            );
            let desc = Span::styled(
                format!(" - {}", entry.detail),
                Style::default().fg(Color::Indexed(245)),
            );
            lines.push(Line::from(vec![name, desc]));
            lines.push(Line::from(""));
        }

        Paragraph::new(lines).block(block).render(area, buf);
    }
}