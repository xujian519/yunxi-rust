//! Ratatui 会话选择器组件（弹出式覆盖层，支持筛选高亮与光标导航）。

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::tui::components::session_picker::SessionPicker;
use crate::tui::ui_palette::{brand, content_color, dim_color, highlight};

pub(crate) struct SessionPickerWidget<'a> {
    pub(crate) picker: &'a SessionPicker,
}

impl Widget for SessionPickerWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);

        let sessions = self.picker.visible_sessions();
        let line_count = if sessions.is_empty() {
            1
        } else {
            sessions.len()
        };
        let popup_height = std::cmp::min(area.height, line_count as u16 + 6);
        let popup_width = std::cmp::min(area.width, 72);
        let popup_area = centered_rect(popup_width, popup_height, area);

        let block = Block::default()
            .title(" 选择会话 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(brand())));

        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let mut lines: Vec<Line<'_>> = Vec::new();

        // Filter line
        let filter_text = self.picker.filter();
        if filter_text.is_empty() {
            lines.push(Line::from(Span::styled(
                "  输入筛选会话 ID/路径 · ↑↓ 选择 · Enter 切换 · Esc 取消",
                Style::default().fg(Color::Indexed(dim_color())),
            )));
        } else {
            let label = Span::styled("  筛选:", Style::default().fg(Color::Indexed(dim_color())));
            let text = Span::styled(
                format!(" {} ", truncate_id(filter_text, inner.width as usize)),
                Style::default()
                    .fg(Color::Indexed(content_color()))
                    .add_modifier(Modifier::BOLD),
            );
            let hint = Span::styled(
                "(Backspace 删除 · Esc 清空)",
                Style::default().fg(Color::Indexed(dim_color())),
            );
            lines.push(Line::from(vec![label, text, hint]));
        }

        // Blank separator
        lines.push(Line::from(""));

        // Session list / empty states
        let all_sessions = self.picker.all_sessions();
        if all_sessions.is_empty() {
            lines.push(Line::from(Span::styled(
                "  暂无已保存会话",
                Style::default().fg(Color::Indexed(dim_color())),
            )));
        } else if sessions.is_empty() {
            lines.push(Line::from(Span::styled(
                "  无匹配会话",
                Style::default().fg(Color::Indexed(dim_color())),
            )));
        } else {
            let active_id = self.picker.active_session_id();
            let selected = self.picker.selected_visible_index();
            for (vis_idx, session) in sessions.iter().enumerate() {
                let is_active = session.id == active_id;
                let is_selected = vis_idx == selected;

                let cursor_span = Span::styled(
                    if is_selected { "▸" } else { " " },
                    Style::default()
                        .fg(Color::Indexed(highlight()))
                        .add_modifier(Modifier::BOLD),
                );
                let marker_span = Span::styled(
                    if is_active { " ●" } else { " ○" },
                    Style::default().fg(if is_active {
                        Color::Indexed(brand())
                    } else {
                        Color::Indexed(dim_color())
                    }),
                );
                let sep = Span::raw(" ");

                let id_display = truncate_id(&session.id, 18);
                let id_span = Span::styled(
                    format!(" {:<18}", id_display),
                    Style::default().fg(Color::Indexed(content_color())),
                );

                let msg_span = Span::styled(
                    format!("msgs={:<4}", session.message_count),
                    Style::default().fg(Color::Indexed(dim_color())),
                );

                let path_span = Span::styled(
                    format!(" {}", session.path.display()),
                    Style::default().fg(Color::Indexed(dim_color())),
                );

                lines.push(Line::from(vec![
                    cursor_span,
                    marker_span,
                    sep,
                    id_span,
                    msg_span,
                    path_span,
                ]));
            }
        }

        // Blank + footer
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "— 会话选择器 —",
            Style::default().fg(Color::Indexed(dim_color())),
        )));

        Paragraph::new(lines).render(inner, buf);
    }
}

fn truncate_id(id: &str, max: usize) -> String {
    if id.chars().count() <= max {
        return id.to_string();
    }
    id.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
}

fn centered_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(height)) / 2),
            Constraint::Length(height),
            Constraint::Length((r.height.saturating_sub(height)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(width)) / 2),
            Constraint::Length(width),
            Constraint::Length((r.width.saturating_sub(width)) / 2),
        ])
        .split(popup_layout[1])[1]
}
