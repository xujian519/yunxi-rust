use crate::tui::core::event::Event;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct DebounceConfig {
    pub delay: Duration,
    pub max_events: usize,
}

impl Default for DebounceConfig {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(50),
            max_events: 100,
        }
    }
}

pub struct EventHandler {
    queue: VecDeque<Event>,
    debounce: DebounceConfig,
    last_process: Instant,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            debounce: DebounceConfig::default(),
            last_process: Instant::now(),
        }
    }

    pub fn with_debounce(mut self, config: DebounceConfig) -> Self {
        self.debounce = config;
        self
    }

    pub fn push(&mut self, event: Event) {
        if self.queue.len() >= self.debounce.max_events {
            self.queue.pop_front();
        }
        self.queue.push_back(event);
    }

    pub fn drain(&mut self) -> Vec<Event> {
        if Instant::now().duration_since(self.last_process) < self.debounce.delay {
            return Vec::new();
        }
        self.last_process = Instant::now();
        self.queue.drain(..).collect()
    }

    pub fn drain_all(&mut self) -> Vec<Event> {
        self.last_process = Instant::now();
        self.queue.drain(..).collect()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    pub fn clear(&mut self) {
        self.queue.clear();
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::core::action::Action;
    use crate::tui::core::event::{ActionEvent, Event};

    fn make_event() -> Event {
        Event::Action(ActionEvent::UserAction(Action::Refresh))
    }

    #[test]
    fn test_push_and_drain() {
        let mut h = EventHandler::new();
        h.push(make_event());
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn test_max_queue() {
        let config = DebounceConfig {
            delay: Duration::from_millis(0),
            max_events: 3,
        };
        let mut h = EventHandler::new().with_debounce(config);
        for _ in 0..5 {
            h.push(make_event());
        }
        assert_eq!(h.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut h = EventHandler::new();
        h.push(make_event());
        h.clear();
        assert!(h.is_empty());
    }
}
