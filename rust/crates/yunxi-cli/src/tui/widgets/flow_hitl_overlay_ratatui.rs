use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::session_meta::SuspendedFlowRecord;
use crate::tui::ui_palette::{content_color, highlight, user_role_color};

pub(crate) struct FlowHitlOverlayWidget<'a> {
    pub(crate) record: &'a SuspendedFlowRecord,
}

/// Character-boundary-safe truncation (CJK-aware).
fn truncate_chars(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().take(max_chars).collect();
    if chars.len() < s.chars().count() {
        format!("{}...", chars.into_iter().collect::<String>())
    } else {
        s.to_string()
    }
}

impl Widget for FlowHitlOverlayWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .title(" 🔄 工作流挂起 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(highlight())));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = Vec::new();

        let flow_name = self
            .record
            .flow_name
            .as_deref()
            .unwrap_or(&self.record.flow_id);
        lines.push(Line::from(Span::styled(
            format!(" 工作流: {}", flow_name),
            Style::default()
                .fg(Color::Indexed(highlight()))
                .add_modifier(Modifier::BOLD),
        )));

        lines.push(Line::from(""));

        if let Some(step_title) = &self.record.step_title {
            lines.push(Line::from(Span::styled(
                format!(" 当前步骤: {}", step_title),
                Style::default().fg(Color::Indexed(content_color())),
            )));
        }

        if let Some(desc) = &self.record.step_description {
            let desc_preview = truncate_chars(desc, 50);
            lines.push(Line::from(Span::styled(
                format!(" 描述: {}", desc_preview),
                Style::default().fg(Color::Indexed(content_color())),
            )));
        }

        if let Some(step) = self.record.current_step {
            lines.push(Line::from(Span::styled(
                format!(" 步骤序号: {}", step + 1),
                Style::default().fg(Color::Indexed(content_color())),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " 按 y 继续执行 · n 稍后处理",
            Style::default()
                .fg(Color::Indexed(user_role_color()))
                .add_modifier(Modifier::BOLD),
        )));

        Paragraph::new(lines).render(inner, buf);
    }
}
