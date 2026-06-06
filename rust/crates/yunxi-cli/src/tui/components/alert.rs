use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

pub struct AlertAction {
    label: String,
    callback: Box<dyn Fn() -> ActionResult + Send + Sync>,
}

impl AlertAction {
    pub fn new(
        label: impl Into<String>,
        callback: impl Fn() -> ActionResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            callback: Box::new(callback),
        }
    }
}

pub struct Alert {
    state: ComponentState,
    style: Box<AlertStyle>,
    level: AlertLevel,
    title: String,
    message: String,
    actions: Vec<AlertAction>,
    dismissible: bool,
    visible: bool,
    selected_action: usize,
}

#[derive(Debug, Clone)]
pub struct AlertStyle {
    pub info_bg: Color,
    pub info_fg: Color,
    pub warning_bg: Color,
    pub warning_fg: Color,
    pub error_bg: Color,
    pub error_fg: Color,
    pub critical_bg: Color,
    pub critical_fg: Color,
    pub border: bool,
}

impl Default for AlertStyle {
    fn default() -> Self {
        Self {
            info_bg: Color::Rgb(26, 35, 50),
            info_fg: Color::Rgb(139, 176, 240),
            warning_bg: Color::Rgb(61, 50, 26),
            warning_fg: Color::Rgb(240, 187, 139),
            error_bg: Color::Rgb(61, 26, 26),
            error_fg: Color::Rgb(240, 139, 139),
            critical_bg: Color::Rgb(61, 0, 0),
            critical_fg: Color::Rgb(255, 0, 0),
            border: true,
        }
    }
}

impl Default for Alert {
    fn default() -> Self {
        Self::new(AlertLevel::Info, "Info", "")
    }
}

impl Alert {
    pub fn new(level: AlertLevel, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("alert")),
            style: Box::new(AlertStyle::default()),
            level,
            title: title.into(),
            message: message.into(),
            actions: Vec::new(),
            dismissible: false,
            visible: true,
            selected_action: 0,
        }
    }

    pub fn with_style(mut self, style: AlertStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_level(mut self, level: AlertLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_dismissible(mut self) -> Self {
        self.dismissible = true;
        self
    }

    pub fn with_action(mut self, action: AlertAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.state.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
        self.state.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn add_action(&mut self, action: AlertAction) {
        self.actions.push(action);
    }

    pub fn clear_actions(&mut self) {
        self.actions.clear();
    }

    fn get_level_style(&self) -> (Color, Color, &'static str) {
        match self.level {
            AlertLevel::Info => (self.style.info_bg, self.style.info_fg, "ℹ"),
            AlertLevel::Warning => (self.style.warning_bg, self.style.warning_fg, "⚠"),
            AlertLevel::Error => (self.style.error_bg, self.style.error_fg, "✕"),
            AlertLevel::Critical => (self.style.critical_bg, self.style.critical_fg, "‼"),
        }
    }

    fn get_level_border_style(&self) -> ratatui::style::Style {
        let (_, fg, _) = self.get_level_style();
        Style::default().fg(fg)
    }

    fn handle_key_event(&mut self, key_code: KeyCode) -> ActionResult {
        match key_code {
            KeyCode::Esc if self.dismissible => {
                self.hide();
                ActionResult::Handled
            }
            KeyCode::Enter => {
                if !self.actions.is_empty() {
                    let action_index = self.selected_action.min(self.actions.len() - 1);
                    let action = &self.actions[action_index];
                    let result = (action.callback)();
                    self.hide();
                    result
                } else if self.dismissible {
                    self.hide();
                    ActionResult::Handled
                } else {
                    ActionResult::Ignored
                }
            }
            KeyCode::Left | KeyCode::Right => {
                if !self.actions.is_empty() {
                    let count = self.actions.len();
                    self.selected_action = if key_code == KeyCode::Right {
                        (self.selected_action + 1) % count
                    } else {
                        self.selected_action.saturating_sub(1)
                    };
                    ActionResult::Handled
                } else {
                    ActionResult::Ignored
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = c.to_digit(10).unwrap_or(0) as usize;
                if idx > 0 && idx <= self.actions.len() {
                    self.selected_action = idx - 1;
                    let result = (self.actions[self.selected_action].callback)();
                    self.hide();
                    result
                } else {
                    ActionResult::Ignored
                }
            }
            _ => ActionResult::Ignored,
        }
    }
}

impl Component for Alert {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible || !self.visible {
            return;
        }

        let (_bg_color, fg_color, icon) = self.get_level_style();
        let border_style = self.get_level_border_style();

        let title_style = Style::default().fg(fg_color).add_modifier(Modifier::BOLD);

        let content_style = Style::default().fg(fg_color);

        let mut lines = vec![
            Line::from(vec![
                Span::styled(icon, title_style),
                Span::raw(" "),
                Span::styled(&self.title, title_style),
            ]),
            Line::from(""),
        ];

        for line in self.message.lines() {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(line, content_style),
            ]));
        }

        if !self.actions.is_empty() {
            lines.push(Line::from(""));

            let mut action_spans = vec![Span::raw("  Actions: ")];
            for (i, action) in self.actions.iter().enumerate() {
                let is_selected = i == self.selected_action;
                let action_style = if is_selected {
                    Style::default()
                        .fg(fg_color)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default().fg(fg_color).add_modifier(Modifier::DIM)
                };

                if i > 0 {
                    action_spans.push(Span::raw(" | "));
                }

                action_spans.push(Span::styled(
                    format!("{}.{}", i + 1, action.label),
                    action_style,
                ));
            }
            lines.push(Line::from(action_spans));
        }

        if self.dismissible {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "[ESC] 关闭",
                    Style::default().fg(fg_color).add_modifier(Modifier::DIM),
                ),
            ]));
        }

        let widget = if self.style.border {
            Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(border_style),
                )
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left)
        } else {
            Paragraph::new(lines)
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Left)
        };

        widget.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible || !self.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => self.handle_key_event(key.code),
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
    use crate::tui::core::action::Action;
    use crossterm::event::{KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_alert_creation() {
        let alert = Alert::new(AlertLevel::Info, "Title", "Message");
        assert!(alert.get_state().visible);
        assert!(alert.is_visible());
    }

    #[test]
    fn test_alert_with_style() {
        let style = AlertStyle::default();
        let alert = Alert::new(AlertLevel::Info, "Title", "Message").with_style(style);
        assert!(alert.get_state().visible);
    }

    #[test]
    fn test_alert_with_dismissible() {
        let alert = Alert::new(AlertLevel::Info, "Title", "Message").with_dismissible();
        assert!(alert.dismissible);
    }

    #[test]
    fn test_alert_show_hide() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message");
        assert!(alert.is_visible());
        alert.hide();
        assert!(!alert.is_visible());
        alert.show();
        assert!(alert.is_visible());
    }

    #[test]
    fn test_alert_set_message() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Old message");
        alert.set_message("New message");
        assert_eq!(alert.message, "New message");
    }

    #[test]
    fn test_alert_with_action() {
        let action = AlertAction::new("OK", || ActionResult::Handled);
        let alert = Alert::new(AlertLevel::Info, "Title", "Message").with_action(action);
        assert_eq!(alert.actions.len(), 1);
    }

    #[test]
    fn test_alert_add_action() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message");
        alert.add_action(AlertAction::new("OK", || ActionResult::Handled));
        alert.add_action(AlertAction::new("Cancel", || ActionResult::Handled));
        assert_eq!(alert.actions.len(), 2);
    }

    #[test]
    fn test_alert_clear_actions() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message");
        alert.add_action(AlertAction::new("OK", || ActionResult::Handled));
        alert.add_action(AlertAction::new("Cancel", || ActionResult::Handled));
        alert.clear_actions();
        assert!(alert.actions.is_empty());
    }

    #[test]
    fn test_alert_handle_esc_dismissible() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message").with_dismissible();
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        let result = alert.handle_event(&event);
        assert!(matches!(result, ActionResult::Handled));
        assert!(!alert.is_visible());
    }

    #[test]
    fn test_alert_handle_esc_not_dismissible() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message");
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        let result = alert.handle_event(&event);
        assert!(matches!(result, ActionResult::Ignored));
        assert!(alert.is_visible());
    }

    #[test]
    fn test_alert_handle_enter_with_actions() {
        let action = AlertAction::new("OK", || {
            ActionResult::Action(Action::Navigate("/test".to_string()))
        });
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message").with_action(action);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )));
        let result = alert.handle_event(&event);
        match result {
            ActionResult::Action(Action::Navigate(route)) => {
                assert_eq!(route, "/test");
            }
            _ => panic!("Expected Navigate action"),
        }
        assert!(!alert.is_visible());
    }

    #[test]
    fn test_alert_handle_left_right() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message")
            .with_action(AlertAction::new("OK", || ActionResult::Handled))
            .with_action(AlertAction::new("Cancel", || ActionResult::Handled));

        let event_right = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )));
        alert.handle_event(&event_right);
        assert_eq!(alert.selected_action, 1);

        let event_left = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Left,
            KeyModifiers::NONE,
        )));
        alert.handle_event(&event_left);
        assert_eq!(alert.selected_action, 0);
    }

    #[test]
    fn test_alert_handle_number_key() {
        let action1 = AlertAction::new("OK", || {
            ActionResult::Action(Action::Navigate("/ok".to_string()))
        });
        let action2 = AlertAction::new("Cancel", || {
            ActionResult::Action(Action::Navigate("/cancel".to_string()))
        });
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message")
            .with_action(action1)
            .with_action(action2);

        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('2'),
            KeyModifiers::NONE,
        )));
        let result = alert.handle_event(&event);
        match result {
            ActionResult::Action(Action::Navigate(route)) => {
                assert_eq!(route, "/cancel");
            }
            _ => panic!("Expected Navigate action"),
        }
    }

    #[test]
    fn test_alert_render() {
        let alert = Alert::new(AlertLevel::Info, "Test Alert", "This is a test message");
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                alert.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_alert_render_with_actions() {
        let alert = Alert::new(AlertLevel::Warning, "Warning", "Test message")
            .with_action(AlertAction::new("OK", || ActionResult::Handled))
            .with_action(AlertAction::new("Cancel", || ActionResult::Handled));
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                alert.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_alert_render_hidden() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message");
        alert.hide();
        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                alert.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_alert_id_generation() {
        let alert = Alert::new(AlertLevel::Info, "Title", "Message");
        assert!(alert.get_state().id.starts_with("alert_"));
    }

    #[test]
    fn test_alert_with_id() {
        let alert =
            Alert::new(AlertLevel::Info, "Title", "Message").with_id("custom_alert".to_string());
        assert_eq!(alert.get_state().id, "custom_alert");
    }

    #[test]
    fn test_alert_levels() {
        let alert_info = Alert::new(AlertLevel::Info, "Info", "");
        assert_eq!(alert_info.level, AlertLevel::Info);

        let alert_warning = Alert::new(AlertLevel::Warning, "Warning", "");
        assert_eq!(alert_warning.level, AlertLevel::Warning);

        let alert_error = Alert::new(AlertLevel::Error, "Error", "");
        assert_eq!(alert_error.level, AlertLevel::Error);

        let alert_critical = Alert::new(AlertLevel::Critical, "Critical", "");
        assert_eq!(alert_critical.level, AlertLevel::Critical);
    }

    #[test]
    fn test_alert_get_level_style() {
        let alert_info = Alert::new(AlertLevel::Info, "Info", "");
        let (_, _, icon) = alert_info.get_level_style();
        assert_eq!(icon, "ℹ");

        let alert_warning = Alert::new(AlertLevel::Warning, "Warning", "");
        let (_, _, icon) = alert_warning.get_level_style();
        assert_eq!(icon, "⚠");

        let alert_error = Alert::new(AlertLevel::Error, "Error", "");
        let (_, _, icon) = alert_error.get_level_style();
        assert_eq!(icon, "✕");

        let alert_critical = Alert::new(AlertLevel::Critical, "Critical", "");
        let (_, _, icon) = alert_critical.get_level_style();
        assert_eq!(icon, "‼");
    }

    #[test]
    fn test_alert_multiline_message() {
        let message = "Line 1\nLine 2\nLine 3";
        let alert = Alert::new(AlertLevel::Info, "Title", message);
        assert_eq!(alert.message.lines().count(), 3);
    }

    #[test]
    fn test_alert_action_cycle() {
        let mut alert = Alert::new(AlertLevel::Info, "Title", "Message")
            .with_action(AlertAction::new("A", || ActionResult::Handled))
            .with_action(AlertAction::new("B", || ActionResult::Handled));

        alert.selected_action = 0;
        let event_right = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )));
        alert.handle_event(&event_right);
        assert_eq!(alert.selected_action, 1);

        alert.handle_event(&event_right);
        assert_eq!(alert.selected_action, 0);
    }
}
