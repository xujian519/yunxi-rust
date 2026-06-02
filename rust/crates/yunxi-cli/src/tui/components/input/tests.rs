#[cfg(test)]
mod tests {
    use crate::tui::components::input::*;
    use crate::tui::components::base::Component;
    use crate::tui::core::action::ActionResult;
    use crate::tui::core::event::{Event, InputEvent};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_text_input_creation() {
        let input = TextInput::new();
        assert!(input.get_value().is_empty());
        assert!(input.get_state().visible);
    }

    #[test]
    fn test_text_input_with_value() {
        let input = TextInput::new().with_value("Hello".to_string());
        assert_eq!(input.get_value(), "Hello");
    }

    #[test]
    fn test_text_input_insert_char() {
        let mut input = TextInput::new();

        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('A'), KeyModifiers::NONE
        )));
        input.handle_event(&event);

        assert_eq!(input.get_value(), "A");
    }

    #[test]
    fn test_text_input_delete_char() {
        let mut input = TextInput::new().with_value("Hello".to_string());
        assert_eq!(input.get_value(), "Hello");

        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Backspace, KeyModifiers::NONE
        )));
        input.handle_event(&event);

        assert_eq!(input.get_value(), "Hell");
    }

    #[test]
    fn test_text_input_clear() {
        let mut input = TextInput::new().with_value("Test".to_string());
        input.clear();
        assert!(input.get_value().is_empty());
    }

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt::new("请输入名称:".to_string());
        assert!(prompt.get_value().is_empty());
    }

    #[test]
    fn test_prompt_with_value() {
        let mut prompt = Prompt::new("输入:".to_string());
        prompt.set_value("Test".to_string());
        assert_eq!(prompt.get_value(), "Test");
    }

    #[test]
    fn test_prompt_clear() {
        let mut prompt = Prompt::new("输入:".to_string());
        prompt.set_value("Test".to_string());
        prompt.clear();
        assert!(prompt.get_value().is_empty());
    }

    #[test]
    fn test_text_input_placeholder() {
        let input = TextInput::new()
            .with_placeholder("请输入内容...".to_string());
        assert!(input.get_value().is_empty());
    }
}
