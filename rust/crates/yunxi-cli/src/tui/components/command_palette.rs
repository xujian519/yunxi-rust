use crate::tui::components::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crate::tui::keymap::{Command, CommandRegistry};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct CommandPalette {
    state: ComponentState,
    registry: CommandRegistry,
    query: String,
    filtered_commands: Vec<Command>,
    selected_index: usize,
    visible: bool,
}

impl CommandPalette {
    pub fn new() -> Self {
        let state = ComponentState::new(generate_component_id("command_palette"));
        let registry = CommandRegistry::new();
        let commands = registry.list().into_iter().cloned().collect();

        Self {
            state,
            registry,
            query: String::new(),
            filtered_commands: commands,
            selected_index: 0,
            visible: false,
        }
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.query.clear();
            self.filtered_commands = self.registry.list().into_iter().cloned().collect();
            self.selected_index = 0;
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn fuzzy_match(&self, pattern: &str, text: &str) -> (bool, i32) {
        if pattern.is_empty() {
            return (true, 0);
        }

        let mut matches = 0;
        let mut pattern_chars = pattern.chars();
        let mut text_chars = text.chars();

        for p_char in pattern_chars {
            let mut found = false;
            for t_char in text_chars.by_ref() {
                if p_char.eq_ignore_ascii_case(&t_char) {
                    matches += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                return (false, -1);
            }
        }

        (true, matches)
    }

    fn filter_commands(&mut self) {
        let pattern = self.query.to_lowercase();
        let mut scored: Vec<(Command, i32)> = self
            .registry
            .list()
            .into_iter()
            .filter_map(|cmd| {
                let (matches, score) = self.fuzzy_match(&pattern, &cmd.name.to_lowercase());
                if matches {
                    Some((cmd.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));

        self.filtered_commands = scored.into_iter().map(|(cmd, _)| cmd).collect();
        self.selected_index = 0;
    }

    fn select_next(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_commands.len();
        }
    }

    fn select_previous(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_commands.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    fn execute_selected(&self) -> Option<ActionResult> {
        if let Some(cmd) = self.filtered_commands.get(self.selected_index) {
            let actions = cmd.execute();
            if let Some(action) = actions.first() {
                return Some(ActionResult::Action(action.clone()));
            }
        }
        None
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for CommandPalette {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.visible {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let input_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let input_text = Text::from(vec![Line::from(vec![
            Span::styled(">", input_style),
            Span::raw(" "),
            Span::raw(&self.query),
        ])]);

        let input =
            Paragraph::new(input_text).block(Block::default().borders(Borders::ALL).title("命令"));
        input.render(chunks[0], buf);

        let list_height = chunks[1].height.saturating_sub(2) as usize;
        let start = self.selected_index.saturating_sub(list_height / 2);
        let end = (start + list_height).min(self.filtered_commands.len());

        let list_items: Vec<Line> = self.filtered_commands[start..end]
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let global_index = start + i;
                let style = if global_index == self.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                Line::from(vec![
                    Span::styled(&cmd.name, style),
                    Span::raw(" - "),
                    Span::styled(&cmd.description, Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect();

        let list = Paragraph::new(list_items).block(Block::default().borders(Borders::ALL));
        list.render(chunks[1], buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(KeyEvent {
                code, modifiers, ..
            })) => match (*code, *modifiers) {
                (KeyCode::Esc, _) => {
                    self.visible = false;
                    ActionResult::Action(Action::HideCommandPalette)
                }
                (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                    self.visible = false;
                    ActionResult::Action(Action::HideCommandPalette)
                }
                (KeyCode::Char(c), _) => {
                    self.query.push(c);
                    self.filter_commands();
                    ActionResult::Handled
                }
                (KeyCode::Backspace, _) => {
                    self.query.pop();
                    self.filter_commands();
                    ActionResult::Handled
                }
                (KeyCode::Down, _) => {
                    self.select_next();
                    ActionResult::Handled
                }
                (KeyCode::Up, _) => {
                    self.select_previous();
                    ActionResult::Handled
                }
                (KeyCode::Enter, _) => {
                    self.visible = false;
                    if let Some(result) = self.execute_selected() {
                        result
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        let palette = CommandPalette::new();

        assert_eq!(palette.fuzzy_match("", "hello"), (true, 0));
        assert_eq!(palette.fuzzy_match("h", "hello"), (true, 1));
        assert_eq!(palette.fuzzy_match("hl", "hello"), (true, 2));
        assert_eq!(palette.fuzzy_match("x", "hello"), (false, -1));
    }

    #[test]
    fn test_filter_commands() {
        let mut palette = CommandPalette::new();
        palette.query = "help".to_string();
        palette.filter_commands();
        assert!(!palette.filtered_commands.is_empty());
        assert!(palette.filtered_commands[0]
            .name
            .to_lowercase()
            .contains("help"));
    }

    #[test]
    fn test_toggle_visibility() {
        let mut palette = CommandPalette::new();
        assert!(!palette.is_visible());

        palette.toggle();
        assert!(palette.is_visible());

        palette.toggle();
        assert!(!palette.is_visible());
    }
}
