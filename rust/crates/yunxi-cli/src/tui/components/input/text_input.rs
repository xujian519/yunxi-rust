use crate::tui::components::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Debug, Clone)]
pub struct TextInputStyle {
    pub bg_color: Color,
    pub fg_color: Color,
    pub placeholder_color: Color,
    pub cursor_color: Color,
    pub border: bool,
    pub border_color: Color,
    pub border_focus_color: Color,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            bg_color: Color::Rgb(26, 35, 50),
            fg_color: Color::Rgb(232, 232, 237),
            placeholder_color: Color::Rgb(106, 106, 128),
            cursor_color: Color::Rgb(139, 176, 240),
            border: true,
            border_color: Color::Rgb(42, 42, 58),
            border_focus_color: Color::Rgb(139, 176, 240),
        }
    }
}

pub struct TextInput {
    state: ComponentState,
    value: String,
    placeholder: String,
    cursor_position: usize,
    max_length: Option<usize>,
    multiline: bool,
    masked: bool,
    mask_char: char,
    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn Fn(&str) -> ActionResult + Send + Sync>>,
    #[allow(clippy::type_complexity)]
    on_submit: Option<Box<dyn Fn(&str) -> ActionResult + Send + Sync>>,
    style: TextInputStyle,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("text_input")),
            value: String::new(),
            placeholder: String::new(),
            cursor_position: 0,
            max_length: None,
            multiline: false,
            masked: false,
            mask_char: '*',
            on_change: None,
            on_submit: None,
            style: TextInputStyle::default(),
        }
    }

    pub fn with_value(mut self, value: String) -> Self {
        self.value = value;
        self.cursor_position = self.value.chars().count();
        self
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn with_multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    pub fn with_masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn with_mask_char(mut self, mask_char: char) -> Self {
        self.mask_char = mask_char;
        self
    }

    pub fn with_on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> ActionResult + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn with_on_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) -> ActionResult + Send + Sync + 'static,
    {
        self.on_submit = Some(Box::new(callback));
        self
    }

    pub fn with_style(mut self, style: TextInputStyle) -> Self {
        self.style = style;
        self
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
        self.cursor_position = self.value.chars().count();
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    fn insert_char(&mut self, c: char) {
        if let Some(max_len) = self.max_length {
            if self.value.chars().count() >= max_len {
                return;
            }
        }

        let mut chars: Vec<char> = self.value.chars().collect();
        chars.insert(self.cursor_position, c);
        self.value = chars.into_iter().collect();
        self.cursor_position += 1;

        self.trigger_change();
    }

    fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let mut chars: Vec<char> = self.value.chars().collect();
            chars.remove(self.cursor_position - 1);
            self.value = chars.into_iter().collect();
            self.cursor_position -= 1;

            self.trigger_change();
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        let char_count = self.value.chars().count();
        if self.cursor_position < char_count {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.value.chars().count();
    }

    fn trigger_change(&mut self) {
        if let Some(ref callback) = self.on_change {
            callback(&self.value);
        }
    }

    fn get_display_text(&self) -> String {
        if self.masked {
            "*".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }

    fn get_placeholder_text(&self) -> &str {
        if self.value.is_empty() {
            &self.placeholder
        } else {
            ""
        }
    }
}

impl Component for TextInput {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let display_text = self.get_display_text();
        let placeholder_text = self.get_placeholder_text();

        let border_color = if self.state.focused {
            self.style.border_focus_color
        } else {
            self.style.border_color
        };

        let text_content = if display_text.is_empty() && !placeholder_text.is_empty() {
            Line::from(Span::styled(
                placeholder_text,
                Style::default().fg(self.style.placeholder_color),
            ))
        } else {
            Line::from(display_text.as_str())
        };

        let widget = if self.style.border {
            Paragraph::new(text_content).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(self.style.bg_color)),
            )
        } else {
            Paragraph::new(text_content).style(
                Style::default()
                    .bg(self.style.bg_color)
                    .fg(self.style.fg_color),
            )
        };

        widget.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible || self.state.disabled {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key_event)) => match key_event.code {
                KeyCode::Char(c) => {
                    self.insert_char(c);
                    ActionResult::Handled
                }
                KeyCode::Backspace => {
                    self.delete_char();
                    ActionResult::Handled
                }
                KeyCode::Left => {
                    self.move_cursor_left();
                    ActionResult::Handled
                }
                KeyCode::Right => {
                    self.move_cursor_right();
                    ActionResult::Handled
                }
                KeyCode::Home => {
                    self.move_cursor_to_start();
                    ActionResult::Handled
                }
                KeyCode::End => {
                    self.move_cursor_to_end();
                    ActionResult::Handled
                }
                KeyCode::Enter => {
                    if let Some(ref callback) = self.on_submit {
                        return callback(&self.value);
                    }
                    ActionResult::Ignored
                }
                _ => ActionResult::Ignored,
            },
            Event::Input(InputEvent::Paste(text)) => {
                for c in text.chars() {
                    self.insert_char(c);
                }
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
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}
