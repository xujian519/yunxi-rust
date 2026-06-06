use crate::tui::components::base::{generate_component_id, Component, ComponentState};
use crate::tui::components::button::{Button, ButtonStyle};
use crate::tui::components::input::text_input::TextInput;
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

pub struct Prompt {
    state: ComponentState,
    message: String,
    input: TextInput,
    confirm_button: Button,
    cancel_button: Button,
}

impl Prompt {
    pub fn new(message: String) -> Self {
        let input = TextInput::new()
            .with_placeholder("输入内容...".to_string())
            .with_max_length(100);

        let confirm_button = Button::new("确认").with_style(ButtonStyle {
            normal_bg: Color::Rgb(123, 200, 156),
            normal_fg: Color::Rgb(13, 13, 18),
            ..Default::default()
        });

        let cancel_button = Button::new("取消").with_style(ButtonStyle {
            normal_bg: Color::Rgb(232, 132, 124),
            normal_fg: Color::Rgb(13, 13, 18),
            ..Default::default()
        });

        Self {
            state: ComponentState::new(generate_component_id("prompt")),
            message,
            input,
            confirm_button,
            cancel_button,
        }
    }

    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.input = self.input.with_placeholder(placeholder);
        self
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.input = self.input.with_max_length(max_length);
        self
    }

    pub fn set_value(&mut self, value: String) {
        self.input.set_value(value);
    }

    pub fn get_value(&self) -> &str {
        self.input.get_value()
    }

    pub fn clear(&mut self) {
        self.input.clear();
    }
}

impl Component for Prompt {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let message_line = Line::from(Span::styled(
            format!("{} ", self.message),
            Style::default().fg(Color::Rgb(232, 232, 237)),
        ));
        let message_paragraph = Paragraph::new(message_line);
        let message_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: 1,
        };
        message_paragraph.render(message_area, buf);

        let input_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: 3,
        };
        self.input.render(input_area, buf);

        let button_width = 8;
        let button_height = 3;
        let button_gap = 2;

        let total_buttons_width = (button_width * 2) + button_gap;
        let start_x = area.x + (area.width - total_buttons_width) / 2;

        let confirm_button_area = Rect {
            x: start_x,
            y: area.y + 4,
            width: button_width,
            height: button_height,
        };

        let cancel_button_area = Rect {
            x: start_x + button_width + button_gap,
            y: area.y + 4,
            width: button_width,
            height: button_height,
        };

        self.confirm_button.render(confirm_button_area, buf);
        self.cancel_button.render(cancel_button_area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible || self.state.disabled {
            return ActionResult::Ignored;
        }

        let input_result = self.input.handle_event(event);
        if !matches!(input_result, ActionResult::Ignored) {
            return input_result;
        }

        let confirm_result = self.confirm_button.handle_event(event);
        if !matches!(confirm_result, ActionResult::Ignored) {
            return confirm_result;
        }

        let cancel_result = self.cancel_button.handle_event(event);
        if !matches!(cancel_result, ActionResult::Ignored) {
            return cancel_result;
        }

        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
        if focused {
            self.input.on_focus(true);
        }
    }
}
