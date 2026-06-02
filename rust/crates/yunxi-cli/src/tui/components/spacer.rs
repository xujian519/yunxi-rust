use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

pub struct Spacer {
    state: ComponentState,
}

impl Spacer {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("spacer")),
        }
    }
}

impl Default for Spacer {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Spacer {
    fn render(&self, _area: Rect, _buf: &mut Buffer) {}

    fn handle_event(&mut self, _event: &Event) -> ActionResult {
        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }
}
