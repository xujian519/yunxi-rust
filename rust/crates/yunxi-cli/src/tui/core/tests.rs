#[cfg(test)]
mod tests {
    use crate::tui::core::action::{Action, ActionResult};
    use crate::tui::core::event::{ActionEvent, Event, EventDispatcher};
    use crate::tui::core::lifecycle::LifecycleManager;
    use crate::tui::state::global::GlobalState;
    use crate::tui::theme::{Theme, ThemeRegistry};
    use crate::tui::router::{Router, RouteType};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_action_display() {
        let action = Action::Navigate("/home".to_string());
        assert_eq!(action.to_string(), "Navigate(/home)");

        let action = Action::ShowDialog("test".to_string());
        assert_eq!(action.to_string(), "ShowDialog(test)");
    }

    #[test]
    fn test_theme_creation() {
        let theme = Theme::default_dark();
        assert_eq!(theme.name, "default_dark");
        assert!(theme.is_dark);

        let theme = Theme::default_light();
        assert_eq!(theme.name, "default_light");
        assert!(!theme.is_dark);
    }

    #[test]
    fn test_theme_registry() {
        let mut registry = ThemeRegistry::new();
        registry.register(Theme::default_dark());

        let theme = registry.get("default_dark");
        assert_eq!(theme.name, "default_dark");

        let theme = registry.get("nonexistent");
        assert_eq!(theme.name, "default_dark");
    }

    #[test]
    fn test_global_state_creation() {
        let state = GlobalState::new();
        assert_eq!(state.theme.current_theme, "default_dark");
        assert!(state.theme.is_dark);
        assert!(state.ui.show_sidebar);
    }

    #[test]
    fn test_router_navigation() {
        let mut router = Router::new();
        assert_eq!(router.current_route(), "/home");

        router.navigate("/session/123".to_string());
        assert_eq!(router.current_route(), "/session/123");

        assert!(router.go_back());
        assert_eq!(router.current_route(), "/home");

        assert!(!router.go_back());

        assert!(router.go_forward());
        assert_eq!(router.current_route(), "/session/123");

        assert!(!router.go_forward());
    }

    #[test]
    fn test_router_parse() {
        let router = Router::new();
        assert!(matches!(router.parse_route(), RouteType::Home));

        let mut router = Router::new();
        router.navigate("/settings".to_string());
        assert!(matches!(router.parse_route(), RouteType::Settings));

        let mut router = Router::new();
        router.navigate("/session/test123".to_string());
        match router.parse_route() {
            RouteType::Session(id) => assert_eq!(id, "test123"),
            _ => panic!("Expected Session route"),
        }
    }

    #[test]
    fn test_event_dispatcher() {
        let mut dispatcher = EventDispatcher::new();
        let action_executed = Arc::new(AtomicBool::new(false));

        let action_executed_clone = Arc::clone(&action_executed);
        dispatcher.subscribe(move |event| {
            if let Event::Action(ActionEvent::UserAction(_)) = event {
                action_executed_clone.store(true, Ordering::SeqCst);
                ActionResult::Handled
            } else {
                ActionResult::Ignored
            }
        });

        let event = Event::Action(ActionEvent::UserAction(Action::Quit));
        dispatcher.dispatch(&event);

        assert!(action_executed.load(Ordering::SeqCst));
    }

    #[test]
    fn test_lifecycle_manager() {
        let mut lifecycle = LifecycleManager::new();
        let mount_called = Arc::new(AtomicBool::new(false));
        let unmount_called = Arc::new(AtomicBool::new(false));

        let mount_called_clone = Arc::clone(&mount_called);
        lifecycle.register_on_mount(move || {
            mount_called_clone.store(true, Ordering::SeqCst);
        });

        let unmount_called_clone = Arc::clone(&unmount_called);
        lifecycle.register_on_unmount(move || {
            unmount_called_clone.store(true, Ordering::SeqCst);
        });

        lifecycle.on_mount();
        assert!(mount_called.load(Ordering::SeqCst));
        assert!(!unmount_called.load(Ordering::SeqCst));

        lifecycle.on_unmount();
        assert!(unmount_called.load(Ordering::SeqCst));
    }
}
