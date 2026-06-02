#[cfg(test)]
mod tests {
    use crate::tui::components::*;
    use crate::tui::core::action::Action;
    use crate::tui::core::action::ActionResult;
    use crate::tui::core::event::{Event, InputEvent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Click me");
        assert!(!button.get_state().disabled);
        assert!(!button.get_state().focused);
    }

    #[test]
    fn test_button_with_style() {
        let style = ButtonStyle::default();
        let button = Button::new("Styled").with_style(style.clone());
        let state = button.get_state();
        assert!(!state.disabled);
    }

    #[test]
    fn test_button_click_handler() {
        let mut button = Button::new("Click me").with_on_click(|| {
            ActionResult::Action(Action::Navigate("/test".to_string()))
        });

        button.set_focused(true);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE
        )));

        let result = button.handle_event(&event);
        match result {
            ActionResult::Action(Action::Navigate(route)) => {
                assert_eq!(route, "/test");
            }
            _ => panic!("Expected Navigate action"),
        }
    }

    #[test]
    fn test_button_disabled() {
        let mut button = Button::new("Disabled");
        button.set_disabled(true);
        assert!(button.get_state().disabled);

        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE
        )));
        let result = button.handle_event(&event);
        assert!(matches!(result, ActionResult::Ignored));
    }

    #[test]
    fn test_button_focus() {
        let mut button = Button::new("Test");
        assert!(!button.is_focused());

        button.set_focused(true);
        assert!(button.is_focused());

        button.set_focused(false);
        assert!(!button.is_focused());
    }

    #[test]
    fn test_button_render() {
        let button = Button::new("Test");

        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            button.render(f.area(), f.buffer_mut());
        }).unwrap();
    }

    #[test]
    fn test_label_creation() {
        let label = Label::new("Test label");
        assert!(label.get_state().visible);
    }

    #[test]
    fn test_label_color() {
        let label = Label::new("Colored");
        assert_eq!(label.get_state().id.starts_with("label_"), true);
    }

    #[test]
    fn test_component_id_generation() {
        let id1 = generate_component_id("test");
        let id2 = generate_component_id("test");

        assert!(id1.starts_with("test_"));
        assert!(id2.starts_with("test_"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_spacer_creation() {
        let spacer = Spacer::new();
        assert!(spacer.get_state().visible);
    }

    #[test]
    fn test_component_state_builder() {
        let state = ComponentState::new("test".to_string())
            .with_visible(false)
            .with_focused(true);

        assert!(!state.visible);
        assert!(state.focused);
        assert!(!state.disabled);
    }
}
