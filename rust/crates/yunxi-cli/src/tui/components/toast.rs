use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::Action;
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Debug, Clone, PartialEq)]
pub enum ToastLevel {
    Info,
    Warning,
    Error,
    Success,
}

pub struct ToastMessage {
    level: ToastLevel,
    message: String,
    duration_ms: u64,
    created_at: u64,
}

impl ToastMessage {
    pub fn new(level: ToastLevel, message: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            level,
            message: message.into(),
            duration_ms,
            created_at: 0,
        }
    }
}

pub struct Toast {
    state: ComponentState,
    style: Box<ToastStyle>,
    messages: Vec<ToastMessage>,
    current_time: u64,
    max_messages: usize,
    position: ToastPosition,
    dismissible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ToastPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Clone)]
pub struct ToastStyle {
    pub info_bg: Color,
    pub info_fg: Color,
    pub warning_bg: Color,
    pub warning_fg: Color,
    pub error_bg: Color,
    pub error_fg: Color,
    pub success_bg: Color,
    pub success_fg: Color,
}

impl Default for ToastStyle {
    fn default() -> Self {
        Self {
            info_bg: Color::Rgb(26, 35, 50),
            info_fg: Color::Rgb(139, 176, 240),
            warning_bg: Color::Rgb(61, 50, 26),
            warning_fg: Color::Rgb(240, 187, 139),
            error_bg: Color::Rgb(61, 26, 26),
            error_fg: Color::Rgb(240, 139, 139),
            success_bg: Color::Rgb(26, 61, 26),
            success_fg: Color::Rgb(139, 240, 139),
        }
    }
}

impl Default for Toast {
    fn default() -> Self {
        Self::new()
    }
}

impl Toast {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("toast")),
            style: Box::new(ToastStyle::default()),
            messages: Vec::new(),
            current_time: 0,
            max_messages: 5,
            position: ToastPosition::TopRight,
            dismissible: false,
        }
    }

    pub fn with_style(mut self, style: ToastStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = max;
        self
    }

    pub fn with_position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn with_dismissible(mut self) -> Self {
        self.dismissible = true;
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn add_message(&mut self, level: ToastLevel, message: impl Into<String>, duration_ms: u64) {
        let mut msg = ToastMessage::new(level, message, duration_ms);
        msg.created_at = self.current_time;
        self.messages.push(msg);

        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn add_info(&mut self, message: impl Into<String>, duration_ms: u64) {
        self.add_message(ToastLevel::Info, message, duration_ms);
    }

    pub fn add_warning(&mut self, message: impl Into<String>, duration_ms: u64) {
        self.add_message(ToastLevel::Warning, message, duration_ms);
    }

    pub fn add_error(&mut self, message: impl Into<String>, duration_ms: u64) {
        self.add_message(ToastLevel::Error, message, duration_ms);
    }

    pub fn add_success(&mut self, message: impl Into<String>, duration_ms: u64) {
        self.add_message(ToastLevel::Success, message, duration_ms);
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    pub fn dismiss(&mut self) {
        if !self.messages.is_empty() {
            self.messages.remove(0);
        }
    }

    pub fn update_time(&mut self, elapsed: u64) {
        self.current_time += elapsed;
        self.messages
            .retain(|msg| self.current_time.saturating_sub(msg.created_at) < msg.duration_ms);
    }

    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    fn get_level_style(&self, level: &ToastLevel) -> (Color, Color) {
        match level {
            ToastLevel::Info => (self.style.info_bg, self.style.info_fg),
            ToastLevel::Warning => (self.style.warning_bg, self.style.warning_fg),
            ToastLevel::Error => (self.style.error_bg, self.style.error_fg),
            ToastLevel::Success => (self.style.success_bg, self.style.success_fg),
        }
    }

    fn get_level_icon(&self, level: &ToastLevel) -> &'static str {
        match level {
            ToastLevel::Info => "ℹ",
            ToastLevel::Warning => "⚠",
            ToastLevel::Error => "✕",
            ToastLevel::Success => "✓",
        }
    }

    fn get_level_label(&self, level: &ToastLevel) -> &'static str {
        match level {
            ToastLevel::Info => "INFO",
            ToastLevel::Warning => "WARN",
            ToastLevel::Error => "ERROR",
            ToastLevel::Success => "SUCCESS",
        }
    }
}

impl Component for Toast {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible || self.messages.is_empty() {
            return;
        }

        let msg_count = self.messages.len().min(area.height as usize);
        let total_height: u16 = msg_count as u16;

        for (i, msg) in self.messages.iter().enumerate().take(msg_count) {
            let y = if matches!(
                self.position,
                ToastPosition::TopRight | ToastPosition::TopLeft
            ) {
                area.y + i as u16
            } else {
                area.y + total_height - 1 - i as u16
            };

            let msg_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };

            let (_bg_color, fg_color) = self.get_level_style(&msg.level);
            let icon = self.get_level_icon(&msg.level);
            let label = self.get_level_label(&msg.level);

            let mut content = vec![
                Span::styled(icon, Style::default().fg(fg_color)),
                Span::raw(" "),
                Span::styled(
                    label,
                    Style::default().fg(fg_color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(": "),
                Span::styled(&msg.message, Style::default().fg(fg_color)),
            ];

            if self.dismissible {
                content.push(Span::raw(" "));
                content.push(Span::styled(
                    "[ESC]",
                    Style::default().fg(fg_color).add_modifier(Modifier::DIM),
                ));
            }

            let lines = vec![Line::from(content)];

            let widget = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL))
                .wrap(Wrap { trim: true });

            widget.render(msg_area, buf);
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible {
            return ActionResult::Ignored;
        }

        if !self.dismissible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key))
                if key.code == KeyCode::Esc && key.modifiers == KeyModifiers::NONE =>
            {
                self.dismiss();
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
    use crossterm::event::KeyEvent;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_toast_creation() {
        let toast = Toast::new();
        assert!(toast.get_state().visible);
        assert!(toast.is_empty());
    }

    #[test]
    fn test_toast_with_style() {
        let style = ToastStyle::default();
        let toast = Toast::new().with_style(style);
        assert!(toast.get_state().visible);
    }

    #[test]
    fn test_toast_add_info() {
        let mut toast = Toast::new();
        toast.add_info("Test message", 1000);
        assert_eq!(toast.message_count(), 1);
    }

    #[test]
    fn test_toast_add_warning() {
        let mut toast = Toast::new();
        toast.add_warning("Warning message", 1000);
        assert_eq!(toast.message_count(), 1);
    }

    #[test]
    fn test_toast_add_error() {
        let mut toast = Toast::new();
        toast.add_error("Error message", 1000);
        assert_eq!(toast.message_count(), 1);
    }

    #[test]
    fn test_toast_add_success() {
        let mut toast = Toast::new();
        toast.add_success("Success message", 1000);
        assert_eq!(toast.message_count(), 1);
    }

    #[test]
    fn test_toast_multiple_messages() {
        let mut toast = Toast::new();
        toast.add_info("Message 1", 1000);
        toast.add_warning("Message 2", 1000);
        toast.add_error("Message 3", 1000);
        assert_eq!(toast.message_count(), 3);
    }

    #[test]
    fn test_toast_max_messages() {
        let mut toast = Toast::new().with_max_messages(2);
        toast.add_info("Message 1", 1000);
        toast.add_info("Message 2", 1000);
        toast.add_info("Message 3", 1000);
        assert_eq!(toast.message_count(), 2);
    }

    #[test]
    fn test_toast_clear_messages() {
        let mut toast = Toast::new();
        toast.add_info("Test", 1000);
        assert!(!toast.is_empty());
        toast.clear_messages();
        assert!(toast.is_empty());
    }

    #[test]
    fn test_toast_dismiss() {
        let mut toast = Toast::new();
        toast.add_info("Message 1", 1000);
        toast.add_info("Message 2", 1000);
        toast.dismiss();
        assert_eq!(toast.message_count(), 1);
    }

    #[test]
    fn test_toast_dismiss_empty() {
        let mut toast = Toast::new();
        toast.dismiss();
        assert!(toast.is_empty());
    }

    #[test]
    fn test_toast_update_time() {
        let mut toast = Toast::new();
        toast.add_info("Test", 1000);
        toast.update_time(500);
        assert_eq!(toast.message_count(), 1);
        toast.update_time(400);
        assert_eq!(toast.message_count(), 1);
        toast.update_time(100);
        assert!(toast.is_empty());
    }

    #[test]
    fn test_toast_with_dismissible() {
        let toast = Toast::new().with_dismissible();
        assert!(toast.dismissible);
    }

    #[test]
    fn test_toast_handle_event_dismiss() {
        let mut toast = Toast::new().with_dismissible();
        toast.add_info("Test", 1000);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        let result = toast.handle_event(&event);
        assert!(matches!(result, ActionResult::Handled));
        assert!(toast.is_empty());
    }

    #[test]
    fn test_toast_handle_event_ignored_when_not_dismissible() {
        let mut toast = Toast::new();
        toast.add_info("Test", 1000);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        let result = toast.handle_event(&event);
        assert!(matches!(result, ActionResult::Ignored));
        assert!(!toast.is_empty());
    }

    #[test]
    fn test_toast_render() {
        let mut toast = Toast::new();
        toast.add_info("Test message", 1000);
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                toast.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_toast_render_empty() {
        let toast = Toast::new();
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                toast.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_toast_id_generation() {
        let toast = Toast::new();
        assert!(toast.get_state().id.starts_with("toast_"));
    }

    #[test]
    fn test_toast_with_id() {
        let toast = Toast::new().with_id("custom_toast".to_string());
        assert_eq!(toast.get_state().id, "custom_toast");
    }

    #[test]
    fn test_toast_message_level() {
        let mut toast = Toast::new();
        toast.add_info("Info", 1000);
        toast.add_warning("Warning", 1000);
        toast.add_error("Error", 1000);
        toast.add_success("Success", 1000);
        assert_eq!(toast.messages[0].level, ToastLevel::Info);
        assert_eq!(toast.messages[1].level, ToastLevel::Warning);
        assert_eq!(toast.messages[2].level, ToastLevel::Error);
        assert_eq!(toast.messages[3].level, ToastLevel::Success);
    }

    #[test]
    fn test_toast_get_level_label() {
        let toast = Toast::new();
        assert_eq!(toast.get_level_label(&ToastLevel::Info), "INFO");
        assert_eq!(toast.get_level_label(&ToastLevel::Warning), "WARN");
        assert_eq!(toast.get_level_label(&ToastLevel::Error), "ERROR");
        assert_eq!(toast.get_level_label(&ToastLevel::Success), "SUCCESS");
    }

    #[test]
    fn test_toast_get_level_icon() {
        let toast = Toast::new();
        assert_eq!(toast.get_level_icon(&ToastLevel::Info), "ℹ");
        assert_eq!(toast.get_level_icon(&ToastLevel::Warning), "⚠");
        assert_eq!(toast.get_level_icon(&ToastLevel::Error), "✕");
        assert_eq!(toast.get_level_icon(&ToastLevel::Success), "✓");
    }
}
