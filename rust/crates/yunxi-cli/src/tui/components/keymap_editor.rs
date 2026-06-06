use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum KeymapConflictResolution {
    Override,
    Cancel,
    Rename(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyBinding {
    pub action: String,
    pub key_sequence: String,
    pub description: String,
    pub editable: bool,
}

impl KeyBinding {
    pub fn new(action: impl Into<String>, key_sequence: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            key_sequence: key_sequence.into(),
            description: String::new(),
            editable: true,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording(Vec<crossterm::event::KeyEvent>),
    Conflict {
        existing_binding: KeyBinding,
        new_key_sequence: String,
    },
}

pub struct KeymapEditor {
    state: ComponentState,
    bindings: Vec<KeyBinding>,
    key_index: KeyMapIndex,
    selected_index: usize,
    recording_state: RecordingState,
    filter: String,
    show_help: bool,
}

type KeyMapIndex = HashMap<String, usize>;

impl KeymapEditor {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("keymap_editor")),
            bindings: Vec::new(),
            key_index: HashMap::new(),
            selected_index: 0,
            recording_state: RecordingState::Idle,
            filter: String::new(),
            show_help: true,
        }
    }

    pub fn with_bindings(mut self, bindings: Vec<KeyBinding>) -> Self {
        self.bindings = bindings;
        self.rebuild_index();
        self
    }

    pub fn with_show_help(mut self, show: bool) -> Self {
        self.show_help = show;
        self
    }

    pub fn add_binding(&mut self, binding: KeyBinding) {
        if binding.editable {
            self.key_index
                .insert(binding.key_sequence.clone(), self.bindings.len());
        }
        self.bindings.push(binding);
    }

    pub fn remove_binding(&mut self, index: usize) {
        if index < self.bindings.len() {
            let binding = self.bindings.remove(index);
            if binding.editable {
                self.key_index.remove(&binding.key_sequence);
            }
            self.rebuild_index();
            if self.selected_index >= self.bindings.len() && !self.bindings.is_empty() {
                self.selected_index = self.bindings.len() - 1;
            }
        }
    }

    pub fn update_binding(&mut self, index: usize, key_sequence: String) {
        if index < self.bindings.len() {
            let binding = &mut self.bindings[index];
            if binding.editable {
                self.key_index.remove(&binding.key_sequence);
                binding.key_sequence = key_sequence.clone();
                self.key_index.insert(key_sequence, index);
            }
        }
    }

    pub fn reset_to_defaults(&mut self, defaults: Vec<KeyBinding>) {
        self.bindings = defaults;
        self.rebuild_index();
        self.selected_index = 0;
    }

    pub fn start_recording(&mut self) {
        if self.selected_index < self.bindings.len() && self.bindings[self.selected_index].editable
        {
            self.recording_state = RecordingState::Recording(Vec::new());
        }
    }

    pub fn stop_recording(&mut self) {
        self.recording_state = RecordingState::Idle;
    }

    pub fn capture_keypress(&mut self, key: KeyEvent) -> Option<KeymapConflictResolution> {
        match &mut self.recording_state {
            RecordingState::Recording(keys) => {
                keys.push(key);
                let key_sequence = Self::format_key_sequence(keys);

                if let Some(&existing_index) = self.key_index.get(&key_sequence) {
                    if existing_index != self.selected_index {
                        let existing_binding = self.bindings[existing_index].clone();
                        self.recording_state = RecordingState::Conflict {
                            existing_binding,
                            new_key_sequence: key_sequence,
                        };
                        return None;
                    }
                }

                self.update_binding(self.selected_index, key_sequence);
                self.recording_state = RecordingState::Idle;
                None
            }
            RecordingState::Conflict { .. } => None,
            RecordingState::Idle => None,
        }
    }

    pub fn resolve_conflict(&mut self, resolution: KeymapConflictResolution) {
        if let RecordingState::Conflict {
            existing_binding: _,
            new_key_sequence,
        } = std::mem::replace(&mut self.recording_state, RecordingState::Idle)
        {
            match resolution {
                KeymapConflictResolution::Override => {
                    self.update_binding(self.selected_index, new_key_sequence);
                }
                KeymapConflictResolution::Cancel => {}
                KeymapConflictResolution::Rename(new_name) => {
                    self.update_binding(self.selected_index, new_name);
                }
            }
        }
    }

    pub fn get_filtered_bindings(&self) -> Vec<&KeyBinding> {
        if self.filter.is_empty() {
            self.bindings.iter().collect()
        } else {
            self.bindings
                .iter()
                .filter(|b| {
                    b.action
                        .to_lowercase()
                        .contains(&self.filter.to_lowercase())
                        || b.key_sequence
                            .to_lowercase()
                            .contains(&self.filter.to_lowercase())
                        || b.description
                            .to_lowercase()
                            .contains(&self.filter.to_lowercase())
                })
                .collect()
        }
    }

    fn rebuild_index(&mut self) {
        self.key_index.clear();
        for (i, binding) in self.bindings.iter().enumerate() {
            if binding.editable {
                self.key_index.insert(binding.key_sequence.clone(), i);
            }
        }
    }

    fn format_key_sequence(keys: &[KeyEvent]) -> String {
        keys.iter()
            .map(Self::format_key)
            .collect::<Vec<_>>()
            .join("+")
    }

    fn format_key(key: &KeyEvent) -> String {
        let mut parts: Vec<String> = Vec::new();

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt".to_string());
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift".to_string());
        }

        match key.code {
            KeyCode::Char(c) => parts.push(c.to_uppercase().to_string()),
            KeyCode::F(n) => parts.push(format!("F{}", n)),
            KeyCode::Enter => parts.push("Enter".to_string()),
            KeyCode::Esc => parts.push("Esc".to_string()),
            KeyCode::Backspace => parts.push("Backspace".to_string()),
            KeyCode::Tab => parts.push("Tab".to_string()),
            KeyCode::Delete => parts.push("Delete".to_string()),
            KeyCode::Insert => parts.push("Insert".to_string()),
            KeyCode::Home => parts.push("Home".to_string()),
            KeyCode::End => parts.push("End".to_string()),
            KeyCode::PageUp => parts.push("PageUp".to_string()),
            KeyCode::PageDown => parts.push("PageDown".to_string()),
            KeyCode::Up => parts.push("↑".to_string()),
            KeyCode::Down => parts.push("↓".to_string()),
            KeyCode::Left => parts.push("←".to_string()),
            KeyCode::Right => parts.push("→".to_string()),
            _ => parts.push(format!("{:?}", key.code)),
        }

        parts.join("+")
    }

    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> ActionResult {
        match &self.recording_state {
            RecordingState::Recording(_) => {
                let event = KeyEvent::new(key, modifiers);
                self.capture_keypress(event);
                ActionResult::Handled
            }
            RecordingState::Conflict { .. } => match key {
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    self.resolve_conflict(KeymapConflictResolution::Override);
                    ActionResult::Handled
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.resolve_conflict(KeymapConflictResolution::Cancel);
                    ActionResult::Handled
                }
                KeyCode::Esc => {
                    self.resolve_conflict(KeymapConflictResolution::Cancel);
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
            RecordingState::Idle => match key {
                KeyCode::Esc => ActionResult::Action(Action::Close),
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.start_recording();
                    ActionResult::Handled
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    if self.selected_index < self.bindings.len() {
                        self.remove_binding(self.selected_index);
                    }
                    ActionResult::Handled
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    let new_binding = KeyBinding::new("new_action", "未设置").with_editable(true);
                    self.add_binding(new_binding);
                    self.selected_index = self.bindings.len() - 1;
                    ActionResult::Handled
                }
                KeyCode::Char('x') | KeyCode::Char('X') => {
                    ActionResult::Action(Action::Custom("reset_keymap".to_string()))
                }
                KeyCode::Char('/') => {
                    self.filter.clear();
                    ActionResult::Handled
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let filtered = self.get_filtered_bindings();
                    if !filtered.is_empty() {
                        self.selected_index = (self.selected_index + 1) % filtered.len();
                    }
                    ActionResult::Handled
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let filtered = self.get_filtered_bindings();
                    if !filtered.is_empty() {
                        self.selected_index = if self.selected_index == 0 {
                            filtered.len() - 1
                        } else {
                            self.selected_index - 1
                        };
                    }
                    ActionResult::Handled
                }
                KeyCode::Char(c)
                    if self.filter.len() < 20
                        && !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    self.filter.push(c);
                    self.selected_index = 0;
                    ActionResult::Handled
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.selected_index = 0;
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
        }
    }
}

impl Default for KeymapEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for KeymapEditor {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        Clear.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        let header_style = Style::default()
            .fg(Color::Rgb(139, 176, 240))
            .add_modifier(Modifier::BOLD);
        let border_style = Style::default().fg(Color::Rgb(100, 100, 100));

        let title = if !self.filter.is_empty() {
            Line::from(vec![
                Span::styled("快捷键编辑器", header_style),
                Span::raw(" - "),
                Span::raw(format!("过滤: {}", self.filter)),
            ])
        } else {
            Line::from(vec![
                Span::styled("快捷键编辑器", header_style),
                Span::raw(" - "),
                Span::raw("所有快捷键"),
            ])
        };

        let header_widget = Paragraph::new(title).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        header_widget.render(chunks[0], buf);

        let filtered_bindings = self.get_filtered_bindings();
        let items: Vec<ListItem> = filtered_bindings
            .iter()
            .enumerate()
            .map(|(i, binding)| {
                let is_selected = i == self.selected_index;
                let item_style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(50, 60, 80))
                        .add_modifier(Modifier::REVERSED)
                } else if binding.editable {
                    Style::default()
                } else {
                    Style::default().add_modifier(Modifier::DIM)
                };

                let key_style = Style::default()
                    .fg(Color::Rgb(139, 176, 240))
                    .add_modifier(Modifier::BOLD);

                let text = Line::from(vec![
                    Span::styled(format!("{:30}", binding.action), item_style),
                    Span::raw("  "),
                    Span::styled(format!("{:15}", binding.key_sequence), key_style),
                    Span::raw("  "),
                    Span::styled(&binding.description, item_style),
                ]);

                ListItem::new(text)
            })
            .collect();

        let list_widget = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        list_widget.render(chunks[1], buf);

        if self.show_help {
            let help_style = Style::default().fg(Color::Rgb(150, 150, 150));
            let help_text = if matches!(self.recording_state, RecordingState::Recording(_)) {
                "录制中... 按下按键组合，ESC 取消"
            } else if matches!(self.recording_state, RecordingState::Conflict { .. }) {
                "冲突检测: [O]覆盖 [C]取消 [ESC]返回"
            } else {
                "[↑/↓]选择 [R]录制 [D]删除 [A]添加 [X]重置 [/]清除过滤 [ESC]关闭"
            };

            let help_widget = Paragraph::new(Line::from(Span::styled(help_text, help_style)))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            help_widget.render(chunks[2], buf);
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(KeyEvent {
                code,
                modifiers,
                kind: _,
                state: _,
            })) => self.handle_key_event(*code, *modifiers),
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

    #[test]
    fn test_keymap_editor_creation() {
        let editor = KeymapEditor::new();
        assert!(editor.bindings.is_empty());
        assert_eq!(editor.selected_index, 0);
        assert!(matches!(editor.recording_state, RecordingState::Idle));
    }

    #[test]
    fn test_keymap_editor_with_bindings() {
        let bindings = vec![
            KeyBinding::new("quit", "Ctrl+Q"),
            KeyBinding::new("save", "Ctrl+S"),
        ];
        let editor = KeymapEditor::new().with_bindings(bindings);
        assert_eq!(editor.bindings.len(), 2);
    }

    #[test]
    fn test_key_binding_creation() {
        let binding = KeyBinding::new("quit", "Ctrl+Q");
        assert_eq!(binding.action, "quit");
        assert_eq!(binding.key_sequence, "Ctrl+Q");
        assert!(binding.editable);
        assert!(binding.description.is_empty());
    }

    #[test]
    fn test_key_binding_with_description() {
        let binding = KeyBinding::new("quit", "Ctrl+Q").with_description("退出应用");
        assert_eq!(binding.description, "退出应用");
    }

    #[test]
    fn test_key_binding_with_editable() {
        let binding = KeyBinding::new("system", "F1").with_editable(false);
        assert!(!binding.editable);
    }

    #[test]
    fn test_add_binding() {
        let mut editor = KeymapEditor::new();
        let binding = KeyBinding::new("quit", "Ctrl+Q");
        editor.add_binding(binding);
        assert_eq!(editor.bindings.len(), 1);
    }

    #[test]
    fn test_remove_binding() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.add_binding(KeyBinding::new("save", "Ctrl+S"));
        editor.remove_binding(0);
        assert_eq!(editor.bindings.len(), 1);
        assert_eq!(editor.bindings[0].action, "save");
    }

    #[test]
    fn test_update_binding() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.update_binding(0, "Ctrl+X".to_string());
        assert_eq!(editor.bindings[0].key_sequence, "Ctrl+X");
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("custom", "Ctrl+C"));

        let defaults = vec![
            KeyBinding::new("quit", "Ctrl+Q"),
            KeyBinding::new("save", "Ctrl+S"),
        ];
        editor.reset_to_defaults(defaults);

        assert_eq!(editor.bindings.len(), 2);
        assert_eq!(editor.bindings[0].action, "quit");
        assert_eq!(editor.bindings[1].action, "save");
    }

    #[test]
    fn test_start_recording() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.selected_index = 0;
        editor.start_recording();
        assert!(matches!(
            editor.recording_state,
            RecordingState::Recording(_)
        ));
    }

    #[test]
    fn test_stop_recording() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.selected_index = 0;
        editor.start_recording();
        editor.stop_recording();
        assert!(matches!(editor.recording_state, RecordingState::Idle));
    }

    #[test]
    fn test_capture_keypress() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.selected_index = 0;
        editor.start_recording();

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
        editor.capture_keypress(key);

        assert!(matches!(editor.recording_state, RecordingState::Idle));
    }

    #[test]
    fn test_format_key_sequence() {
        let keys = vec![
            KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT),
        ];
        let sequence = KeymapEditor::format_key_sequence(&keys);
        assert!(sequence.contains("Ctrl"));
        assert!(sequence.contains("Alt"));
    }

    #[test]
    fn test_filter_bindings() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q").with_description("退出应用"));
        editor.add_binding(KeyBinding::new("save", "Ctrl+S").with_description("保存文件"));
        editor.filter = "quit".to_string();

        let filtered = editor.get_filtered_bindings();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].action, "quit");
    }

    #[test]
    fn test_handle_navigation_keys() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.add_binding(KeyBinding::new("save", "Ctrl+S"));

        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::NONE,
        ))));
        assert_eq!(editor.selected_index, 1);

        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE,
        ))));
        assert_eq!(editor.selected_index, 0);
    }

    #[test]
    fn test_handle_add_key() {
        let mut editor = KeymapEditor::new();
        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        ))));
        assert_eq!(editor.bindings.len(), 1);
        assert_eq!(editor.bindings[0].action, "new_action");
    }

    #[test]
    fn test_handle_delete_key() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('d'),
            KeyModifiers::NONE,
        ))));
        assert!(editor.bindings.is_empty());
    }

    #[test]
    fn test_resolve_conflict_override() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));
        editor.add_binding(KeyBinding::new("save", "Ctrl+S"));
        editor.selected_index = 1;

        editor.recording_state = RecordingState::Conflict {
            existing_binding: KeyBinding::new("quit", "Ctrl+Q"),
            new_key_sequence: "Ctrl+Q".to_string(),
        };

        editor.resolve_conflict(KeymapConflictResolution::Override);
        assert_eq!(editor.bindings[1].key_sequence, "Ctrl+Q");
    }

    #[test]
    fn test_with_show_help() {
        let editor = KeymapEditor::new().with_show_help(false);
        assert!(!editor.show_help);
    }

    #[test]
    fn test_keymap_editor_id_generation() {
        let editor = KeymapEditor::new();
        assert!(editor.get_state().id.starts_with("keymap_editor_"));
    }

    #[test]
    fn test_keymap_editor_state_update() {
        let mut editor = KeymapEditor::new();
        editor.on_focus(true);
        assert!(editor.state.focused);

        let area = Rect::new(10, 10, 80, 20);
        editor.on_resize(area);
        assert_eq!(editor.state.bounds, area);
    }

    #[test]
    fn test_filter_input() {
        let mut editor = KeymapEditor::new();
        editor.add_binding(KeyBinding::new("quit", "Ctrl+Q"));

        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE,
        ))));
        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('u'),
            KeyModifiers::NONE,
        ))));
        assert_eq!(editor.filter, "qu");
    }

    #[test]
    fn test_filter_backspace() {
        let mut editor = KeymapEditor::new();
        editor.filter = "quit".to_string();
        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Backspace,
            KeyModifiers::NONE,
        ))));
        assert_eq!(editor.filter, "qui");
    }

    #[test]
    fn test_clear_filter() {
        let mut editor = KeymapEditor::new();
        editor.filter = "quit".to_string();
        editor.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('/'),
            KeyModifiers::NONE,
        ))));
        assert!(editor.filter.is_empty());
    }
}
