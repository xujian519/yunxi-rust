use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::components::chat_view::wrap_line;
use crate::tui::components::tool_panel::ToolPanel;
use crate::tui::ui_palette;

pub(crate) struct ToolPanelWidget<'a> {
    pub(crate) tools: &'a ToolPanel,
}

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
        let border = Color::Rgb(
            ui_palette::BORDER.0,
            ui_palette::BORDER.1,
            ui_palette::BORDER.2,
        );
        let muted = Color::Rgb(
            ui_palette::TEXT_MUTED.0,
            ui_palette::TEXT_MUTED.1,
            ui_palette::TEXT_MUTED.2,
        );
        let primary = Color::Rgb(
            ui_palette::TEXT_PRIMARY.0,
            ui_palette::TEXT_PRIMARY.1,
            ui_palette::TEXT_PRIMARY.2,
        );
        let success = Color::Rgb(
            ui_palette::SUCCESS.0,
            ui_palette::SUCCESS.1,
            ui_palette::SUCCESS.2,
        );
        let error = Color::Rgb(
            ui_palette::ERROR.0,
            ui_palette::ERROR.1,
            ui_palette::ERROR.2,
        );
        let brand = Color::Rgb(
            ui_palette::BRAND_YUNXI.0,
            ui_palette::BRAND_YUNXI.1,
            ui_palette::BRAND_YUNXI.2,
        );

        let block = Block::default()
            .title(" Tools ")
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(border));

        let inner = block.inner(area);
        let inner_width = inner.width as usize;

        if self.tools.is_empty() {
            let msg = Line::from(Span::styled(
                "Tool output panel (F2 toggle)",
                Style::default().fg(muted),
            ));
            Paragraph::new(vec![msg]).block(block).render(area, buf);
            return;
        }

        let mut lines: Vec<Line> = Vec::new();

        for entry in self.tools.entries() {
            let (icon, icon_color) = if entry.is_error {
                ("✗", error)
            } else {
                ("✓", success)
            };

            let name_max = inner_width.saturating_sub(3);
            let name_str = truncate_to_width(&entry.name, name_max);

            lines.push(Line::from(vec![
                Span::styled(format!("{icon} "), Style::default().fg(icon_color)),
                Span::styled(
                    name_str,
                    Style::default().fg(brand).add_modifier(Modifier::BOLD),
                ),
            ]));

            if entry.collapsed {
                let line_count = entry.detail.lines().count();
                let summary = format!("  ▸ {} lines", line_count);
                lines.push(Line::from(Span::styled(
                    summary,
                    Style::default().fg(muted),
                )));
            } else {
                let detail_width = inner_width.saturating_sub(2);
                for detail_line in entry.detail.lines() {
                    let wrapped = wrap_line(detail_line, detail_width);
                    for wl in &wrapped {
                        let mut spans: Vec<Span> = vec![Span::raw("  ")];
                        if !wl.is_empty() {
                            spans.push(Span::styled(wl.clone(), Style::default().fg(primary)));
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
