use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::Action;
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear};

pub struct Modal {
    state: ComponentState,
    title: String,
    content: Box<dyn Component>,
    visible: bool,
    backdrop: bool,
    close_on_esc: bool,
    close_on_click_outside: bool,
    on_close: Option<Box<dyn Fn() -> ActionResult + Send + Sync>>,
}

impl Modal {
    pub fn new(title: impl Into<String>, content: Box<dyn Component>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("modal")),
            title: title.into(),
            content,
            visible: false,
            backdrop: true,
            close_on_esc: true,
            close_on_click_outside: true,
            on_close: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self.state.visible = visible;
        self
    }

    pub fn with_backdrop(mut self, backdrop: bool) -> Self {
        self.backdrop = backdrop;
        self
    }

    pub fn with_close_on_esc(mut self, close_on_esc: bool) -> Self {
        self.close_on_esc = close_on_esc;
        self
    }

    pub fn with_close_on_click_outside(mut self, close_on_click_outside: bool) -> Self {
        self.close_on_click_outside = close_on_click_outside;
        self
    }

    pub fn with_on_close<F>(mut self, callback: F) -> Self
    where
        F: Fn() -> ActionResult + Send + Sync + 'static,
    {
        self.on_close = Some(Box::new(callback));
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

    pub fn set_focused(&mut self, focused: bool) {
        self.state.focused = focused;
        self.content.on_focus(focused);
    }

    fn calculate_modal_area(&self, terminal_area: Rect) -> Rect {
        let width = terminal_area.width.min(80);
        let height = terminal_area.height.min(24);

        Rect {
            x: (terminal_area.width - width) / 2,
            y: (terminal_area.height - height) / 2,
            width,
            height,
        }
    }

    fn render_backdrop(&self, area: Rect, buf: &mut Buffer) {
        if !self.backdrop {
            return;
        }

        let backdrop_style = Style::default()
            .bg(Color::Rgb(0, 0, 0))
            .fg(Color::Rgb(0, 0, 0));

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_style(backdrop_style);
                    cell.set_char(' ');
                }
            }
        }
    }

    fn is_click_outside(&self, click_x: u16, click_y: u16, terminal_area: Rect) -> bool {
        if !self.visible {
            return false;
        }

        let modal_area = self.calculate_modal_area(terminal_area);

        click_x < modal_area.x
            || click_x >= modal_area.x + modal_area.width
            || click_y < modal_area.y
            || click_y >= modal_area.y + modal_area.height
    }

    fn handle_close(&mut self) -> ActionResult {
        self.hide();
        if let Some(ref callback) = self.on_close {
            return callback();
        }
        ActionResult::Action(Action::HideDialog)
    }
}

impl Component for Modal {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.visible {
            return;
        }

        if self.backdrop {
            self.render_backdrop(area, buf);
        }

        let modal_area = self.calculate_modal_area(area);

        Clear.render(modal_area, buf);

        let title_style = Style::default()
            .fg(Color::Rgb(232, 232, 237))
            .add_modifier(Modifier::BOLD);

        let border_style = if self.state.focused {
            Style::default().fg(Color::Rgb(139, 176, 240))
        } else {
            Style::default().fg(Color::Rgb(106, 106, 128))
        };

        let border = Block::default()
            .title(self.title.as_str())
            .title_style(title_style)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(border_style);

        let content_area = border.inner(modal_area);
        border.render(modal_area, buf);

        self.content.render(content_area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.visible {
            return ActionResult::Ignored;
        }

        if let Event::Input(InputEvent::Resize(width, height)) = event {
            self.state.bounds = Rect::new(0, 0, *width, *height);
            return ActionResult::Handled;
        }

        let content_result = self.content.handle_event(event);

        if matches!(
            content_result,
            ActionResult::Handled | ActionResult::Action(_)
        ) {
            return content_result;
        }

        match event {
            Event::Input(InputEvent::Key(key)) if self.close_on_esc => {
                if key.code == KeyCode::Esc && key.modifiers == KeyModifiers::NONE {
                    return self.handle_close();
                }
            }
            Event::Input(InputEvent::Mouse(mouse_event)) if self.close_on_click_outside => {
                if let MouseEventKind::Down(crossterm::event::MouseButton::Left) = mouse_event.kind
                {
                    if self.is_click_outside(mouse_event.column, mouse_event.row, self.state.bounds)
                    {
                        return self.handle_close();
                    }
                }
            }
            _ => {}
        }

        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
        self.content.on_focus(focused);
    }

    fn on_resize(&mut self, area: Rect) {
        self.state.bounds = area;
        self.content.on_resize(area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::components::label::Label;

    fn create_test_modal() -> Modal {
        let content = Box::new(Label::new("Test content"));
        Modal::new("Test Modal", content).with_visible(true)
    }

    #[test]
    fn test_modal_creation() {
        let content = Box::new(Label::new("Test content"));
        let modal = Modal::new("Test Modal", content);

        assert!(!modal.is_visible());
        assert!(modal.close_on_esc);
        assert!(modal.close_on_click_outside);
        assert!(modal.backdrop);
    }

    #[test]
    fn test_modal_show_hide() {
        let mut modal = create_test_modal();

        modal.show();
        assert!(modal.is_visible());

        modal.hide();
        assert!(!modal.is_visible());
    }

    #[test]
    fn test_modal_with_options() {
        let content = Box::new(Label::new("Test content"));
        let modal = Modal::new("Test Modal", content)
            .with_visible(true)
            .with_backdrop(false)
            .with_close_on_esc(false)
            .with_close_on_click_outside(false)
            .with_id("custom_id".to_string());

        assert!(modal.is_visible());
        assert!(!modal.backdrop);
        assert!(!modal.close_on_esc);
        assert!(!modal.close_on_click_outside);
        assert_eq!(modal.state.id, "custom_id");
    }

    #[test]
    fn test_modal_handle_esc() {
        let mut modal = create_test_modal();
        modal.state.bounds = Rect::new(0, 0, 100, 100);

        let key_event = Event::Input(InputEvent::Key(crossterm::event::KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }));

        let result = modal.handle_event(&key_event);
        assert!(!modal.is_visible());
        assert!(matches!(result, ActionResult::Action(_)));
    }

    #[test]
    fn test_modal_handle_click_outside() {
        let mut modal = create_test_modal();
        modal.state.bounds = Rect::new(0, 0, 100, 100);

        let modal_area = modal.calculate_modal_area(modal.state.bounds);
        let outside_x = modal_area.x - 1;
        let outside_y = modal_area.y;

        assert!(modal.is_click_outside(outside_x, outside_y, modal.state.bounds));

        let inside_x = modal_area.x + 1;
        let inside_y = modal_area.y + 1;

        assert!(!modal.is_click_outside(inside_x, inside_y, modal.state.bounds));
    }

    #[test]
    fn test_modal_calculate_area() {
        let modal = create_test_modal();
        let terminal_area = Rect::new(0, 0, 100, 50);

        let modal_area = modal.calculate_modal_area(terminal_area);

        assert_eq!(modal_area.width, 80);
        assert_eq!(modal_area.height, 24);
        assert_eq!(modal_area.x, 10);
        assert_eq!(modal_area.y, 13);
    }

    #[test]
    fn test_modal_with_small_terminal() {
        let modal = create_test_modal();
        let terminal_area = Rect::new(0, 0, 50, 20);

        let modal_area = modal.calculate_modal_area(terminal_area);

        assert_eq!(modal_area.width, 50);
        assert_eq!(modal_area.height, 20);
    }
}
