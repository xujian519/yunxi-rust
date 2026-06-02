use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

pub struct Label {
    state: ComponentState,
    text: String,
    color: Color,
    style: Style,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("label")),
            text: text.into(),
            color: Color::Rgb(232, 232, 237),
            style: Style::default(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl Component for Label {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let span = Span::styled(self.text.as_str(), self.style.fg(self.color));
        let paragraph = Paragraph::new(span);
        paragraph.render(area, buf);
    }

    fn handle_event(&mut self, _event: &Event) -> ActionResult {
        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }
}
