//! 消息气泡组件 - 像素级复刻 Claude Code 消息样式
//!
//! 特性：
//! - 用户消息右对齐，蓝色左边框
//! - AI 消息左对齐，紫色左边框
//! - 宽度限制（用户 85%，AI 90%）
//! - 流式响应时显示闪烁光标 ▎

use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::tui::components::chat_view::ChatRole;
use crate::tui::markdown;
use crate::tui::ui_palette;

/// 消息气泡渲染参数
pub(crate) struct MessageBubble<'a> {
    pub role: ChatRole,
    pub content: &'a str,
    pub is_streaming: bool,
}

impl Widget for MessageBubble<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let is_user = self.role == ChatRole::User;

        // ── 1. 计算气泡宽度 ──
        let bubble_width = if is_user {
            ((area.width as f32) * 0.85) as u16
        } else {
            ((area.width as f32) * 0.90) as u16
        }
        .min(area.width)
        .max(10);

        // ── 2. 计算气泡水平位置 ──
        let bubble_x = if is_user {
            area.x + area.width.saturating_sub(bubble_width)
        } else {
            area.x
        };

        let bubble_area = Rect {
            x: bubble_x,
            y: area.y,
            width: bubble_width.min(area.width),
            height: area.height,
        };

        // ── 3. 选择样式 ──
        let (border_color, label_style, bg_color) = match self.role {
            ChatRole::User => (
                Color::Rgb(
                    ui_palette::active::label_you().0,
                    ui_palette::active::label_you().1,
                    ui_palette::active::label_you().2,
                ),
                Style::default()
                    .fg(Color::Rgb(
                        ui_palette::active::label_you().0,
                        ui_palette::active::label_you().1,
                        ui_palette::active::label_you().2,
                    ))
                    .add_modifier(Modifier::BOLD),
                Color::Rgb(
                    ui_palette::active::bg_message_user().0,
                    ui_palette::active::bg_message_user().1,
                    ui_palette::active::bg_message_user().2,
                ),
            ),
            ChatRole::Assistant => (
                Color::Rgb(
                    ui_palette::active::label_yunxi().0,
                    ui_palette::active::label_yunxi().1,
                    ui_palette::active::label_yunxi().2,
                ),
                Style::default()
                    .fg(Color::Rgb(
                        ui_palette::active::label_yunxi().0,
                        ui_palette::active::label_yunxi().1,
                        ui_palette::active::label_yunxi().2,
                    ))
                    .add_modifier(Modifier::BOLD),
                Color::Rgb(
                    ui_palette::active::bg_message_ai().0,
                    ui_palette::active::bg_message_ai().1,
                    ui_palette::active::bg_message_ai().2,
                ),
            ),
            ChatRole::System => (
                Color::Rgb(
                    ui_palette::active::text_muted().0,
                    ui_palette::active::text_muted().1,
                    ui_palette::active::text_muted().2,
                ),
                Style::default()
                    .fg(Color::Rgb(
                        ui_palette::active::text_muted().0,
                        ui_palette::active::text_muted().1,
                        ui_palette::active::text_muted().2,
                    ))
                    .add_modifier(Modifier::BOLD),
                Color::Rgb(
                    ui_palette::active::bg_primary().0,
                    ui_palette::active::bg_primary().1,
                    ui_palette::active::bg_primary().2,
                ),
            ),
        };

        // ── 4. 创建气泡块（左边框强调） ──
        let bubble_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(bg_color));

        bubble_block.render(bubble_area, buf);

        // ── 5. 计算内部内容区域 ──
        let inner_area = bubble_area.inner(Margin {
            horizontal: 2,
            vertical: 1,
        });

        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        // ── 6. 构建标签行 ──
        let label_text = match self.role {
            ChatRole::User => "You",
            ChatRole::Assistant => "yunxi",
            ChatRole::System => "System",
        };
        let label_line = Line::from(vec![Span::styled(label_text, label_style)]);

        // ── 7. 构建消息内容 ──
        let primary = Color::Rgb(
            ui_palette::active::text_primary().0,
            ui_palette::active::text_primary().1,
            ui_palette::active::text_primary().2,
        );

        let body = match self.role {
            ChatRole::Assistant | ChatRole::System => markdown::markdown_to_text(self.content),
            ChatRole::User => Text::from(Line::from(Span::styled(
                self.content,
                Style::default().fg(primary),
            ))),
        };

        // ── 8. 组合所有行 ──
        let mut all_lines: Vec<Line> = Vec::new();
        all_lines.push(label_line);
        all_lines.extend(body.lines.into_iter().map(|l| {
            let spans: Vec<Span> = l
                .spans
                .into_iter()
                .map(|s| {
                    let mut style = s.style;
                    if style.fg.is_none() {
                        style = style.fg(primary);
                    }
                    style = style.bg(bg_color);
                    Span::styled(s.content, style)
                })
                .collect();
            Line::from(spans)
        }));

        // 流式光标
        if self.is_streaming {
            let brand = Color::Rgb(
                ui_palette::active::brand_yunxi().0,
                ui_palette::active::brand_yunxi().1,
                ui_palette::active::brand_yunxi().2,
            );
            all_lines.push(Line::from(Span::styled(
                "▎",
                Style::default().fg(brand).add_modifier(Modifier::BOLD),
            )));
        }

        // ── 9. 渲染 ──
        let text = Text::from(all_lines);
        let paragraph = Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .style(Style::default().bg(bg_color));

        paragraph.render(inner_area, buf);
    }
}
