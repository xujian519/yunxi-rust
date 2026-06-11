//! Toast, Error, and UI helper methods extracted from App.

use std::time::{Duration, Instant};

use super::App;

// ── Toast ──

#[derive(Debug, Clone, PartialEq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct ToastData {
    pub message: String,
    pub level: ToastLevel,
    pub expire_at: Instant,
}

impl ToastData {
    pub fn new(message: impl Into<String>, level: ToastLevel, duration: Duration) -> Self {
        Self {
            message: message.into(),
            level,
            expire_at: Instant::now() + duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expire_at
    }
}

impl App {
    /// Push a transient toast notification (auto-dismiss after 3s).
    pub fn push_toast(&mut self, message: &str) {
        self.push_toast_level(message, ToastLevel::Info);
    }

    pub fn push_toast_success(&mut self, message: &str) {
        self.push_toast_level(message, ToastLevel::Success);
    }

    pub fn push_toast_warning(&mut self, message: &str) {
        self.push_toast_level(message, ToastLevel::Warning);
    }

    pub fn push_toast_error(&mut self, message: &str) {
        self.push_toast_level(message, ToastLevel::Error);
    }

    fn push_toast_level(&mut self, message: &str, level: ToastLevel) {
        self.toast = Some(ToastData::new(message, level, Duration::from_secs(3)));
        self.needs_render = true;
    }

    /// Check and clear expired toast.
    pub fn tick_toast(&mut self) {
        if self.toast.as_ref().is_some_and(|t| t.is_expired()) {
            self.toast = None;
            self.needs_render = true;
        }
    }
}
