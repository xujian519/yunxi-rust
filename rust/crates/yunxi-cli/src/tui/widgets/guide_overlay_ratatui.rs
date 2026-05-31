use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Widget};

use crate::tui::ui_palette::{content_color, highlight};

pub(crate) struct GuideOverlayWidget {
    pub(crate) thinking: bool,
}

impl Widget for GuideOverlayWidget {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        Clear.render(area, buf);

        let block = Block::default()
            .title(" 📋 人机引导 ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(highlight())));

        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            " 引导模式：在底栏编辑指引内容后按 Enter 发送",
            Style::default()
                .fg(Color::Indexed(highlight()))
                .add_modifier(Modifier::BOLD),
        )));

        lines.push(Line::from(""));

        lines.push(Line::from(Span::styled(
            " 可用模板：",
            Style::default().fg(Color::Indexed(content_color())),
        )));

        lines.push(Line::from(Span::styled(
            "   /import   — 导入材料",
            Style::default().fg(Color::Indexed(content_color())),
        )));
        lines.push(Line::from(Span::styled(
            "   /search   — 检索对话",
            Style::default().fg(Color::Indexed(content_color())),
        )));
        lines.push(Line::from(Span::styled(
            "   /help     — 查看完整命令列表",
            Style::default().fg(Color::Indexed(content_color())),
        )));

        if self.thinking {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " ⏳ AI 正在思考...",
                Style::default()
                    .fg(Color::Indexed(highlight()))
                    .add_modifier(Modifier::BOLD),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " 按 Esc 退出引导模式",
            Style::default().fg(Color::Indexed(content_color())),
        )));

        Paragraph::new(lines).render(inner, buf);
    }
}
