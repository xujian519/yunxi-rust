use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::components::chat_view::wrap_line;
use crate::tui::components::tool_panel::ToolPanel;
use crate::tui::ui_palette::{content_color, dim_color, highlight};

pub(crate) struct ToolPanelWidget<'a> {
    pub(crate) tools: &'a ToolPanel,
}

/// Truncate a string to fit within `max_width` display columns (CJK-aware), appending "…" if needed.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let mut width = 0;
    let mut result = String::new();
    for ch in s.chars() {
        let cw = if ch.is_ascii() { 1 } else { 2 };
        if width + cw > max_width {
            result.push('…');
            break;
        }
        result.push(ch);
        width += cw;
    }
    result
}

impl Widget for ToolPanelWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let block = Block::default()
            .title(" 工具 ")
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::Indexed(dim_color())));

        let inner = block.inner(area);
        let inner_width = inner.width as usize;

        // Empty state
        if self.tools.is_empty() {
            let msg = Line::from(Span::styled(
                "工具输出面板 (F2 切换显示)",
                Style::default().fg(Color::Indexed(dim_color())),
            ));
            Paragraph::new(vec![msg]).block(block).render(area, buf);
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        for entry in self.tools.entries() {
            // Icon + name line
            let (icon, icon_color) = if entry.is_error {
                ("✗", Color::Red)
            } else {
                ("✓", Color::Green)
            };

            let name_max = inner_width.saturating_sub(2); // icon char + space
            let name_str = truncate_to_width(&entry.name, name_max);

            lines.push(Line::from(vec![
                Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
                Span::styled(
                    name_str,
                    Style::default()
                        .fg(Color::Indexed(highlight()))
                        .add_modifier(Modifier::BOLD),
                ),
            ]));

            if entry.collapsed {
                // Collapsed: show summary
                let line_count = entry.detail.lines().count();
                let summary = format!("  ▸ 展开输出 ({line_count} 行)");
                lines.push(Line::from(Span::styled(
                    summary,
                    Style::default().fg(Color::Indexed(dim_color())),
                )));
            } else {
                // Expanded: wrap detail lines with "  " prefix
                let detail_width = inner_width.saturating_sub(2); // "  " prefix
                for detail_line in entry.detail.lines() {
                    let wrapped = wrap_line(detail_line, detail_width);
                    for wl in &wrapped {
                        let mut spans: Vec<Span> = vec![Span::raw("  ")];
                        if !wl.is_empty() {
                            spans.push(Span::styled(
                                wl.clone(),
                                Style::default().fg(Color::Indexed(content_color())),
                            ));
                        }
                        lines.push(Line::from(spans));
                    }
                }
            }
        }

        let scroll_y = self.tools.scroll_offset() as u16;
        Paragraph::new(lines)
            .block(block)
            .scroll((scroll_y, 0))
            .render(area, buf);
    }
}
