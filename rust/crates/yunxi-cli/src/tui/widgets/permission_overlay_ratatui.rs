use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use runtime::PermissionRequest;

use crate::tui::ui_palette::{content_color, highlight, user_role_color};

pub(crate) struct PermissionOverlayWidget<'a> {
    pub(crate) request: &'a PermissionRequest,
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

impl Widget for PermissionOverlayWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .title(" ⚠ 工具权限请求 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(highlight())));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            format!(" 工具: {}", self.request.tool_name),
            Style::default()
                .fg(Color::Indexed(highlight()))
                .add_modifier(Modifier::BOLD),
        )));

        lines.push(Line::from(""));

        let input_preview = truncate_chars(&self.request.input, 60);
        lines.push(Line::from(Span::styled(
            format!(" 输入: {}", input_preview),
            Style::default().fg(Color::Indexed(content_color())),
        )));

        lines.push(Line::from(Span::styled(
            format!(
                " 模式: {} → {}",
                self.request.current_mode.as_str(),
                self.request.required_mode.as_str()
            ),
            Style::default().fg(Color::Indexed(content_color())),
        )));

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " 按 y 允许 · n 拒绝 · a 允许所有",
            Style::default()
                .fg(Color::Indexed(user_role_color()))
                .add_modifier(Modifier::BOLD),
        )));

        Paragraph::new(lines).render(inner, buf);
    }
}
