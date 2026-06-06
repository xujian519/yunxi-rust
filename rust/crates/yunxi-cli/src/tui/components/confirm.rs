use super::base::{Component, ComponentState};
use super::button::{Button, ButtonStyle};
use super::layout::Flex;
use super::modal::Modal;
use super::spacer::Spacer;
use crate::tui::core::action::Action;
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::layout::{Alignment, Direction};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

pub struct Confirm {
    modal: Modal,
    message: String,
    confirm_text: String,
    cancel_text: String,
    confirm_button: Button,
    cancel_button: Button,
    confirm_action: Option<ActionResult>,
    cancel_action: Option<ActionResult>,
}

impl Confirm {
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();

        let confirm_style = ButtonStyle {
            normal_bg: Color::Rgb(139, 92, 246),
            normal_fg: Color::Rgb(255, 255, 255),
            focused_bg: Color::Rgb(167, 139, 250),
            focused_fg: Color::Rgb(13, 13, 18),
            disabled_bg: Color::Rgb(22, 22, 30),
            disabled_fg: Color::Rgb(106, 106, 128),
            border: false,
        };

        let cancel_style = ButtonStyle {
            normal_bg: Color::Rgb(26, 35, 50),
            normal_fg: Color::Rgb(232, 232, 237),
            focused_bg: Color::Rgb(51, 65, 85),
            focused_fg: Color::Rgb(232, 232, 237),
            disabled_bg: Color::Rgb(22, 22, 30),
            disabled_fg: Color::Rgb(106, 106, 128),
            border: false,
        };

        let confirm_button = Button::new("Confirm")
            .with_style(confirm_style)
            .with_id("confirm_ok_button");

        let cancel_button = Button::new("Cancel")
            .with_style(cancel_style)
            .with_id("confirm_cancel_button");

        let confirm = Self {
            modal: Modal::new(
                "Confirm",
                Box::new(
                    Flex::new()
                        .with_direction(Direction::Vertical)
                        .with_id("confirm_content_layout"),
                ),
            ),
            message,
            confirm_text: "Confirm".to_string(),
            cancel_text: "Cancel".to_string(),
            confirm_button,
            cancel_button,
            confirm_action: None,
            cancel_action: None,
        };

        confirm
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.modal = self.modal.with_title(title);
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn with_confirm_text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.confirm_text = text.clone();
        self.confirm_button = Button::new(text)
            .with_style(self.confirm_button.get_style().clone())
            .with_id("confirm_ok_button");
        self
    }

    pub fn with_cancel_text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.cancel_text = text.clone();
        self.cancel_button = Button::new(text)
            .with_style(self.cancel_button.get_style().clone())
            .with_id("confirm_cancel_button");
        self
    }

    pub fn with_confirm_button_style(mut self, style: ButtonStyle) -> Self {
        self.confirm_button = Button::new(&self.confirm_text)
            .with_style(style)
            .with_id("confirm_ok_button");
        self
    }

    pub fn with_cancel_button_style(mut self, style: ButtonStyle) -> Self {
        self.cancel_button = Button::new(&self.cancel_text)
            .with_style(style)
            .with_id("confirm_cancel_button");
        self
    }

    pub fn with_on_confirm(mut self, action: ActionResult) -> Self {
        self.confirm_action = Some(action);
        self
    }

    pub fn with_on_cancel(mut self, action: ActionResult) -> Self {
        self.cancel_action = Some(action);
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.modal = self.modal.with_id(id);
        self
    }

    pub fn show(&mut self) {
        self.modal.show();
    }

    pub fn hide(&mut self) {
        self.modal.hide();
    }

    pub fn is_visible(&self) -> bool {
        self.modal.is_visible()
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.modal.set_focused(focused);
    }

    fn build_content_layout(&self) -> Flex {
        let message_text = self.message.clone();

        let confirm_style = self.confirm_button.get_style().clone();
        let cancel_style = self.cancel_button.get_style().clone();

        Flex::new()
            .with_direction(Direction::Vertical)
            .add_child(Box::new(CustomParagraph::new(message_text)))
            .add_child(Box::new(Spacer::new()))
            .add_child(Box::new(
                Flex::new()
                    .with_direction(Direction::Horizontal)
                    .add_child(Box::new(
                        Button::new(&self.confirm_text)
                            .with_style(confirm_style.clone())
                            .with_id("confirm_ok_button"),
                    ))
                    .add_child(Box::new(Spacer::new()))
                    .add_child(Box::new(Spacer::new()))
                    .add_child(Box::new(
                        Button::new(&self.cancel_text)
                            .with_style(cancel_style.clone())
                            .with_id("confirm_cancel_button"),
                    )),
            ))
            .with_id("confirm_content_layout")
    }
}

struct CustomParagraph {
    text: String,
}

impl CustomParagraph {
    fn new(text: String) -> Self {
        Self { text }
    }
}

impl Component for CustomParagraph {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(self.text.as_str())
            .style(Style::default().fg(Color::Rgb(232, 232, 237)))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });
        paragraph.render(area, buf);
    }

    fn handle_event(&mut self, _event: &Event) -> ActionResult {
        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        ComponentState::new("custom_paragraph".to_string())
    }

    fn on_focus(&mut self, _focused: bool) {}
}

impl Component for Confirm {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.modal.is_visible() {
            return;
        }

        self.modal.render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.modal.is_visible() {
            return ActionResult::Ignored;
        }

        if let Event::Input(InputEvent::Key(key)) = event {
            if key.code == KeyCode::Esc && key.modifiers == KeyModifiers::NONE {
                if self.cancel_action.is_some() {
                    let action = self.cancel_action.take().unwrap();
                    self.hide();
                    return action;
                }
                self.hide();
                return ActionResult::Action(Action::HideDialog);
            }
        }

        if matches!(
            self.modal.handle_event(event),
            ActionResult::Handled | ActionResult::Action(_)
        ) {
            return ActionResult::Ignored;
        }

        let confirm_result = self.confirm_button.handle_event(event);
        if matches!(
            confirm_result,
            ActionResult::Handled | ActionResult::Action(_)
        ) {
            self.hide();
            if let Some(ref action) = self.confirm_action {
                return action.clone();
            }
            return ActionResult::Action(Action::Navigate("/home".to_string()));
        }

        let cancel_result = self.cancel_button.handle_event(event);
        if matches!(
            cancel_result,
            ActionResult::Handled | ActionResult::Action(_)
        ) {
            self.hide();
            if let Some(ref action) = self.cancel_action {
                return action.clone();
            }
            return ActionResult::Action(Action::HideDialog);
        }

        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.modal.get_state()
    }

    fn on_focus(&mut self, focused: bool) {
        self.modal.on_focus(focused);
        self.confirm_button.on_focus(focused);
        self.cancel_button.on_focus(focused);
    }

    fn on_resize(&mut self, area: Rect) {
        self.modal.on_resize(area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_creation() {
        let confirm = Confirm::new("Are you sure?");

        assert!(!confirm.is_visible());
        assert_eq!(confirm.message, "Are you sure?");
        assert_eq!(confirm.confirm_text, "Confirm");
        assert_eq!(confirm.cancel_text, "Cancel");
    }

    #[test]
    fn test_confirm_with_options() {
        let confirm = Confirm::new("Delete this item?")
            .with_title("Delete Confirmation")
            .with_confirm_text("Yes, Delete")
            .with_cancel_text("No, Keep")
            .with_on_confirm(ActionResult::Action(Action::ExecuteCommand(
                "delete".to_string(),
            )));

        assert_eq!(confirm.message, "Delete this item?");
        assert_eq!(confirm.confirm_text, "Yes, Delete");
        assert_eq!(confirm.cancel_text, "No, Keep");
        assert!(confirm.confirm_action.is_some());
    }

    #[test]
    fn test_confirm_show_hide() {
        let mut confirm = Confirm::new("Test message");

        assert!(!confirm.is_visible());

        confirm.show();
        assert!(confirm.is_visible());

        confirm.hide();
        assert!(!confirm.is_visible());
    }

    #[test]
    fn test_confirm_handle_esc() {
        let mut confirm = Confirm::new("Test message");
        confirm.show();

        let key_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        let result = confirm.handle_event(&key_event);
        assert!(!confirm.is_visible());
        assert!(matches!(result, ActionResult::Action(_)));
    }

    #[test]
    fn test_confirm_with_custom_styles() {
        let custom_style = ButtonStyle {
            normal_bg: Color::Red,
            normal_fg: Color::White,
            focused_bg: Color::Rgb(255, 100, 100),
            focused_fg: Color::Black,
            disabled_bg: Color::Rgb(50, 0, 0),
            disabled_fg: Color::Rgb(100, 100, 100),
            border: true,
        };

        let confirm = Confirm::new("Test")
            .with_confirm_button_style(custom_style.clone())
            .with_cancel_button_style(custom_style);

        assert!(confirm.confirm_button.get_style().border);
        assert!(confirm.cancel_button.get_style().border);
    }
}
