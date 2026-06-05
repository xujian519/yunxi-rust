//! 工具调用块组件 - 像素级复刻 Claude Code 工具样式
//!
//! 特性：
//! - 状态图标（▶/✓/✗/⋯）+ 工具名 + 参数预览
//! - 可折叠/展开
//! - 执行时间或错误信息显示

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};

use crate::tui::ui_palette;

fn tool_block_bg() -> Color {
    let c = ui_palette::active::bg_tertiary();
    Color::Rgb(c.0, c.1, c.2)
}

/// 工具调用状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolBlockStatus {
    Running,
    Success,
    Failed,
    Thinking,
}

impl ToolBlockStatus {
    fn icon(self) -> &'static str {
        match self {
            Self::Running => "▶",
            Self::Success => "✓",
            Self::Failed => "✗",
            Self::Thinking => "⋯",
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Running | Self::Thinking => Color::Rgb(
                ui_palette::active::brand_yunxi().0,
                ui_palette::active::brand_yunxi().1,
                ui_palette::active::brand_yunxi().2,
            ),
            Self::Success => Color::Rgb(
                ui_palette::active::success().0,
                ui_palette::active::success().1,
                ui_palette::active::success().2,
            ),
            Self::Failed => Color::Rgb(
                ui_palette::active::error().0,
                ui_palette::active::error().1,
                ui_palette::active::error().2,
            ),
        }
    }
}

/// 工具调用块
pub(crate) struct ToolBlock<'a> {
    pub name: &'a str,
    pub arguments: &'a str,
    pub status: ToolBlockStatus,
    pub duration_ms: Option<u64>,
    pub error: Option<&'a str>,
    pub expanded: bool,
    pub output: Option<&'a str>,
    pub focused: bool,
}

impl Widget for ToolBlock<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let primary = Color::Rgb(
            ui_palette::active::text_primary().0,
            ui_palette::active::text_primary().1,
            ui_palette::active::text_primary().2,
        );
        let accent = Color::Rgb(
            ui_palette::active::brand_yunxi().0,
            ui_palette::active::brand_yunxi().1,
            ui_palette::active::brand_yunxi().2,
        );
        let secondary = Color::Rgb(
            ui_palette::active::text_secondary().0,
            ui_palette::active::text_secondary().1,
            ui_palette::active::text_secondary().2,
        );
        let muted = Color::Rgb(
            ui_palette::active::text_muted().0,
            ui_palette::active::text_muted().1,
            ui_palette::active::text_muted().2,
        );

        // 背景块
        let bg_color = if self.focused {
            let c = ui_palette::active::border();
            Color::Rgb(c.0, c.1, c.2)
        } else {
            tool_block_bg()
        };
        let block = Block::default().style(Style::default().bg(bg_color));
        block.render(area, buf);

        // 状态后缀
        let status_suffix = match (self.status, self.duration_ms, self.error) {
            (ToolBlockStatus::Success, Some(ms), _) => format!(" ({:.1}s)", ms as f64 / 1000.0),
            (ToolBlockStatus::Failed, _, Some(err)) => format!(" (failed: {})", err),
            (ToolBlockStatus::Thinking, _, _) => " Thinking...".to_string(),
            _ => String::new(),
        };

        // 截断参数
        let args = truncate_arguments(self.arguments);

        // 构建工具行
        let focus_indicator = if self.focused {
            Span::styled("▸ ", Style::default().fg(accent))
        } else {
            Span::styled("  ", Style::default())
        };

        let tool_line = Line::from(vec![
            focus_indicator,
            Span::styled(self.status.icon(), Style::default().fg(self.status.color())),
            Span::styled(" ", Style::default()),
            Span::styled(
                self.name,
                Style::default().fg(primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default()),
            Span::styled(args, Style::default().fg(secondary)),
            Span::styled(status_suffix, Style::default().fg(muted)),
        ]);

        let paragraph = Paragraph::new(tool_line).style(Style::default().bg(bg_color));
        paragraph.render(area, buf);

        // 展开时显示输出
        if self.expanded {
            if let Some(output) = self.output {
                let output_y = area.y + 1;
                if output_y < area.y + area.height {
                    let output_height = area.height.saturating_sub(1);
                    if output_height > 0 {
                        let output_area = Rect {
                            x: area.x + 4,
                            y: output_y,
                            width: area.width.saturating_sub(4),
                            height: output_height,
                        };

                        let output_lines: Vec<Line> = output
                            .lines()
                            .map(|line| {
                                Line::from(Span::styled(line, Style::default().fg(secondary)))
                            })
                            .collect();

                        let output_para = Paragraph::new(output_lines)
                            .style(Style::default().bg(tool_block_bg()));
                        output_para.render(output_area, buf);
                    }
                }
            }
        }
    }
}

/// 截断参数预览
fn truncate_arguments(arguments: &str) -> String {
    const MAX_LEN: usize = 40;
    if arguments.len() <= MAX_LEN {
        arguments.to_string()
    } else {
        format!("{}...", &arguments[..MAX_LEN])
    }
}
