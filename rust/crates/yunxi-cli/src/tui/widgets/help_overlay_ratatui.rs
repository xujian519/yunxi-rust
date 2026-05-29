use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

const HELP_TEXT: &[(&str, &str)] = &[
    ("Enter", "发送消息"),
    ("Shift+Enter", "换行"),
    ("Esc / Ctrl+C", "清空输入 / 退出"),
    ("Ctrl+H / F1", "帮助"),
    ("Ctrl+G", "引导面板"),
    ("Ctrl+I", "中断当前轮次"),
    ("Ctrl+U", "导入预填"),
    ("Ctrl+F", "搜索预填"),
    ("Tab", "斜杠命令补全"),
    ("F2", "切换工具面板"),
    ("F3", "专利导航循环"),
    ("1-6", "专利导航快捷键"),
    ("[/]", "证据面板滚动"),
    ("j/k / ↑/↓", "滚动（输入框空时）"),
    ("g/G", "滚动到顶部/底部"),
];

pub(crate) struct HelpOverlay;

impl Widget for HelpOverlay {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);

        let popup_width = std::cmp::min(area.width, 50);
        let popup_height = std::cmp::min(area.height, (HELP_TEXT.len() as u16) + 4);
        let popup_area = centered_rect(popup_width, popup_height, area);

        let block = Block::default()
            .title(" 快捷键 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(183)));

        let mut lines = Vec::new();
        for (key, desc) in HELP_TEXT {
            let key_span = Span::styled(
                format!(" {:<16}", key),
                Style::default()
                    .fg(Color::Indexed(214))
                    .add_modifier(Modifier::BOLD),
            );
            let desc_span = Span::styled(
                *desc,
                Style::default().fg(Color::Indexed(252)),
            );
            lines.push(Line::from(vec![key_span, desc_span]));
        }

        Paragraph::new(lines)
            .block(block)
            .render(popup_area, buf);
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