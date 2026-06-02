use crate::tui::components::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::layout::{Alignment, Direction};

pub struct Flex {
    state: ComponentState,
    direction: Direction,
    align: Alignment,
    gap: u16,
    children: Vec<Box<dyn Component>>,
}

impl Flex {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("flex")),
            direction: Direction::Vertical,
            align: Alignment::Left,
            gap: 0,
            children: Vec::new(),
        }
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_alignment(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }

    pub fn with_gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    pub fn add_child(mut self, child: Box<dyn Component>) -> Self {
        self.children.push(child);
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn Component>>) -> Self {
        self.children = children;
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.state.id = id.into();
        self
    }
}

impl Component for Flex {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible || self.children.is_empty() {
            return;
        }

        let children_count = self.children.len();
        let total_gap = self.gap * (children_count.saturating_sub(1)) as u16;

        let available_space = match self.direction {
            Direction::Horizontal => area.width.saturating_sub(total_gap),
            Direction::Vertical => area.height.saturating_sub(total_gap),
        };

        let base_size = available_space / children_count as u16;
        let mut current_pos = match self.direction {
            Direction::Horizontal => area.x,
            Direction::Vertical => area.y,
        };

        for (_index, child) in self.children.iter().enumerate() {
            let child_size = base_size;

            let child_area = match self.direction {
                Direction::Horizontal => {
                    let x = current_pos;
                    let width = child_size;
                    current_pos += width + self.gap;
                    Rect {
                        x,
                        y: area.y,
                        width,
                        height: area.height,
                    }
                }
                Direction::Vertical => {
                    let y = current_pos;
                    let height = child_size;
                    current_pos += height + self.gap;
                    Rect {
                        x: area.x,
                        y,
                        width: area.width,
                        height,
                    }
                }
            };

            child.render(child_area, buf);
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible || self.state.disabled {
            return ActionResult::Ignored;
        }

        for child in &mut self.children {
            let result = child.handle_event(event);
            if !matches!(result, ActionResult::Ignored) {
                return result;
            }
        }

        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_mount(&mut self) {
        for child in &mut self.children {
            child.on_mount();
        }
    }

    fn on_unmount(&mut self) {
        for child in &mut self.children {
            child.on_unmount();
        }
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
        if focused && !self.children.is_empty() {
            self.children[0].on_focus(true);
        }
    }
}

impl Default for Flex {
    fn default() -> Self {
        Self::new()
    }
}
