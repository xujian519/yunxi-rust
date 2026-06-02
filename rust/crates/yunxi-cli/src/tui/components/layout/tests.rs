#[cfg(test)]
mod tests {
    use crate::tui::components::*;
    use crate::tui::components::layout::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_container_creation() {
        let container = Container::new();
        let state = container.get_state();
        assert!(state.visible);
    }

    #[test]
    fn test_container_render() {
        let container = Container::new()
            .with_padding(1)
            .with_margin(1);

        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            container.render(f.area(), f.buffer_mut());
        }).unwrap();
    }

    #[test]
    fn test_flex_creation() {
        let flex = Flex::new();
        let state = flex.get_state();
        assert!(state.id.starts_with("flex_"));
    }

    #[test]
    fn test_flex_render_with_children() {
        let flex = Flex::new()
            .add_child(Box::new(Button::new("Button 1")))
            .add_child(Box::new(Button::new("Button 2")));

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            flex.render(f.area(), f.buffer_mut());
        }).unwrap();
    }

    #[test]
    fn test_split_creation() {
        let first = Box::new(Button::new("First"));
        let second = Box::new(Button::new("Second"));
        let split = Split::new(first, second);
        let state = split.get_state();
        assert!(state.id.starts_with("split_"));
    }

    #[test]
    fn test_split_render() {
        let first = Box::new(Label::new("First"));
        let second = Box::new(Label::new("Second"));
        let split = Split::new(first, second)
            .with_ratio(0.3);

        let backend = TestBackend::new(40, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| {
            split.render(f.area(), f.buffer_mut());
        }).unwrap();
    }

    #[test]
    fn test_flex_direction_change() {
        let flex = Flex::new();
        let _ = flex.get_state();
    }
}
