use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::Action;
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

pub struct Picker {
    state: ComponentState,
    items: Vec<String>,
    selected_index: usize,
    search_query: String,
    highlight_matches: bool,
    page_size: usize,
    filtered_items: Vec<(usize, String)>,
    list_state: ListState,
    on_select: Option<Box<dyn Fn(usize, &str) -> ActionResult + Send + Sync>>,
    on_cancel: Option<Box<dyn Fn() -> ActionResult + Send + Sync>>,
}

impl Picker {
    pub fn new(items: Vec<String>) -> Self {
        let mut picker = Self {
            state: ComponentState::new(generate_component_id("picker")),
            items,
            selected_index: 0,
            search_query: String::new(),
            highlight_matches: true,
            page_size: 10,
            filtered_items: Vec::new(),
            list_state: ListState::default(),
            on_select: None,
            on_cancel: None,
        };

        picker.update_filtered_items();
        picker
    }

    pub fn with_selected_index(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.items.len().saturating_sub(1));
        self.update_filtered_items();
        self
    }

    pub fn with_highlight_matches(mut self, highlight: bool) -> Self {
        self.highlight_matches = highlight;
        self
    }

    pub fn with_page_size(mut self, size: usize) -> Self {
        self.page_size = size.max(1);
        self
    }

    pub fn with_on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, &str) -> ActionResult + Send + Sync + 'static,
    {
        self.on_select = Some(Box::new(callback));
        self
    }

    pub fn with_on_cancel<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> ActionResult + Send + Sync + 'static,
    {
        self.on_cancel = Some(Box::new(callback));
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn get_selected_item(&self) -> Option<&str> {
        self.items.get(self.selected_index).map(|s| s.as_str())
    }

    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    fn update_filtered_items(&mut self) {
        self.filtered_items.clear();

        if self.search_query.is_empty() {
            self.filtered_items = self.items.iter().cloned().enumerate().collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_items = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.to_lowercase().contains(&query))
                .map(|(i, s)| (i, s.clone()))
                .collect();
        }

        self.list_state.select(Some(0));
    }

    fn get_selected_filtered_index(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn move_selection(&mut self, delta: i32) {
        let current = self.get_selected_filtered_index().unwrap_or(0);
        let count = self.filtered_items.len();

        if count == 0 {
            return;
        }

        let new_index = if delta > 0 {
            (current as i32 + delta).rem_euclid(count as i32) as usize
        } else {
            ((current as i32 + delta).rem_euclid(count as i32) + count as i32) as usize
        };

        self.list_state.select(Some(new_index));
    }

    fn confirm_selection(&mut self) -> ActionResult {
        if let Some(selected) = self.get_selected_filtered_index() {
            if let Some((original_index, item)) = self.filtered_items.get(selected) {
                self.selected_index = *original_index;
                if let Some(ref callback) = self.on_select {
                    return callback(*original_index, item);
                }
                return ActionResult::Action(Action::ExecuteCommand(format!("select {}", item)));
            }
        }
        ActionResult::Ignored
    }

    fn cancel(&mut self) -> ActionResult {
        if let Some(ref callback) = self.on_cancel {
            return callback();
        }
        ActionResult::Action(Action::HideDialog)
    }

    fn highlight_text(&self, text: &str) -> Text<'static> {
        if !self.highlight_matches || self.search_query.is_empty() {
            return Text::from(text.to_string());
        }

        let mut spans = Vec::new();
        let mut last_end = 0;
        let text_lower = text.to_lowercase();
        let query_lower = self.search_query.to_lowercase();

        while let Some(start) = text_lower[last_end..].find(&query_lower) {
            let start = last_end + start;
            let end = start + query_lower.len();

            if start > last_end {
                spans.push(Span::raw(text[last_end..start].to_string()));
            }

            spans.push(Span::styled(
                text[start..end].to_string(),
                Style::default()
                    .fg(Color::Rgb(255, 255, 0))
                    .add_modifier(Modifier::BOLD),
            ));

            last_end = end;
        }

        if last_end < text.len() {
            spans.push(Span::raw(text[last_end..].to_string()));
        }

        if spans.is_empty() {
            Text::from(text.to_string())
        } else {
            Text::from(Line::from(spans))
        }
    }

    fn render_search_input(&self, area: Rect, buf: &mut Buffer) {
        let input_text = if self.search_query.is_empty() {
            "Search..."
        } else {
            &self.search_query
        };

        let style = if self.state.focused {
            Style::default().fg(Color::Rgb(232, 232, 237))
        } else {
            Style::default().fg(Color::Rgb(106, 106, 128))
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .style(style)
            .border_style(Style::default().fg(Color::Rgb(139, 176, 240)));

        let paragraph = ratatui::widgets::Paragraph::new(input_text)
            .block(block)
            .style(Style::default().fg(Color::Rgb(232, 232, 237)));

        paragraph.render(area, buf);
    }
}

impl Component for Picker {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        self.render_search_input(chunks[0], buf);

        if self.filtered_items.is_empty() {
            let no_items = ratatui::widgets::Paragraph::new("No items found")
                .style(Style::default().fg(Color::Rgb(106, 106, 128)))
                .alignment(Alignment::Center);
            no_items.render(chunks[1], buf);
            return;
        }

        let items: Vec<ListItem> = self
            .filtered_items
            .iter()
            .map(|(_, text)| ListItem::new(self.highlight_text(text)))
            .collect();

        let mut list_state = self.list_state.clone();

        let border_style = if self.state.focused {
            Style::default().fg(Color::Rgb(139, 176, 240))
        } else {
            Style::default().fg(Color::Rgb(106, 106, 128))
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).style(border_style))
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(139, 92, 246))
                    .fg(Color::Rgb(255, 255, 255)),
            )
            .highlight_symbol("> ");

        let mut list_area = chunks[1];

        let scrollbar_area = Rect {
            x: list_area.right() - 1,
            y: list_area.top() + 1,
            width: 1,
            height: list_area.height.saturating_sub(2),
        };

        list_area.width = list_area.width.saturating_sub(1);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = ScrollbarState::new(self.filtered_items.len());
        if let Some(selected) = self.get_selected_filtered_index() {
            scrollbar_state = scrollbar_state.position(selected);
        }

        Widget::render(list, list_area, buf);
        StatefulWidget::render(scrollbar, scrollbar_area, buf, &mut scrollbar_state);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                    self.search_query.push(c);
                    self.update_filtered_items();
                    ActionResult::Handled
                }
                KeyCode::Backspace if key.modifiers == KeyModifiers::NONE => {
                    self.search_query.pop();
                    self.update_filtered_items();
                    ActionResult::Handled
                }
                KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                    self.move_selection(1);
                    ActionResult::Handled
                }
                KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                    self.move_selection(-1);
                    ActionResult::Handled
                }
                KeyCode::Enter if key.modifiers == KeyModifiers::NONE => self.confirm_selection(),
                KeyCode::Esc if key.modifiers == KeyModifiers::NONE => self.cancel(),
                KeyCode::PageDown if key.modifiers == KeyModifiers::NONE => {
                    self.move_selection(self.page_size as i32);
                    ActionResult::Handled
                }
                KeyCode::PageUp if key.modifiers == KeyModifiers::NONE => {
                    self.move_selection(-(self.page_size as i32));
                    ActionResult::Handled
                }
                KeyCode::Home if key.modifiers == KeyModifiers::NONE => {
                    self.list_state.select(Some(0));
                    ActionResult::Handled
                }
                KeyCode::End if key.modifiers == KeyModifiers::NONE => {
                    if !self.filtered_items.is_empty() {
                        self.list_state.select(Some(self.filtered_items.len() - 1));
                    }
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
            Event::Input(InputEvent::Resize(width, height)) => {
                self.state.bounds = Rect::new(0, 0, *width, *height);
                ActionResult::Handled
            }
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_picker() -> Picker {
        let items = vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
            "Date".to_string(),
            "Elderberry".to_string(),
        ];
        Picker::new(items)
    }

    #[test]
    fn test_picker_creation() {
        let picker = create_test_picker();

        assert_eq!(picker.items.len(), 5);
        assert_eq!(picker.get_selected_index(), 0);
        assert_eq!(picker.get_search_query(), "");
        assert!(picker.highlight_matches);
    }

    #[test]
    fn test_picker_with_options() {
        let picker = Picker::new(vec!["Item1".to_string(), "Item2".to_string()])
            .with_selected_index(1)
            .with_highlight_matches(false)
            .with_page_size(5)
            .with_id("custom_picker".to_string());

        assert_eq!(picker.get_selected_index(), 1);
        assert!(!picker.highlight_matches);
        assert_eq!(picker.page_size, 5);
        assert_eq!(picker.state.id, "custom_picker");
    }

    #[test]
    fn test_picker_get_selected_item() {
        let picker = create_test_picker();

        assert_eq!(picker.get_selected_item(), Some("Apple"));

        let picker = create_test_picker().with_selected_index(2);
        assert_eq!(picker.get_selected_item(), Some("Cherry"));
    }

    #[test]
    fn test_picker_search() {
        let mut picker = create_test_picker();

        picker.search_query = "an".to_string();
        picker.update_filtered_items();

        assert_eq!(picker.filtered_items.len(), 2);
        assert!(picker.filtered_items.iter().any(|(_, s)| s == "Banana"));
        assert!(picker.filtered_items.iter().any(|(_, s)| s == "Orange"));
    }

    #[test]
    fn test_picker_move_selection() {
        let mut picker = create_test_picker();

        picker.move_selection(1);
        assert_eq!(picker.get_selected_filtered_index(), Some(1));

        picker.move_selection(1);
        assert_eq!(picker.get_selected_filtered_index(), Some(2));

        picker.move_selection(-1);
        assert_eq!(picker.get_selected_filtered_index(), Some(1));
    }

    #[test]
    fn test_picker_move_selection_wrap() {
        let mut picker = create_test_picker();

        picker.move_selection(-1);
        assert_eq!(picker.get_selected_filtered_index(), Some(4));

        picker.move_selection(1);
        assert_eq!(picker.get_selected_filtered_index(), Some(0));
    }

    #[test]
    fn test_picker_handle_key_events() {
        let mut picker = create_test_picker();

        let char_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        let result = picker.handle_event(&char_event);
        assert!(matches!(result, ActionResult::Handled));
        assert_eq!(picker.search_query, "a");

        let down_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Down,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        let result = picker.handle_event(&down_event);
        assert!(matches!(result, ActionResult::Handled));
    }

    #[test]
    fn test_picker_handle_backspace() {
        let mut picker = create_test_picker();

        picker.search_query = "test".to_string();
        picker.update_filtered_items();

        let backspace_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        picker.handle_event(&backspace_event);
        assert_eq!(picker.search_query, "tes");
    }

    #[test]
    fn test_picker_handle_esc() {
        let mut picker = create_test_picker();

        let esc_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        let result = picker.handle_event(&esc_event);
        assert!(matches!(result, ActionResult::Action(_)));
    }

    #[test]
    fn test_picker_handle_enter() {
        let mut picker = create_test_picker();

        let enter_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        let result = picker.handle_event(&enter_event);
        assert!(matches!(result, ActionResult::Action(_)));
    }

    #[test]
    fn test_picker_highlight_text() {
        let mut picker = Picker::new(vec!["Apple".to_string()]);
        picker.search_query = "ap".to_string();

        let text = picker.highlight_text("Apple");
        let line = text.lines.first();
        assert!(line.is_some());
    }

    #[test]
    fn test_picker_empty_items() {
        let picker = Picker::new(vec![]);

        assert_eq!(picker.items.len(), 0);
        assert!(picker.filtered_items.is_empty());
    }

    #[test]
    fn test_picker_page_navigation() {
        let items: Vec<String> = (0..30).map(|i| format!("Item {}", i)).collect();
        let mut picker = Picker::new(items).with_page_size(10);

        let page_down = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::PageDown,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        picker.handle_event(&page_down);
        assert_eq!(picker.get_selected_filtered_index(), Some(10));

        let page_up = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::PageUp,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        picker.handle_event(&page_up);
        assert_eq!(picker.get_selected_filtered_index(), Some(0));
    }

    #[test]
    fn test_picker_home_end() {
        let mut picker = create_test_picker();

        picker.move_selection(4);

        let home_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Home,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        picker.handle_event(&home_event);
        assert_eq!(picker.get_selected_filtered_index(), Some(0));

        picker.move_selection(4);

        let end_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::End,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        picker.handle_event(&end_event);
        assert_eq!(picker.get_selected_filtered_index(), Some(4));
    }
}
