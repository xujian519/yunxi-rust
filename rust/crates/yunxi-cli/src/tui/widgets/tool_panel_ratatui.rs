use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::tui::components::tool_panel::ToolPanel;
use crate::tui::ui_palette;
use crate::tui::widgets::tool_block::{ToolBlock, ToolBlockStatus};

pub(crate) struct ToolPanelWidget<'a> {
    pub(crate) tools: &'a ToolPanel,
    pub(crate) focus_index: usize,
}

impl Widget for ToolPanelWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let border = Color::Rgb(
            ui_palette::active::border().0,
            ui_palette::active::border().1,
            ui_palette::active::border().2,
        );
        let muted = Color::Rgb(
            ui_palette::active::text_muted().0,
            ui_palette::active::text_muted().1,
            ui_palette::active::text_muted().2,
        );

        let block = Block::default()
            .title(" Tools ")
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(border));

        let inner = block.inner(area);

        if self.tools.is_empty() {
            let msg = Line::from(Span::styled(
                "Tool output panel (F2 toggle)",
                Style::default().fg(muted),
            ));
            Paragraph::new(vec![msg]).block(block).render(area, buf);
            return;
        }

        // 渲染每个工具条目为 ToolBlock
        let mut current_y = inner.y;
        for (idx, entry) in self.tools.entries().iter().enumerate() {
            // 估算高度：标题1行 + 展开时的输出
            let estimated_height = if entry.collapsed {
                2u16 // 标题行 + 摘要行
            } else {
                let detail_lines = entry.detail.lines().count() as u16;
                (1 + detail_lines).min(10) // 最多10行
            };

            if current_y + estimated_height > inner.y + inner.height {
                break;
            }

            let entry_area = Rect {
                x: inner.x,
                y: current_y,
                width: inner.width,
                height: estimated_height,
            };

            let status = if entry.is_error {
                ToolBlockStatus::Failed
            } else {
                ToolBlockStatus::Success
            };

            ToolBlock {
                name: &entry.name,
                arguments: "", // 当前 ToolEntry 没有参数字段
                status,
                duration_ms: None,
                error: if entry.is_error {
                    Some(&entry.detail)
                } else {
                    None
                },
                expanded: !entry.collapsed,
                output: if entry.collapsed {
                    None
                } else {
                    Some(&entry.detail)
                },
                focused: idx == self.focus_index,
            }
            .render(entry_area, buf);

            current_y += estimated_height;
        }

        // 渲染边框
        block.render(area, buf);
    }
}
