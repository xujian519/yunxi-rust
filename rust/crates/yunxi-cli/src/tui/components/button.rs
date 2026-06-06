use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::Action;
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct Button {
    state: ComponentState,
    text: String,
    on_click: Option<Box<dyn Fn() -> ActionResult + Send + Sync>>,
    style: Box<ButtonStyle>,
}

#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub normal_bg: Color,
    pub normal_fg: Color,
    pub focused_bg: Color,
    pub focused_fg: Color,
    pub disabled_bg: Color,
    pub disabled_fg: Color,
    pub border: bool,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            focused_bg: Color::Rgb(139, 176, 240),
            focused_fg: Color::Rgb(13, 13, 18),
            disabled_bg: Color::Rgb(22, 22, 30),
            disabled_fg: Color::Rgb(106, 106, 128),
            border: false,
        }
    }
}

impl Button {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("button")),
            text: text.into(),
            on_click: None,
            style: Box::new(ButtonStyle::default()),
        }
    }

    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> ActionResult + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(callback));
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.state.id = id.into();
        self
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.state.disabled = disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.state.focused
    }

    pub fn get_style(&self) -> &ButtonStyle {
        self.style.as_ref()
    }
}

impl Component for Button {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let text_width = self.text.len() as u16;
        let button_width = area.width.max(text_width + 2);
        let button_height = area.height.max(3);

        let button_area = Rect {
            x: area.x,
            y: area.y,
            width: button_width.min(area.width),
            height: button_height.min(area.height),
        };

        let (bg_color, fg_color) = if self.state.disabled {
            (self.style.disabled_bg, self.style.disabled_fg)
        } else if self.state.focused {
            (self.style.focused_bg, self.style.focused_fg)
        } else {
            (self.style.normal_bg, self.style.normal_fg)
        };

        let mut style = Style::default().bg(bg_color).fg(fg_color);
        if self.state.focused {
            style = style.add_modifier(Modifier::BOLD);
        }

        let widget = if self.style.border {
            Paragraph::new(self.text.as_str())
                .block(Block::default().borders(Borders::ALL).style(style))
        } else {
            Paragraph::new(self.text.as_str()).style(style)
        };

        widget.render(button_area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key))
                if key.code == KeyCode::Enter
                    && key.modifiers == KeyModifiers::NONE
                    && self.state.focused =>
            {
                if let Some(ref callback) = self.on_click {
                    return callback();
                }
                return ActionResult::Action(Action::Navigate("/home".to_string()));
            }
            _ => {}
        }

        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }
}
