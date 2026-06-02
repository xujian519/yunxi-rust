use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;

#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    pub label: String,
    pub value: Option<String>,
    pub action: Option<Action>,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct BreadcrumbStyle {
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub active_bg: Color,
    pub active_fg: Color,
    pub disabled_bg: Color,
    pub disabled_fg: Color,
    pub separator_fg: Color,
    pub clickable_indicator: bool,
}

impl Default for BreadcrumbStyle {
    fn default() -> Self {
        Self {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            active_bg: Color::Rgb(139, 176, 240),
            active_fg: Color::Rgb(13, 13, 18),
            disabled_bg: Color::Rgb(22, 22, 30),
            disabled_fg: Color::Rgb(106, 106, 128),
            separator_fg: Color::Rgb(106, 106, 128),
            clickable_indicator: true,
        }
    }
}

pub struct Breadcrumb {
    state: ComponentState,
    items: Vec<BreadcrumbItem>,
    separator: String,
    clickable: bool,
    truncate: bool,
    active_index: usize,
    style: Box<BreadcrumbStyle>,
}

impl Breadcrumb {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("breadcrumb")),
            items: Vec::new(),
            separator: " / ".to_string(),
            clickable: true,
            truncate: true,
            active_index: 0,
            style: Box::new(BreadcrumbStyle::default()),
        }
    }

    pub fn with_items(mut self, items: Vec<BreadcrumbItem>) -> Self {
        self.items = items;
        if !self.items.is_empty() && self.active_index >= self.items.len() {
            self.active_index = self.items.len() - 1;
        }
        self
    }

    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    pub fn with_clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    pub fn with_truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    pub fn with_active_index(mut self, index: usize) -> Self {
        self.active_index = index.min(self.items.len().saturating_sub(1));
        self
    }

    pub fn with_style(mut self, style: BreadcrumbStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn add_item(&mut self, item: BreadcrumbItem) {
        self.items.push(item);
        if self.active_index == 0 && !self.items.is_empty() {
            self.active_index = self.items.len() - 1;
        }
    }

    pub fn set_items(&mut self, items: Vec<BreadcrumbItem>) {
        self.items = items;
        if !self.items.is_empty() && self.active_index >= self.items.len() {
            self.active_index = self.items.len() - 1;
        }
    }

    pub fn set_separator(&mut self, separator: impl Into<String>) {
        self.separator = separator.into();
    }

    pub fn set_clickable(&mut self, clickable: bool) {
        self.clickable = clickable;
    }

    pub fn set_truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }

    pub fn set_active_index(&mut self, index: usize) {
        self.active_index = index.min(self.items.len().saturating_sub(1));
    }

    pub fn next_item(&mut self) {
        if !self.items.is_empty() {
            self.active_index = (self.active_index + 1).min(self.items.len() - 1);
        }
    }

    pub fn prev_item(&mut self) {
        if !self.items.is_empty() {
            self.active_index = self.active_index.saturating_sub(1);
        }
    }

    pub fn get_active_item(&self) -> Option<&BreadcrumbItem> {
        self.items.get(self.active_index)
    }

    pub fn get_path(&self) -> String {
        self.items
            .iter()
            .map(|item| item.value.as_ref().unwrap_or(&item.label).clone())
            .collect::<Vec<_>>()
            .join(&self.separator)
    }

    pub fn get_display_items(&self, max_width: usize) -> Vec<(usize, &BreadcrumbItem)> {
        if !self.truncate || self.items.len() <= 3 {
            return self.items.iter().enumerate().collect();
        }

        let separator_len = self.separator.len();
        let mut items = Vec::new();

        items.push((0, &self.items[0]));
        items.push((1, &self.items[1]));

        let total_width: usize = self.items[..2]
            .iter()
            .chain(self.items.iter().skip(self.items.len() - 2))
            .map(|item| item.label.len())
            .sum::<usize>()
            + separator_len * 2;

        if total_width + 3 <= max_width {
            items.push((2, &self.items[2]));
            items.push((self.items.len() - 2, &self.items[self.items.len() - 2]));
        }

        items.push((self.items.len() - 1, &self.items[self.items.len() - 1]));

        items
    }
}

impl Component for Breadcrumb {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible || self.items.is_empty() {
            return;
        }

        let display_items = self.get_display_items(area.width as usize);
        let display_items_len = display_items.len();

        let mut x = area.x;

        for (idx, item) in display_items.iter() {
            let is_active = *idx == self.active_index;
            let _is_clickable = self.clickable && item.action.is_some() && !item.disabled;

            let (bg_color, fg_color) = if item.disabled {
                (self.style.disabled_bg, self.style.disabled_fg)
            } else if is_active && self.state.focused {
                (self.style.active_bg, self.style.active_fg)
            } else {
                (self.style.normal_bg, self.style.normal_fg)
            };

            let style = Style::default().bg(bg_color).fg(fg_color);

            let label = if *idx > 0 {
                format!("{}{}", self.separator, item.label)
            } else {
                item.label.clone()
            };

            let label_len = label.len();
            let remaining_width = (area.x + area.width - x) as usize;

            if remaining_width < label_len {
                break;
            }

            let paragraph = Paragraph::new(label).style(style);
            paragraph.render(
                Rect {
                    x,
                    y: area.y,
                    width: label_len as u16,
                    height: area.height,
                },
                buf,
            );

            x += label_len as u16;

            if *idx < display_items_len - 1
                && (*idx + 1) < self.items.len()
                && display_items
                    .iter()
                    .position(|(i, _)| *i == *idx + 1)
                    .is_none()
            {
                let ellipsis = "...";
                let ellipsis_len = ellipsis.len();
                if (area.x + area.width - x) as usize >= ellipsis_len + self.separator.len() {
                    let paragraph = Paragraph::new(ellipsis)
                        .style(Style::default().fg(self.style.separator_fg));
                    paragraph.render(
                        Rect {
                            x,
                            y: area.y,
                            width: ellipsis_len as u16,
                            height: area.height,
                        },
                        buf,
                    );
                    x += ellipsis_len as u16;

                    let paragraph = Paragraph::new(self.separator.clone())
                        .style(Style::default().fg(self.style.separator_fg));
                    paragraph.render(
                        Rect {
                            x,
                            y: area.y,
                            width: self.separator.len() as u16,
                            height: area.height,
                        },
                        buf,
                    );
                    x += self.separator.len() as u16;
                }
            }
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        if !self.clickable {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    self.prev_item();
                    ActionResult::Handled
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.next_item();
                    ActionResult::Handled
                }
                KeyCode::Enter => {
                    if let Some(item) = self.get_active_item() {
                        if !item.disabled {
                            if let Some(ref action) = item.action {
                                return ActionResult::Action(action.clone());
                            }
                        }
                    }
                    ActionResult::Handled
                }
                KeyCode::Home => {
                    if !self.items.is_empty() {
                        self.set_active_index(0);
                    }
                    ActionResult::Handled
                }
                KeyCode::End => {
                    if !self.items.is_empty() {
                        self.set_active_index(self.items.len() - 1);
                    }
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
            _ => ActionResult::Ignored,
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }
}
