use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::tui::ui_palette::{content_color, highlight, user_role_color};

const HELP_TEXT: &[(&str, &str)] = &[
    ("Enter", "发送消息"),
    ("Shift+Enter", "换行"),
    ("Ctrl+P / F3", "命令面板"),
    ("Ctrl+B", "切换工具面板"),
    ("Ctrl+D", "切换主题"),
    ("Ctrl+C / Esc", "清空输入或退出"),
    ("Ctrl+G", "人机引导（预填模板）"),
    ("Ctrl+I", "中断轮次并打开引导"),
    ("Ctrl+U", "预填 /import 导入材料"),
    ("Ctrl+F", "预填 /search 检索对话"),
    ("Ctrl+H / F1", "显示帮助"),
    ("F2", "切换工具面板"),
    ("j / ↓", "向下滚动"),
    ("k / ↑", "向上滚动"),
    ("g", "滚动到顶部"),
    ("G", "滚动到底部"),
    ("/", "输入斜杠命令"),
    ("鼠标拖选", "选中文字后 Cmd/Ctrl+C 复制"),
    ("Ctrl+Shift+C", "复制对话到剪贴板"),
    ("q", "退出 TUI 模式"),
];

pub(crate) struct HelpOverlay;

impl Widget for HelpOverlay {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);

        let line_count = HELP_TEXT.len() + 4; // header sep + 2 footer lines + padding
        let popup_width = std::cmp::min(area.width, 50);
        let popup_height = std::cmp::min(area.height, line_count as u16 + 4);
        let popup_area = centered_rect(popup_width, popup_height, area);

        let block = Block::default()
            .title(" 快捷键 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(user_role_color())));

        let mut lines = Vec::new();
        for (key, desc) in HELP_TEXT {
            let key_span = Span::styled(
                format!(" {:<16}", key),
                Style::default()
                    .fg(Color::Indexed(highlight()))
                    .add_modifier(Modifier::BOLD),
            );
            let desc_span =
                Span::styled(*desc, Style::default().fg(Color::Indexed(content_color())));
            lines.push(Line::from(vec![key_span, desc_span]));
        }

        // Footer lines
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " 斜杠命令：输入 /help 打开完整命令列表（分页器）",
            Style::default().fg(Color::Indexed(content_color())),
        )));
        lines.push(Line::from(Span::styled(
            " 按任意键关闭帮助",
            Style::default()
                .fg(Color::Indexed(highlight()))
                .add_modifier(Modifier::BOLD),
        )));

        Paragraph::new(lines).block(block).render(popup_area, buf);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height.saturating_sub(percent_y)) / 2),
            Constraint::Length(percent_y),
            Constraint::Length((r.height.saturating_sub(percent_y)) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length((r.width.saturating_sub(percent_x)) / 2),
            Constraint::Length(percent_x),
            Constraint::Length((r.width.saturating_sub(percent_x)) / 2),
        ])
        .split(popup_layout[1])[1]
}
