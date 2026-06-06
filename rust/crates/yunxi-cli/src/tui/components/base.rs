use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait Component: Send + Sync {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_event(&mut self, event: &Event) -> ActionResult;
    fn get_state(&self) -> ComponentState;

    fn on_mount(&mut self) {}
    fn on_unmount(&mut self) {}
    fn on_focus(&mut self, _focused: bool) {}
    fn on_resize(&mut self, _area: Rect) {}
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentState {
    pub id: String,
    pub visible: bool,
    pub focused: bool,
    pub disabled: bool,
    pub bounds: Rect,
}

impl ComponentState {
    pub fn new(id: String) -> Self {
        Self {
            id,
            visible: true,
            focused: false,
            disabled: false,
            bounds: Rect::default(),
        }
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub static COMPONENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn generate_component_id(prefix: &str) -> String {
    let id = COMPONENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}_{}", prefix, id)
}
