use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::sync::Arc;

#[derive(Clone)]
pub struct Collapsible {
    state: ComponentState,
    expanded: bool,
    title: String,
    content: String,
    toggle_key: KeyCode,
    style: CollapsibleStyle,
    on_toggle: Option<Arc<dyn Fn(bool) -> ActionResult + Send + Sync>>,
}

impl std::fmt::Debug for Collapsible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Collapsible")
            .field("state", &self.state)
            .field("expanded", &self.expanded)
            .field("title", &self.title)
            .field("content", &self.content)
            .field("toggle_key", &self.toggle_key)
            .field("style", &self.style)
            .field("on_toggle", &self.on_toggle.is_some())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct CollapsibleStyle {
    pub bg: Color,
    pub fg: Color,
    pub border: bool,
    pub border_color: Color,
    pub border_style: Style,
    pub title_style: Style,
    pub title_expanded_style: Style,
    pub content_style: Style,
    pub indicator_color: Color,
    pub indicator_expanded: &'static str,
    pub indicator_collapsed: &'static str,
}

impl Default for CollapsibleStyle {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(26, 35, 50),
            fg: Color::Rgb(232, 232, 237),
            border: true,
            border_color: Color::Rgb(42, 42, 58),
            border_style: Style::default().fg(Color::Rgb(42, 42, 58)),
            title_style: Style::default()
                .fg(Color::Rgb(139, 176, 240))
                .add_modifier(Modifier::BOLD),
            title_expanded_style: Style::default()
                .fg(Color::Rgb(123, 200, 156))
                .add_modifier(Modifier::BOLD),
            content_style: Style::default().fg(Color::Rgb(160, 160, 176)),
            indicator_color: Color::Rgb(139, 176, 240),
            indicator_expanded: "▼",
            indicator_collapsed: "▶",
        }
    }
}

impl Collapsible {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            state: ComponentState::new(generate_component_id("collapsible")),
            expanded: false,
            title: title.into(),
            content: String::new(),
            toggle_key: KeyCode::Enter,
            style: CollapsibleStyle::default(),
            on_toggle: None,
        }
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn with_toggle_key(mut self, key: KeyCode) -> Self {
        self.toggle_key = key;
        self
    }

    pub fn with_style(mut self, style: CollapsibleStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_on_toggle<F>(mut self, callback: F) -> Self
    where
        F: Fn(bool) -> ActionResult + Send + Sync + 'static,
    {
        self.on_toggle = Some(Arc::new(callback));
        self
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    pub fn toggle(&mut self) -> ActionResult {
        self.expanded = !self.expanded;

        if let Some(callback) = &self.on_toggle {
            callback(self.expanded)
        } else {
            ActionResult::Handled
        }
    }

    pub fn is_expanded(&self) -> bool {
        self.expanded
    }

    pub fn expand(&mut self) -> ActionResult {
        if !self.expanded {
            self.expanded = true;
            if let Some(callback) = &self.on_toggle {
                return callback(true);
            }
        }
        ActionResult::Handled
    }

    pub fn collapse(&mut self) -> ActionResult {
        if self.expanded {
            self.expanded = false;
            if let Some(callback) = &self.on_toggle {
                return callback(false);
            }
        }
        ActionResult::Handled
    }
}

impl Widget for Collapsible {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = if self.style.border {
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.style.border_style)
                .border_type(ratatui::widgets::BorderType::Rounded)
        } else {
            Block::default()
        };

        let indicator = if self.expanded {
            self.style.indicator_expanded
        } else {
            self.style.indicator_collapsed
        };

        let title_style = if self.expanded {
            self.style.title_expanded_style
        } else {
            self.style.title_style
        };

        let title_line = Line::from(vec![
            Span::styled(indicator, Style::default().fg(self.style.indicator_color)),
            Span::raw(" "),
            Span::styled(&self.title, title_style),
        ]);

        let block = block.title(title_line);
        block.render(area, buf);

        let inner = if self.style.border {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        if self.expanded && !self.content.is_empty() && inner.height > 0 {
            let paragraph = Paragraph::new(self.content.as_str()).style(self.style.content_style);
            paragraph.render(inner, buf);
        }
    }
}

impl Component for Collapsible {
    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = if self.style.border {
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.style.border_style)
                .border_type(ratatui::widgets::BorderType::Rounded)
        } else {
            Block::default()
        };

        let indicator = if self.expanded {
            self.style.indicator_expanded
        } else {
            self.style.indicator_collapsed
        };

        let title_style = if self.expanded {
            self.style.title_expanded_style
        } else {
            self.style.title_style
        };

        let title_line = Line::from(vec![
            Span::styled(indicator, Style::default().fg(self.style.indicator_color)),
            Span::raw(" "),
            Span::styled(&self.title, title_style),
        ]);

        let block = block.title(title_line);
        block.render(area, buf);

        let inner = if self.style.border {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        if self.expanded && !self.content.is_empty() && inner.height > 0 {
            let paragraph = Paragraph::new(self.content.as_str()).style(self.style.content_style);
            paragraph.render(inner, buf);
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if let Event::Input(InputEvent::Key(key)) = event {
            match key.code {
                KeyCode::Enter => {
                    return self.toggle();
                }
                KeyCode::Char('e') if key.modifiers == KeyModifiers::CONTROL => {
                    return self.expand();
                }
                KeyCode::Char('w') if key.modifiers == KeyModifiers::CONTROL => {
                    return self.collapse();
                }
                _ => {}
            }
        }

        ActionResult::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapsible_creation() {
        let collapsible = Collapsible::new("Test Title");
        assert_eq!(collapsible.title, "Test Title");
        assert!(!collapsible.is_expanded());
    }

    #[test]
    fn test_collapsible_with_content() {
        let collapsible = Collapsible::new("Test").with_content("Test content");
        assert_eq!(collapsible.content, "Test content");
    }

    #[test]
    fn test_collapsible_with_expanded() {
        let collapsible = Collapsible::new("Test").with_expanded(true);
        assert!(collapsible.is_expanded());
    }

    #[test]
    fn test_toggle() {
        let mut collapsible = Collapsible::new("Test");
        collapsible.toggle();
        assert!(collapsible.is_expanded());
        collapsible.toggle();
        assert!(!collapsible.is_expanded());
    }

    #[test]
    fn test_expand() {
        let mut collapsible = Collapsible::new("Test");
        assert!(!collapsible.is_expanded());
        collapsible.expand();
        assert!(collapsible.is_expanded());
    }

    #[test]
    fn test_collapse() {
        let mut collapsible = Collapsible::new("Test").with_expanded(true);
        assert!(collapsible.is_expanded());
        collapsible.collapse();
        assert!(!collapsible.is_expanded());
    }

    #[test]
    fn test_set_content() {
        let mut collapsible = Collapsible::new("Test");
        collapsible.set_content("New content");
        assert_eq!(collapsible.content, "New content");
    }

    #[test]
    fn test_on_toggle_callback() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let toggle_count = Arc::new(AtomicUsize::new(0));
        let count = Arc::clone(&toggle_count);

        let mut collapsible = Collapsible::new("Test").with_on_toggle(move |_is_expanded| {
            count.fetch_add(1, Ordering::SeqCst);
            ActionResult::Handled
        });

        collapsible.toggle();
        assert_eq!(toggle_count.load(Ordering::SeqCst), 1);
        collapsible.toggle();
        assert_eq!(toggle_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_default_style() {
        let style = CollapsibleStyle::default();
        assert!(style.border);
    }

    #[test]
    fn test_with_toggle_key() {
        let collapsible = Collapsible::new("Test").with_toggle_key(KeyCode::Char('t'));
        assert_eq!(collapsible.toggle_key, KeyCode::Char('t'));
    }
}
