use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarPosition {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct SidebarItem {
    pub label: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub action: Option<Action>,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub struct SidebarStyle {
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub active_bg: Color,
    pub active_fg: Color,
    pub badge_bg: Color,
    pub badge_fg: Color,
    pub border: bool,
}

impl Default for SidebarStyle {
    fn default() -> Self {
        Self {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            active_bg: Color::Rgb(139, 176, 240),
            active_fg: Color::Rgb(13, 13, 18),
            badge_bg: Color::Rgb(232, 97, 107),
            badge_fg: Color::Rgb(255, 255, 255),
            border: true,
        }
    }
}

pub struct Sidebar {
    state: ComponentState,
    items: Vec<SidebarItem>,
    collapsed: bool,
    active_index: usize,
    position: SidebarPosition,
    style: Box<SidebarStyle>,
    list_state: ListState,
}

impl Sidebar {
    pub fn new() -> Self {
        let mut sidebar = Self {
            state: ComponentState::new(generate_component_id("sidebar")),
            items: Vec::new(),
            collapsed: false,
            active_index: 0,
            position: SidebarPosition::Left,
            style: Box::new(SidebarStyle::default()),
            list_state: ListState::default(),
        };
        sidebar.list_state.select(Some(0));
        sidebar
    }

    pub fn with_items(mut self, items: Vec<SidebarItem>) -> Self {
        self.items = items;
        if !self.items.is_empty() && self.active_index >= self.items.len() {
            self.active_index = 0;
            self.list_state.select(Some(0));
        }
        self
    }

    pub fn with_collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn with_active_index(mut self, index: usize) -> Self {
        self.active_index = index.min(self.items.len().saturating_sub(1));
        self.list_state.select(Some(self.active_index));
        self
    }

    pub fn with_position(mut self, position: SidebarPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_style(mut self, style: SidebarStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn add_item(&mut self, item: SidebarItem) {
        self.items.push(item);
        if self.items.len() == 1 {
            self.list_state.select(Some(0));
        }
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }

    pub fn set_collapsed(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    pub fn set_active_index(&mut self, index: usize) {
        self.active_index = index.min(self.items.len().saturating_sub(1));
        self.list_state.select(Some(self.active_index));
    }

    pub fn next_item(&mut self) {
        if !self.items.is_empty() {
            self.active_index = (self.active_index + 1) % self.items.len();
            self.list_state.select(Some(self.active_index));
        }
    }

    pub fn prev_item(&mut self) {
        if !self.items.is_empty() {
            self.active_index = if self.active_index == 0 {
                self.items.len() - 1
            } else {
                self.active_index - 1
            };
            self.list_state.select(Some(self.active_index));
        }
    }

    pub fn get_active_item(&self) -> Option<&SidebarItem> {
        self.items.get(self.active_index)
    }
}

impl Component for Sidebar {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible || self.items.is_empty() {
            return;
        }

        let (sidebar_width, _badge_width) = if self.collapsed {
            (4, 0)
        } else {
            let max_label = self
                .items
                .iter()
                .map(|item| item.label.len())
                .max()
                .unwrap_or(0);
            let icon_width = self.items.iter().any(|i| i.icon.is_some()) as usize * 3;
            let badge_len = self
                .items
                .iter()
                .filter_map(|i| i.badge.as_ref())
                .map(|b| b.len())
                .max()
                .unwrap_or(0);
            (icon_width + max_label + badge_len + 4, badge_len)
        };

        let sidebar_width = (sidebar_width as u16).min(area.width);
        let sidebar_height = area.height;

        let sidebar_area = if self.position == SidebarPosition::Left {
            Rect {
                x: area.x,
                y: area.y,
                width: sidebar_width,
                height: sidebar_height,
            }
        } else {
            Rect {
                x: area.x + area.width - sidebar_width,
                y: area.y,
                width: sidebar_width,
                height: sidebar_height,
            }
        };

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_active = idx == self.active_index;
                let (bg_color, fg_color) = if is_active {
                    (self.style.active_bg, self.style.active_fg)
                } else if item.disabled {
                    (self.style.normal_bg, Color::Rgb(106, 106, 128))
                } else {
                    (self.style.normal_bg, self.style.normal_fg)
                };

                let mut style = Style::default().bg(bg_color).fg(fg_color);
                if is_active && self.state.focused {
                    style = style.add_modifier(Modifier::BOLD);
                }

                let content = if self.collapsed {
                    item.icon.as_ref().unwrap_or(&"●".to_string()).clone()
                } else {
                    let mut text = String::new();
                    if let Some(ref icon) = item.icon {
                        text.push_str(icon);
                        text.push(' ');
                    }
                    text.push_str(&item.label);
                    if let Some(ref badge) = item.badge {
                        text.push(' ');
                        text.push_str(&format!("[{}]", badge));
                    }
                    text
                };

                ListItem::new(content).style(style)
            })
            .collect();

        let block = if self.style.border {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(self.style.normal_fg))
        } else {
            Block::default()
        };

        let list = List::new(list_items)
            .block(block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        list.render(sidebar_area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.prev_item();
                    ActionResult::Handled
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_item();
                    ActionResult::Handled
                }
                KeyCode::Char(' ') => {
                    self.toggle_collapse();
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
                    if index < self.items.len() {
                        self.set_active_index(index);
                        if let Some(item) = self.get_active_item() {
                            if !item.disabled {
                                if let Some(ref action) = item.action {
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
