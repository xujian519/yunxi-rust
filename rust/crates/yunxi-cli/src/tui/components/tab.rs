use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Paragraph;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub struct TabItem {
    pub label: String,
    pub content: String,
    pub action: Option<Action>,
    pub disabled: bool,
    pub closable: bool,
}

#[derive(Debug, Clone)]
pub struct TabStyle {
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub active_bg: Color,
    pub active_fg: Color,
    pub disabled_bg: Color,
    pub disabled_fg: Color,
    pub pinned_bg: Color,
    pub pinned_fg: Color,
    pub border: bool,
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            active_bg: Color::Rgb(139, 176, 240),
            active_fg: Color::Rgb(13, 13, 18),
            disabled_bg: Color::Rgb(22, 22, 30),
            disabled_fg: Color::Rgb(106, 106, 128),
            pinned_bg: Color::Rgb(248, 189, 78),
            pinned_fg: Color::Rgb(13, 13, 18),
            border: false,
        }
    }
}

pub struct Tab {
    state: ComponentState,
    tabs: Vec<TabItem>,
    active_index: usize,
    closable: bool,
    pinned_indices: HashSet<usize>,
    style: Box<TabStyle>,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("tab")),
            tabs: Vec::new(),
            active_index: 0,
            closable: true,
            pinned_indices: HashSet::new(),
            style: Box::new(TabStyle::default()),
        }
    }

    pub fn with_tabs(mut self, tabs: Vec<TabItem>) -> Self {
        self.tabs = tabs;
        if !self.tabs.is_empty() && self.active_index >= self.tabs.len() {
            self.active_index = 0;
        }
        self
    }

    pub fn with_active_index(mut self, index: usize) -> Self {
        self.active_index = index.min(self.tabs.len().saturating_sub(1));
        self
    }

    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    pub fn with_pinned_indices(mut self, indices: HashSet<usize>) -> Self {
        self.pinned_indices = indices;
        self
    }

    pub fn with_style(mut self, style: TabStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn add_tab(&mut self, tab: TabItem) {
        self.tabs.push(tab);
    }

    pub fn remove_tab(&mut self, index: usize) -> Option<TabItem> {
        if self.pinned_indices.contains(&index) {
            return None;
        }
        if index >= self.tabs.len() {
            return None;
        }

        let removed = self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.active_index = 0;
        } else if self.active_index >= self.tabs.len() {
            self.active_index = self.tabs.len() - 1;
        } else if index < self.active_index {
            self.active_index = self.active_index.saturating_sub(1);
        }

        let mut new_pinned = HashSet::new();
        for &pinned_idx in &self.pinned_indices {
            if pinned_idx > index {
                new_pinned.insert(pinned_idx - 1);
            } else if pinned_idx < index {
                new_pinned.insert(pinned_idx);
            }
        }
        self.pinned_indices = new_pinned;

        Some(removed)
    }

    pub fn set_active_index(&mut self, index: usize) {
        self.active_index = index.min(self.tabs.len().saturating_sub(1));
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = (self.active_index + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_index = if self.active_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_index - 1
            };
        }
    }

    pub fn pin_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.pinned_indices.insert(index);
        }
    }

    pub fn unpin_tab(&mut self, index: usize) {
        self.pinned_indices.remove(&index);
    }

    pub fn toggle_pin(&mut self, index: usize) {
        if self.pinned_indices.contains(&index) {
            self.unpin_tab(index);
        } else {
            self.pin_tab(index);
        }
    }

    pub fn get_active_tab(&self) -> Option<&TabItem> {
        self.tabs.get(self.active_index)
    }

    pub fn is_pinned(&self, index: usize) -> bool {
        self.pinned_indices.contains(&index)
    }
}

impl Component for Tab {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible || self.tabs.is_empty() {
            return;
        }

        let mut x = area.x;
        let y = area.y;
        let height = 3;

        for (idx, tab) in self.tabs.iter().enumerate() {
            let is_active = idx == self.active_index;
            let is_pinned = self.pinned_indices.contains(&idx);

            let (bg_color, fg_color) = if tab.disabled {
                (self.style.disabled_bg, self.style.disabled_fg)
            } else if is_pinned {
                (self.style.pinned_bg, self.style.pinned_fg)
            } else if is_active {
                (self.style.active_bg, self.style.active_fg)
            } else {
                (self.style.normal_bg, self.style.normal_fg)
            };

            let mut style = Style::default().bg(bg_color).fg(fg_color);
            if is_active && self.state.focused {
                style = style.add_modifier(Modifier::BOLD);
            }

            let mut label = tab.label.clone();
            if is_pinned {
                label.insert(0, '📌');
            }

            let close_indicator = if self.closable && tab.closable && !tab.disabled {
                if is_active {
                    "×"
                } else {
                    " "
                }
            } else {
                ""
            };

            let text = format!(" {} {} ", label, close_indicator);
            let tab_width = (text.len() as u16).min(area.width - (x - area.x));

            if x + tab_width > area.x + area.width {
                break;
            }

            let tab_area = Rect {
                x,
                y,
                width: tab_width,
                height,
            };

            let paragraph = Paragraph::new(text).style(style);

            if self.style.border {
                paragraph.render(
                    Rect {
                        x: tab_area.x,
                        y: tab_area.y,
                        width: tab_area.width - 1,
                        height: tab_area.height,
                    },
                    buf,
                );
            } else {
                paragraph.render(tab_area, buf);
            }

            x += tab_width;
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Tab => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        self.next_tab();
                    } else {
                        self.prev_tab();
                    }
                    ActionResult::Handled
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.next_tab();
                    ActionResult::Handled
                }
                KeyCode::Char('w') | KeyCode::Char('c') => {
                    if self.closable {
                        let active_index = self.active_index;
                        if let Some(tab) = self.tabs.get(active_index) {
                            if tab.closable && !self.pinned_indices.contains(&active_index) {
                                self.remove_tab(active_index);
                            }
                        }
                    }
                    ActionResult::Handled
                }
                KeyCode::Char('p') => {
                    self.toggle_pin(self.active_index);
                    ActionResult::Handled
                }
                KeyCode::Char('1')
                | KeyCode::Char('2')
                | KeyCode::Char('3')
                | KeyCode::Char('4')
                | KeyCode::Char('5')
                | KeyCode::Char('6')
                | KeyCode::Char('7')
                | KeyCode::Char('8')
                | KeyCode::Char('9') => {
                    let index = match key.code {
                        KeyCode::Char('1') => 0,
                        KeyCode::Char('2') => 1,
                        KeyCode::Char('3') => 2,
                        KeyCode::Char('4') => 3,
                        KeyCode::Char('5') => 4,
                        KeyCode::Char('6') => 5,
                        KeyCode::Char('7') => 6,
                        KeyCode::Char('8') => 7,
                        KeyCode::Char('9') => 8,
                        _ => 0,
                    };
                    if index < self.tabs.len() {
                        self.set_active_index(index);
                        if let Some(tab) = self.get_active_tab() {
                            if !tab.disabled {
                                if let Some(ref action) = tab.action {
                                    return ActionResult::Action(action.clone());
                                }
                            }
                        }
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
