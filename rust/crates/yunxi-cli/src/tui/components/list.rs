use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List as RatatuiList, ListItem, ListState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Single,
    Multiple,
}

pub struct List<T: Clone + ToString + Send + Sync> {
    state: ComponentState,
    items: Vec<ListItemData<T>>,
    selected_indices: Vec<usize>,
    focused_index: usize,
    scroll_offset: usize,
    page_size: usize,
    sorted: bool,
    sort_ascending: bool,
    selection_mode: SelectionMode,
    on_select: Option<Box<dyn Fn(usize, &T) -> ActionResult + Send + Sync>>,
    on_double_click: Option<Box<dyn Fn(usize, &T) -> ActionResult + Send + Sync>>,
    style: ListStyle,
}

#[derive(Debug, Clone)]
pub struct ListItemData<T: Clone + ToString + Send + Sync> {
    pub value: T,
    pub text: String,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub struct ListStyle {
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
    pub focused_bg: Color,
    pub focused_fg: Color,
    pub header_bg: Color,
    pub header_fg: Color,
    pub border: bool,
    pub show_header: bool,
    pub header_text: String,
}

impl Default for ListStyle {
    fn default() -> Self {
        Self {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            selected_bg: Color::Rgb(68, 138, 255),
            selected_fg: Color::Rgb(13, 13, 18),
            focused_bg: Color::Rgb(139, 176, 240),
            focused_fg: Color::Rgb(13, 13, 18),
            header_bg: Color::Rgb(36, 45, 60),
            header_fg: Color::Rgb(232, 232, 237),
            border: true,
            show_header: false,
            header_text: String::new(),
        }
    }
}

impl<T: Clone + ToString + Send + Sync> List<T> {
    pub fn new(items: Vec<T>) -> Self {
        let items = items
            .into_iter()
            .map(|value| ListItemData {
                text: value.to_string(),
                value,
                visible: true,
            })
            .collect();

        Self {
            state: ComponentState::new(generate_component_id("list")),
            items,
            selected_indices: Vec::new(),
            focused_index: 0,
            scroll_offset: 0,
            page_size: 10,
            sorted: false,
            sort_ascending: true,
            selection_mode: SelectionMode::Single,
            on_select: None,
            on_double_click: None,
            style: ListStyle::default(),
        }
    }

    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.items = items
            .into_iter()
            .map(|value| ListItemData {
                text: value.to_string(),
                value,
                visible: true,
            })
            .collect();
        self
    }

    pub fn with_selection_mode(mut self, selection_mode: SelectionMode) -> Self {
        self.selection_mode = selection_mode;
        self
    }

    pub fn with_multi_select(mut self, multi_select: bool) -> Self {
        self.selection_mode = if multi_select {
            SelectionMode::Multiple
        } else {
            SelectionMode::Single
        };
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn with_style(mut self, style: ListStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &T) -> ActionResult + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn with_on_double_click<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &T) -> ActionResult + Send + Sync + 'static,
    {
        self.on_double_click = Some(Box::new(callback));
        self
    }

    pub fn with_sorted(mut self, sorted: bool) -> Self {
        self.sorted = sorted;
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn get_selected_indices(&self) -> &[usize] {
        &self.selected_indices
    }

    pub fn get_selected_items(&self) -> Vec<&T> {
        self.selected_indices
            .iter()
            .filter_map(|&idx| self.items.get(idx).map(|item| &item.value))
            .collect()
    }

    pub fn get_focused_index(&self) -> usize {
        self.focused_index
    }

    pub fn get_items(&self) -> &[ListItemData<T>] {
        &self.items
    }

    pub fn get_selection_mode(&self) -> SelectionMode {
        self.selection_mode
    }

    pub fn set_selection_mode(&mut self, mode: SelectionMode) {
        self.selection_mode = mode;
        if mode == SelectionMode::Single && self.selected_indices.len() > 1 {
            self.selected_indices = vec![self.focused_index];
        }
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(ListItemData {
            text: item.to_string(),
            value: item,
            visible: true,
        });
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            self.selected_indices.retain(|&i| i != index);
            self.selected_indices.iter_mut().for_each(|i| {
                if *i > index {
                    *i -= 1
                }
            });
            if self.focused_index >= self.items.len() && !self.items.is_empty() {
                self.focused_index = self.items.len() - 1;
            }
        }
    }

    pub fn toggle_selection(&mut self) {
        match self.selection_mode {
            SelectionMode::Single => {
                self.selected_indices.clear();
                self.selected_indices.push(self.focused_index);
            }
            SelectionMode::Multiple => {
                if self.selected_indices.contains(&self.focused_index) {
                    self.selected_indices.retain(|&i| i != self.focused_index);
                } else {
                    self.selected_indices.push(self.focused_index);
                }
            }
        }
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        if self.selection_mode != SelectionMode::Multiple {
            return;
        }
        if start > end {
            self.select_range(end, start);
            return;
        }
        self.selected_indices.clear();
        for i in start..=end {
            if i < self.items.len() {
                self.selected_indices.push(i);
            }
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_indices.clear();
    }

    pub fn select_all(&mut self) {
        if self.selection_mode == SelectionMode::Multiple {
            self.selected_indices = (0..self.items.len()).collect();
        }
    }

    pub fn filter<F>(&mut self, predicate: F)
    where
        F: Fn(&T) -> bool,
    {
        for item in &mut self.items {
            item.visible = predicate(&item.value);
        }
    }

    pub fn sort(&mut self) {
        if self.sorted {
            self.items.sort_by(|a, b| {
                if self.sort_ascending {
                    a.text.cmp(&b.text)
                } else {
                    b.text.cmp(&a.text)
                }
            });
        }
    }

    pub fn toggle_sort_order(&mut self) {
        self.sort_ascending = !self.sort_ascending;
        self.sort();
    }
}

impl<T: Clone + ToString + Send + Sync> Component for List<T> {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let visible_items: Vec<ListItem> = self
            .items
            .iter()
            .filter(|item| item.visible)
            .enumerate()
            .map(|(idx, item)| {
                let style = if self.selected_indices.contains(&idx) {
                    Style::default()
                        .bg(self.style.selected_bg)
                        .fg(self.style.selected_fg)
                } else if idx == self.focused_index && self.state.focused {
                    Style::default()
                        .bg(self.style.focused_bg)
                        .fg(self.style.focused_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .bg(self.style.normal_bg)
                        .fg(self.style.normal_fg)
                };
                ListItem::new(item.text.clone()).style(style)
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(self.focused_index.saturating_sub(self.scroll_offset)));

        let list = RatatuiList::new(visible_items).block(
            Block::default()
                .borders(if self.style.border {
                    Borders::ALL
                } else {
                    Borders::NONE
                })
                .style(Style::default()),
        );

        list.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Down | KeyCode::Char('j') => {
                    let old_index = self.focused_index;
                    if self.focused_index < self.items.len().saturating_sub(1) {
                        self.focused_index += 1;
                        if self.focused_index >= self.scroll_offset + self.page_size {
                            self.scroll_offset += 1;
                        }
                    }
                    if key.modifiers.contains(KeyModifiers::SHIFT)
                        && self.selection_mode == SelectionMode::Multiple
                    {
                        self.select_range(old_index, self.focused_index);
                    }
                    ActionResult::Handled
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let old_index = self.focused_index;
                    if self.focused_index > 0 {
                        self.focused_index -= 1;
                        if self.focused_index < self.scroll_offset {
                            self.scroll_offset = self.scroll_offset.saturating_sub(1);
                        }
                    }
                    if key.modifiers.contains(KeyModifiers::SHIFT)
                        && self.selection_mode == SelectionMode::Multiple
                    {
                        self.select_range(old_index, self.focused_index);
                    }
                    ActionResult::Handled
                }
                KeyCode::PageDown => {
                    let page = self.page_size.min(self.items.len());
                    let new_index =
                        (self.focused_index + page).min(self.items.len().saturating_sub(1));
                    self.focused_index = new_index;
                    self.scroll_offset = self
                        .scroll_offset
                        .saturating_add(page)
                        .min(self.items.len().saturating_sub(self.page_size));
                    ActionResult::Handled
                }
                KeyCode::PageUp => {
                    let page = self.page_size.min(self.items.len());
                    let new_index = self.focused_index.saturating_sub(page);
                    self.focused_index = new_index;
                    self.scroll_offset = self.scroll_offset.saturating_sub(page);
                    ActionResult::Handled
                }
                KeyCode::Home => {
                    self.focused_index = 0;
                    self.scroll_offset = 0;
                    ActionResult::Handled
                }
                KeyCode::End => {
                    self.focused_index = self.items.len().saturating_sub(1);
                    self.scroll_offset = self.items.len().saturating_sub(self.page_size);
                    ActionResult::Handled
                }
                KeyCode::Enter => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        if let Some(ref callback) = self.on_double_click {
                            if let Some(item) = self.items.get(self.focused_index) {
                                return callback(self.focused_index, &item.value);
                            }
                        }
                    } else if let Some(ref callback) = self.on_select {
                        if let Some(item) = self.items.get(self.focused_index) {
                            return callback(self.focused_index, &item.value);
                        }
                    }
                    ActionResult::Ignored
                }
                KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.select_all();
                    ActionResult::Handled
                }
                KeyCode::Char(' ') => {
                    self.toggle_selection();
                    ActionResult::Handled
                }
                KeyCode::Esc => {
                    self.clear_selection();
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

    fn on_resize(&mut self, area: Rect) {
        self.state.bounds = area;
        self.page_size = area.height as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_creation() {
        let list: List<String> = List::new(vec!["item1".to_string(), "item2".to_string()]);
        assert_eq!(list.get_items().len(), 2);
        assert_eq!(list.get_focused_index(), 0);
    }

    #[test]
    fn test_list_selection() {
        let mut list: List<String> = List::new(vec!["item1".to_string(), "item2".to_string()])
            .with_selection_mode(SelectionMode::Multiple);
        list.handle_event(&Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char(' '),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })));
        assert_eq!(list.get_selected_indices().len(), 1);
    }

    #[test]
    fn test_list_select_all() {
        let mut list: List<String> = List::new(vec!["item1".to_string(), "item2".to_string()])
            .with_selection_mode(SelectionMode::Multiple);
        list.select_all();
        assert_eq!(list.get_selected_indices().len(), 2);
    }

    #[test]
    fn test_list_select_range() {
        let mut list: List<String> = List::new(vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
            "item4".to_string(),
        ])
        .with_selection_mode(SelectionMode::Multiple);
        list.select_range(1, 3);
        assert_eq!(list.get_selected_indices(), vec![1, 2, 3]);
    }

    #[test]
    fn test_list_toggle_selection_single() {
        let mut list: List<String> = List::new(vec!["item1".to_string(), "item2".to_string()])
            .with_selection_mode(SelectionMode::Single);
        list.toggle_selection();
        assert_eq!(list.get_selected_indices(), vec![0]);
        list.focused_index = 1;
        list.toggle_selection();
        assert_eq!(list.get_selected_indices(), vec![1]);
    }

    #[test]
    fn test_list_toggle_selection_multiple() {
        let mut list: List<String> = List::new(vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ])
        .with_selection_mode(SelectionMode::Multiple);
        list.toggle_selection();
        assert_eq!(list.get_selected_indices(), vec![0]);
        list.focused_index = 1;
        list.toggle_selection();
        assert_eq!(list.get_selected_indices().len(), 2);
        list.focused_index = 0;
        list.toggle_selection();
        assert_eq!(list.get_selected_indices(), vec![1]);
    }

    #[test]
    fn test_list_clear_selection() {
        let mut list: List<String> = List::new(vec!["item1".to_string(), "item2".to_string()])
            .with_selection_mode(SelectionMode::Multiple);
        list.select_all();
        list.clear_selection();
        assert_eq!(list.get_selected_indices().len(), 0);
    }

    #[test]
    fn test_list_navigation() {
        let mut list: List<String> = List::new(vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ]);

        list.handle_event(&Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })));
        assert_eq!(list.get_focused_index(), 1);

        list.handle_event(&Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        })));
        assert_eq!(list.get_focused_index(), 0);
    }

    #[test]
    fn test_list_filter() {
        let mut list: List<String> = List::new(vec![
            "apple".to_string(),
            "banana".to_string(),
            "apricot".to_string(),
        ]);
        list.filter(|item| item.starts_with('a'));
        assert_eq!(list.get_items().iter().filter(|i| i.visible).count(), 2);
    }
}
