use crate::tui::components::base::{Component, ComponentState, generate_component_id};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Style};

pub struct Container {
    state: ComponentState,
    children: Vec<Box<dyn Component>>,
    padding: u16,
    margin: u16,
    background: Option<Color>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("container")),
            children: Vec::new(),
            padding: 0,
            margin: 0,
            background: None,
        }
    }

    pub fn with_padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_margin(mut self, margin: u16) -> Self {
        self.margin = margin;
        self
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn add_child(mut self, child: Box<dyn Component>) -> Self {
        self.children.push(child);
        self
    }

    pub fn add_children(mut self, children: Vec<Box<dyn Component>>) -> Self {
        self.children.extend(children);
        self
    }
}

impl Component for Container {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let content_area = Rect {
            x: area.x.saturating_add(self.margin),
            y: area.y.saturating_add(self.margin),
            width: area.width.saturating_sub(self.margin * 2),
            height: area.height.saturating_sub(self.margin * 2),
        };

        if let Some(bg_color) = self.background {
            let style = Style::default().bg(bg_color);
            for y in content_area.top()..content_area.bottom() {
                for x in content_area.left()..content_area.right() {
                    buf.get_mut(x, y).set_style(style);
                }
            }
        }

        let child_area = Rect {
            x: content_area.x.saturating_add(self.padding),
            y: content_area.y.saturating_add(self.padding),
            width: content_area.width.saturating_sub(self.padding * 2),
            height: content_area.height.saturating_sub(self.padding * 2),
        };

        for child in &self.children {
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

    fn on_resize(&mut self, area: Rect) {
        self.state.bounds = area;
        for child in &mut self.children {
            child.on_resize(area);
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
