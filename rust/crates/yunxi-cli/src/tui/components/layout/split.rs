use crate::tui::components::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Direction;
use ratatui::layout::Rect;

pub struct Split {
    state: ComponentState,
    direction: Direction,
    ratio: f32,
    resizable: bool,
    first: Box<dyn Component>,
    second: Box<dyn Component>,
}

impl Split {
    pub fn new(first: Box<dyn Component>, second: Box<dyn Component>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("split")),
            direction: Direction::Vertical,
            ratio: 0.5,
            resizable: false,
            first,
            second,
        }
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_ratio(mut self, ratio: f32) -> Self {
        self.ratio = ratio.clamp(0.1, 0.9);
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

impl Component for Split {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let split_point = match self.direction {
            Direction::Horizontal => (area.width as f32 * self.ratio) as u16,
            Direction::Vertical => (area.height as f32 * self.ratio) as u16,
        };

        let first_area = match self.direction {
            Direction::Horizontal => Rect {
                x: area.x,
                y: area.y,
                width: split_point,
                height: area.height,
            },
            Direction::Vertical => Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: split_point,
            },
        };

        let second_area = match self.direction {
            Direction::Horizontal => Rect {
                x: area.x + split_point,
                y: area.y,
                width: area.width.saturating_sub(split_point),
                height: area.height,
            },
            Direction::Vertical => Rect {
                x: area.x,
                y: area.y + split_point,
                width: area.width,
                height: area.height.saturating_sub(split_point),
            },
        };

        self.first.render(first_area, buf);
        self.second.render(second_area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible || self.state.disabled {
            return ActionResult::Ignored;
        }

        if self.resizable {
            if let Event::Input(InputEvent::Key(key_event)) = event {
                match (key_event.code, self.direction) {
                    (crossterm::event::KeyCode::Right, Direction::Horizontal)
                        if self.ratio < 0.9 =>
                    {
                        self.ratio += 0.05;
                        return ActionResult::Handled;
                    }
                    (crossterm::event::KeyCode::Left, Direction::Horizontal)
                        if self.ratio > 0.1 =>
                    {
                        self.ratio -= 0.05;
                        return ActionResult::Handled;
                    }
                    (crossterm::event::KeyCode::Down, Direction::Vertical) if self.ratio < 0.9 => {
                        self.ratio += 0.05;
                        return ActionResult::Handled;
                    }
                    (crossterm::event::KeyCode::Up, Direction::Vertical) if self.ratio > 0.1 => {
                        self.ratio -= 0.05;
                        return ActionResult::Handled;
                    }
                    _ => {}
                }
            }
        }

        let first_result = self.first.handle_event(event);
        if !matches!(first_result, ActionResult::Ignored) {
            return first_result;
        }

        self.second.handle_event(event)
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_mount(&mut self) {
        self.first.on_mount();
        self.second.on_mount();
    }

    fn on_unmount(&mut self) {
        self.first.on_unmount();
        self.second.on_unmount();
    }
}
