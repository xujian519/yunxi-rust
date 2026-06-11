//! Route-driven sidebar widget.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::ui_palette;

/// Navigation items for the sidebar.
const NAV_ITEMS: &[(&str, &str, &str)] = &[
    ("💬", "Chat", "/home"),
    ("🔧", "Tools", "/tools"),
    ("📋", "Sessions", "/sessions"),
    ("⚙️", "Settings", "/settings"),
];

pub(crate) struct SidebarWidget<'a> {
    pub current_route: &'a str,
    pub focus_index: usize,
}

impl<'a> SidebarWidget<'a> {
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let bg_c = ui_palette::active::bg_secondary();
        let bg = Color::Rgb(bg_c.0, bg_c.1, bg_c.2);
        let tc = ui_palette::active::text_primary();
        let text = Color::Rgb(tc.0, tc.1, tc.2);
        let ac = ui_palette::active::brand_yunxi();
        let accent = Color::Rgb(ac.0, ac.1, ac.2);
        let bc = ui_palette::active::border();
        let border_c = Color::Rgb(bc.0, bc.1, bc.2);
        let sc = ui_palette::active::text_secondary();
        let secondary = Color::Rgb(sc.0, sc.1, sc.2);

        let lines: Vec<Line> = NAV_ITEMS
            .iter()
            .enumerate()
            .map(|(i, (icon, label, route))| {
                let is_active = self.current_route == *route;
                let is_focused = i == self.focus_index;

                let style = if is_active {
                    Style::default().fg(accent).add_modifier(Modifier::BOLD)
                } else if is_focused {
                    Style::default().fg(text).add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default().fg(secondary)
                };

                let indicator = if is_active { "▸ " } else { "  " };
                Line::from(Span::styled(
                    format!("{}{} {}", indicator, icon, label),
                    style,
                ))
            })
            .collect();

        let block = Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(border_c))
            .title(" 导航 ")
            .style(Style::default().bg(bg));

        let paragraph = Paragraph::new(ratatui::text::Text::from(lines)).block(block);
        frame.render_widget(paragraph, area);
    }
}

/// Get the number of navigation items.
pub fn nav_item_count() -> usize {
    NAV_ITEMS.len()
}

/// Get the route for a navigation item by index.
pub fn nav_route(index: usize) -> Option<&'static str> {
    NAV_ITEMS.get(index).map(|(_, _, route)| *route)
}
