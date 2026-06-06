use crate::tui::core::action::{Action, ActionResult};
use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, Clone)]
pub enum Event {
    Input(InputEvent),
    Action(ActionEvent),
    System(SystemEvent),
    Network(NetworkEvent),
    Timer(TimerEvent),
}

#[derive(Debug, Clone)]
pub enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
}

#[derive(Debug, Clone)]
pub enum ActionEvent {
    UserAction(Action),
    ComponentAction(String, Action),
}

#[derive(Debug, Clone)]
pub enum SystemEvent {
    Tick,
    FocusGained,
    FocusLost,
    Terminate,
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Connected,
    Disconnected,
    Message(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum TimerEvent {
    Timeout(u64),
    Interval(u64),
}

#[allow(clippy::type_complexity)]
pub struct EventDispatcher {
    listeners: Vec<Box<dyn Fn(&Event) -> ActionResult + Send + Sync>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn subscribe<F>(&mut self, handler: F)
    where
        F: Fn(&Event) -> ActionResult + Send + Sync + 'static,
    {
        self.listeners.push(Box::new(handler));
    }

    pub fn dispatch(&self, event: &Event) -> Vec<ActionResult> {
        self.listeners
            .iter()
            .map(|listener| listener(event))
            .collect()
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
