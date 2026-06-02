use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, EventDispatcher};
use crate::tui::core::lifecycle::LifecycleManager;
use crate::tui::core::renderer::Renderer;
use crate::tui::state::global::GlobalState;
use crate::tui::theme::ThemeRegistry;
use crossterm::terminal;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct App {
    state: Arc<Mutex<GlobalState>>,
    event_dispatcher: EventDispatcher,
    renderer: Renderer,
    lifecycle: LifecycleManager,
    theme_registry: ThemeRegistry,
    running: bool,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        terminal::enable_raw_mode()?;

        let state = Arc::new(Mutex::new(GlobalState::new()));
        let event_dispatcher = EventDispatcher::new();
        let renderer = Renderer::new();
        let lifecycle = LifecycleManager::new();
        let theme_registry = ThemeRegistry::new();

        Ok(Self {
            state,
            event_dispatcher,
            renderer,
            lifecycle,
            theme_registry,
            running: false,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.running = true;
        self.lifecycle.on_mount();

        while self.running {
            if let Some(event) = self.wait_for_event()? {
                self.handle_event(event);
            }
            self.render()?;
            std::thread::sleep(Duration::from_millis(16));
        }

        self.lifecycle.on_unmount();
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn wait_for_event(&self) -> Result<Option<Event>, Box<dyn std::error::Error>> {
        Ok(None)
    }

    fn handle_event(&mut self, event: Event) {
        let results = self.event_dispatcher.dispatch(&event);
        for result in results {
            if let ActionResult::Action(action) = result {
                self.dispatch(action);
            }
        }
    }

    pub fn dispatch(&mut self, action: Action) {
        match action {
            Action::Quit => {
                self.running = false;
            }
            Action::ShowCommandPalette => {
                let mut state = self.state.lock().unwrap();
                state.ui.command_palette_visible = true;
            }
            Action::HideCommandPalette => {
                let mut state = self.state.lock().unwrap();
                state.ui.command_palette_visible = false;
            }
            Action::SwitchTheme(name) => {
                let theme = self.theme_registry.get(&name);
                let mut state = self.state.lock().unwrap();
                state.theme.current_theme = name.clone();
                state.theme.is_dark = theme.is_dark;
                drop(state);
                self.renderer.request_rerender();
            }
            _ => {}
        }
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.state.lock().unwrap();
        self.renderer.render(&state)?;
        Ok(())
    }

    pub fn state(&self) -> Arc<Mutex<GlobalState>> {
        Arc::clone(&self.state)
    }

    pub fn event_dispatcher(&mut self) -> &mut EventDispatcher {
        &mut self.event_dispatcher
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}
