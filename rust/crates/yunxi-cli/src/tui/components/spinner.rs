use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, SystemEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

pub struct Spinner {
    state: ComponentState,
    style: Box<SpinnerStyle>,
    frames: Vec<String>,
    current_frame: usize,
    speed: u64,
    paused: bool,
    last_tick: u64,
}

#[derive(Debug, Clone)]
pub struct SpinnerStyle {
    pub fg: Color,
}

impl Default for SpinnerStyle {
    fn default() -> Self {
        Self {
            fg: Color::Rgb(139, 176, 240),
        }
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("spinner")),
            style: Box::new(SpinnerStyle::default()),
            frames: vec![
                "⠋".to_string(),
                "⠙".to_string(),
                "⠹".to_string(),
                "⠸".to_string(),
                "⠼".to_string(),
                "⠴".to_string(),
                "⠦".to_string(),
                "⠧".to_string(),
                "⠇".to_string(),
                "⠏".to_string(),
            ],
            current_frame: 0,
            speed: 100,
            paused: false,
            last_tick: 0,
        }
    }

    pub fn with_style(mut self, style: SpinnerStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_speed(mut self, speed: u64) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_dots_style(mut self) -> Self {
        self.frames = vec![
            "⣾".to_string(),
            "⣽".to_string(),
            "⣻".to_string(),
            "⢿".to_string(),
        ];
        self
    }

    pub fn with_line_style(mut self) -> Self {
        self.frames = vec![
            "-".to_string(),
            "\\".to_string(),
            "|".to_string(),
            "/".to_string(),
        ];
        self
    }

    pub fn with_arrows_style(mut self) -> Self {
        self.frames = vec![
            "↖".to_string(),
            "↑".to_string(),
            "↗".to_string(),
            "→".to_string(),
            "↘".to_string(),
            "↓".to_string(),
            "↙".to_string(),
            "←".to_string(),
        ];
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn advance_frame(&mut self) {
        if self.paused {
            return;
        }
        self.current_frame = (self.current_frame + 1) % self.frames.len();
    }
}

impl Component for Spinner {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let frame = self
            .frames
            .get(self.current_frame)
            .unwrap_or(&self.frames[0]);
        let style = Style::default().fg(self.style.fg);

        let widget = Paragraph::new(frame.as_str()).style(style);
        widget.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::System(SystemEvent::Tick) => {
                self.advance_frame();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::core::event::SystemEvent;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new();
        assert!(spinner.get_state().visible);
        assert!(!spinner.is_paused());
    }

    #[test]
    fn test_spinner_with_style() {
        let style = SpinnerStyle { fg: Color::Red };
        let spinner = Spinner::new().with_style(style);
        assert!(spinner.get_state().visible);
    }

    #[test]
    fn test_spinner_pause_resume() {
        let mut spinner = Spinner::new();
        assert!(!spinner.is_paused());

        spinner.pause();
        assert!(spinner.is_paused());

        spinner.resume();
        assert!(!spinner.is_paused());
    }

    #[test]
    fn test_spinner_set_paused() {
        let mut spinner = Spinner::new();
        spinner.set_paused(true);
        assert!(spinner.is_paused());
        spinner.set_paused(false);
        assert!(!spinner.is_paused());
    }

    #[test]
    fn test_spinner_advance_frame() {
        let mut spinner = Spinner::new();
        let initial_frame = spinner.current_frame;
        spinner.advance_frame();
        assert_ne!(initial_frame, spinner.current_frame);
    }

    #[test]
    fn test_spinner_tick_event() {
        let mut spinner = Spinner::new();
        let initial_frame = spinner.current_frame;
        let event = Event::System(SystemEvent::Tick);
        let result = spinner.handle_event(&event);
        assert!(matches!(result, ActionResult::Handled));
        assert_ne!(initial_frame, spinner.current_frame);
    }

    #[test]
    fn test_spinner_tick_while_paused() {
        let mut spinner = Spinner::new();
        spinner.pause();
        let initial_frame = spinner.current_frame;
        let event = Event::System(SystemEvent::Tick);
        spinner.handle_event(&event);
        assert_eq!(initial_frame, spinner.current_frame);
    }

    #[test]
    fn test_spinner_dots_style() {
        let spinner = Spinner::new().with_dots_style();
        assert_eq!(spinner.frames.len(), 4);
    }

    #[test]
    fn test_spinner_line_style() {
        let spinner = Spinner::new().with_line_style();
        assert_eq!(spinner.frames.len(), 4);
    }

    #[test]
    fn test_spinner_arrows_style() {
        let spinner = Spinner::new().with_arrows_style();
        assert_eq!(spinner.frames.len(), 8);
    }

    #[test]
    fn test_spinner_render() {
        let spinner = Spinner::new();
        let backend = TestBackend::new(10, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                spinner.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_spinner_id_generation() {
        let spinner = Spinner::new();
        assert!(spinner.get_state().id.starts_with("spinner_"));
    }

    #[test]
    fn test_spinner_with_id() {
        let spinner = Spinner::new().with_id("custom_spinner".to_string());
        assert_eq!(spinner.get_state().id, "custom_spinner");
    }

    #[test]
    fn test_spinner_frame_cycle() {
        let mut spinner = Spinner::new();
        let len = spinner.frames.len();
        for _ in 0..len {
            spinner.advance_frame();
        }
        assert_eq!(spinner.current_frame, 0);
    }
}
