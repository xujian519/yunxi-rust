use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Widget};

use crate::tui::components::chat_view::{ChatRole, ChatView};
use crate::tui::markdown;
use crate::tui::ui_palette;

pub(crate) struct ChatViewWidget<'a> {
    pub(crate) chat: &'a ChatView,
    pub(crate) thinking: bool,
    pub(crate) spinner_frame: usize,
}

fn role_accent_color(role: ChatRole) -> Color {
    match role {
        ChatRole::User => Color::Rgb(
            ui_palette::LABEL_YOU.0,
            ui_palette::LABEL_YOU.1,
            ui_palette::LABEL_YOU.2,
        ),
        ChatRole::Assistant => Color::Rgb(
            ui_palette::LABEL_YUNXI.0,
            ui_palette::LABEL_YUNXI.1,
            ui_palette::LABEL_YUNXI.2,
        ),
        ChatRole::System => Color::Rgb(
            ui_palette::TEXT_MUTED.0,
            ui_palette::TEXT_MUTED.1,
            ui_palette::TEXT_MUTED.2,
        ),
    }
}

fn role_label(role: ChatRole) -> &'static str {
    match role {
        ChatRole::User => "You",
        ChatRole::Assistant => "yunxi",
        ChatRole::System => "System",
    }
}

impl Widget for ChatViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line> = Vec::new();
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

        for entry in self.chat.entries() {
            if entry.text.is_empty() && matches!(entry.role, ChatRole::Assistant) {
                continue;
            }
            let is_error = entry.text.starts_with("Error")
                || entry.text.starts_with("error")
                || entry.text.contains("Unauthorized")
                || entry.text.contains("401")
                || entry.text.contains("403")
                || entry.text.contains("500");

            let accent = role_accent_color(entry.role);
            let label = role_label(entry.role);

            let gutter = Span::styled("┃ ", Style::default().fg(accent));
            let label_span = Span::styled(
                label,
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            );

            let body = match entry.role {
                ChatRole::System if is_error => Text::from(Line::from(Span::styled(
                    entry.text.clone(),
                    Style::default()
                        .fg(Color::Rgb(
                            ui_palette::ERROR.0,
                            ui_palette::ERROR.1,
                            ui_palette::ERROR.2,
                        ))
                        .add_modifier(Modifier::BOLD),
                ))),
                ChatRole::Assistant | ChatRole::System => markdown::markdown_to_text(&entry.text),
                ChatRole::User => Text::from(Line::from(Span::styled(
                    entry.text.clone(),
                    Style::default().fg(primary),
                ))),
            };

            if body.lines.len() == 1 {
                let mut spans = vec![
                    gutter,
                    label_span,
                    Span::styled(": ", Style::default().fg(muted)),
                ];
                spans.extend(body.lines.into_iter().flat_map(|l| l.spans));
                lines.push(Line::from(spans));
            } else {
                lines.push(Line::from(vec![
                    gutter.clone(),
                    label_span,
                    Span::styled(":", Style::default().fg(muted)),
                ]));
                for line in body.lines {
                    let indent = Span::styled("┃ ", Style::default().fg(accent));
                    let mut spans = vec![indent];
                    spans.extend(line.spans);
                    lines.push(Line::from(spans));
                }
            }
            lines.push(Line::from(Span::styled(" ", Style::default())));
        }

        if self.thinking {
            let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let spinner_char = spinner_chars[self.spinner_frame % spinner_chars.len()];
            let t = (self.spinner_frame % 8) as f32 / 8.0;
            let r = (ui_palette::BRAND_YUNXI.0 as f32
                + t * (ui_palette::BRAND_YUNXI_SHIMMER.0 as f32 - ui_palette::BRAND_YUNXI.0 as f32))
                as u8;
            let g = (ui_palette::BRAND_YUNXI.1 as f32
                + t * (ui_palette::BRAND_YUNXI_SHIMMER.1 as f32 - ui_palette::BRAND_YUNXI.1 as f32))
                as u8;
            let b = (ui_palette::BRAND_YUNXI.2 as f32
                + t * (ui_palette::BRAND_YUNXI_SHIMMER.2 as f32 - ui_palette::BRAND_YUNXI.2 as f32))
                as u8;
            let gradient = Color::Rgb(r, g, b);
            lines.push(Line::from(vec![
                Span::styled("┃ ", Style::default().fg(gradient)),
                Span::styled(spinner_char, Style::default().fg(gradient)),
                Span::styled(
                    " thinking...",
                    Style::default().fg(muted).add_modifier(Modifier::ITALIC),
                ),
            ]));
        }

        Paragraph::new(Text::from(lines))
            .scroll((self.chat.scroll_offset() as u16, 0))
            .render(area, buf);
    }
}
