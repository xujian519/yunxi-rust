use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemType {
    Action,
    Separator,
    Submenu(String),
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: Option<Action>,
    pub item_type: MenuItemType,
    pub disabled: bool,
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    pub fn action(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            action: None,
            item_type: MenuItemType::Action,
            disabled: false,
            children: Vec::new(),
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            action: None,
            item_type: MenuItemType::Separator,
            disabled: false,
            children: Vec::new(),
        }
    }

    pub fn submenu(label: impl Into<String>, children: Vec<MenuItem>) -> Self {
        let label_str = label.into();
        Self {
            label: label_str.clone(),
            shortcut: None,
            action: None,
            item_type: MenuItemType::Submenu(label_str),
            disabled: false,
            children,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct MenuStyle {
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub focused_bg: Color,
    pub focused_fg: Color,
    pub disabled_bg: Color,
    pub disabled_fg: Color,
    pub separator_fg: Color,
    pub border: bool,
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            focused_bg: Color::Rgb(139, 176, 240),
            focused_fg: Color::Rgb(13, 13, 18),
            disabled_bg: Color::Rgb(22, 22, 30),
            disabled_fg: Color::Rgb(106, 106, 128),
            separator_fg: Color::Rgb(106, 106, 128),
            border: true,
        }
    }
}

pub struct Menu {
    state: ComponentState,
    items: Vec<MenuItem>,
    position: (u16, u16),
    visible: bool,
    parent: Option<String>,
    active_index: usize,
    style: Box<MenuStyle>,
    list_state: ListState,
}

impl Menu {
    pub fn new() -> Self {
        let mut menu = Self {
            state: ComponentState::new(generate_component_id("menu")),
            items: Vec::new(),
            position: (0, 0),
            visible: false,
            parent: None,
            active_index: 0,
            style: Box::new(MenuStyle::default()),
            list_state: ListState::default(),
        };
        menu.list_state.select(Some(0));
        menu
    }

    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self.update_active_index();
        self
    }

    pub fn with_position(mut self, x: u16, y: u16) -> Self {
        self.position = (x, y);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_parent(mut self, parent: Option<String>) -> Self {
        self.parent = parent;
        self
    }

    pub fn with_style(mut self, style: MenuStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
        self.update_active_index();
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.state.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.state.visible = false;
        self.active_index = 0;
        self.update_active_index();
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    pub fn set_position(&mut self, x: u16, y: u16) {
        self.position = (x, y);
    }

    pub fn set_active_index(&mut self, index: usize) {
        self.active_index = index;
        self.list_state.select(Some(index));
    }

    pub fn next_item(&mut self) {
        let mut next_idx = self.active_index;
        loop {
            next_idx = (next_idx + 1) % self.items.len();
            if next_idx == self.active_index {
                break;
            }
            if self.is_selectable(next_idx) {
                self.active_index = next_idx;
                self.list_state.select(Some(next_idx));
                break;
            }
        }
    }

    pub fn prev_item(&mut self) {
        let mut prev_idx = self.active_index;
        loop {
            prev_idx = if prev_idx == 0 {
                self.items.len() - 1
            } else {
                prev_idx - 1
            };
            if prev_idx == self.active_index {
                break;
            }
            if self.is_selectable(prev_idx) {
                self.active_index = prev_idx;
                self.list_state.select(Some(prev_idx));
                break;
            }
        }
    }

    pub fn get_active_item(&self) -> Option<&MenuItem> {
        self.items.get(self.active_index)
    }

    fn update_active_index(&mut self) {
        if !self.items.is_empty() {
            if !self.is_selectable(self.active_index) {
                for (idx, item) in self.items.iter().enumerate() {
                    if self.is_selectable_item(item) {
                        self.active_index = idx;
                        self.list_state.select(Some(idx));
                        break;
                    }
                }
            }
            self.list_state.select(Some(self.active_index));
        }
    }

    fn is_selectable(&self, index: usize) -> bool {
        if let Some(item) = self.items.get(index) {
            self.is_selectable_item(item)
        } else {
            false
        }
    }

    fn is_selectable_item(&self, item: &MenuItem) -> bool {
        matches!(
            item.item_type,
            MenuItemType::Action | MenuItemType::Submenu(_)
        )
    }

    pub fn is_submenu(&self, index: usize) -> bool {
        matches!(
            self.items.get(index).map(|i| &i.item_type),
            Some(MenuItemType::Submenu(_))
        )
    }

    pub fn get_submenu(&self, index: usize) -> Option<&Vec<MenuItem>> {
        if let Some(MenuItemType::Submenu(_)) = self.items.get(index).map(|i| &i.item_type) {
            Some(&self.items[index].children)
        } else {
            None
        }
    }

    pub fn get_submenu_name(&self, index: usize) -> Option<String> {
        if let MenuItemType::Submenu(name) =
            self.items.get(index).map(|i| i.item_type.clone()).unwrap()
        {
            Some(name)
        } else {
            None
        }
    }
}

impl Component for Menu {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.visible || !self.state.visible || self.items.is_empty() {
            return;
        }

        let max_width = self
            .items
            .iter()
            .map(|item| {
                if item.item_type == MenuItemType::Separator {
                    3
                } else {
                    let label_len = item.label.len();
                    let shortcut_len = item.shortcut.as_ref().map(|s| s.len() + 3).unwrap_or(0);
                    label_len + shortcut_len + 4
                }
            })
            .max()
            .unwrap_or(20) as u16;

        let menu_width = max_width.min(area.width);
        let menu_height = (self.items.len() as u16).min(area.height);

        let x = (self.position.0).min(area.x + area.width - menu_width);
        let y = (self.position.1).min(area.y + area.height - menu_height);

        let menu_area = Rect {
            x: x.max(area.x),
            y: y.max(area.y),
            width: menu_width,
            height: menu_height,
        };

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_focused = idx == self.active_index && self.state.focused;
                let (bg_color, fg_color) = if item.disabled {
                    (self.style.disabled_bg, self.style.disabled_fg)
                } else if item.item_type == MenuItemType::Separator {
                    (self.style.normal_bg, self.style.separator_fg)
                } else if is_focused {
                    (self.style.focused_bg, self.style.focused_fg)
                } else {
                    (self.style.normal_bg, self.style.normal_fg)
                };

                let mut style = Style::default().bg(bg_color).fg(fg_color);
                if is_focused {
                    style = style.add_modifier(Modifier::BOLD);
                }

                let content = if item.item_type == MenuItemType::Separator {
                    "─".repeat(menu_width as usize)
                } else {
                    let mut text = String::new();
                    text.push_str(&item.label);

                    if let Some(ref shortcut) = item.shortcut {
                        let spaces = menu_width as usize - item.label.len() - shortcut.len() - 4;
                        text.push_str(&" ".repeat(spaces.max(0)));
                        text.push(' ');
                        text.push_str(shortcut);
                    } else if matches!(item.item_type, MenuItemType::Submenu(_)) {
                        let spaces = menu_width as usize - item.label.len() - 3;
                        text.push_str(&" ".repeat(spaces.max(0)));
                        text.push('▶');
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

        let list = List::new(list_items).block(block);

        list.render(menu_area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.visible || !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Esc => {
                    self.hide();
                    ActionResult::Handled
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.prev_item();
                    ActionResult::Handled
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.next_item();
                    ActionResult::Handled
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(item) = self.get_active_item() {
                        if !item.disabled {
                            if matches!(item.item_type, MenuItemType::Submenu(_)) {
                                ActionResult::Action(Action::ShowSubmenu(
                                    self.state.id.clone(),
                                    self.active_index,
                                ))
                            } else if let Some(action) = item.action.clone() {
                                self.hide();
                                ActionResult::Action(action)
                            } else {
                                ActionResult::Handled
                            }
                        } else {
                            ActionResult::Handled
                        }
                    } else {
                        ActionResult::Handled
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    if self.parent.is_some() {
                        ActionResult::Action(Action::ShowParentMenu(self.parent.clone().unwrap()))
                    } else {
                        ActionResult::Handled
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if let Some(item) = self.get_active_item() {
                        if matches!(item.item_type, MenuItemType::Submenu(_)) {
                            ActionResult::Action(Action::ShowSubmenu(
                                self.state.id.clone(),
                                self.active_index,
                            ))
                        } else {
                            ActionResult::Handled
                        }
                    } else {
                        ActionResult::Handled
                    }
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
