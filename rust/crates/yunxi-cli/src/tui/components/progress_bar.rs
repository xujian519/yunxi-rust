use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

pub struct ProgressBar {
    state: ComponentState,
    style: Box<ProgressBarStyle>,
    progress: f64,
    label: Option<String>,
    indeterminate: bool,
    indeterminate_position: u16,
}

#[derive(Debug, Clone)]
pub struct ProgressBarStyle {
    pub fill_color: Color,
    pub empty_color: Color,
    pub text_color: Color,
    pub striped: bool,
}

impl Default for ProgressBarStyle {
    fn default() -> Self {
        Self {
            fill_color: Color::Rgb(139, 176, 240),
            empty_color: Color::Rgb(26, 35, 50),
            text_color: Color::Rgb(232, 232, 237),
            striped: false,
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("progress_bar")),
            style: Box::new(ProgressBarStyle::default()),
            progress: 0.0,
            label: None,
            indeterminate: false,
            indeterminate_position: 0,
        }
    }

    pub fn with_style(mut self, style: ProgressBarStyle) -> Self {
        self.style = Box::new(style);
        self
    }

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_indeterminate(mut self) -> Self {
        self.indeterminate = true;
        self
    }

    pub fn with_striped(mut self) -> Self {
        self.style.striped = true;
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn get_progress(&self) -> f64 {
        self.progress
    }

    pub fn set_label(&mut self, label: impl Into<String>) {
        self.label = Some(label.into());
    }

    pub fn clear_label(&mut self) {
        self.label = None;
    }

    pub fn advance_indeterminate(&mut self) {
        if self.indeterminate {
            self.indeterminate_position = (self.indeterminate_position + 1) % 20;
        }
    }

    fn calculate_fill_width(&self, total_width: u16) -> u16 {
        if self.indeterminate {
            let base_width = (total_width as f64 * 0.3) as u16;
            let max_position = 20;
            let cycle = total_width as f64 / max_position as f64;
            let position = self.indeterminate_position as f64 * cycle;
            let start = position.min(total_width as f64 - base_width as f64) as u16;
            start
        } else {
            (total_width as f64 * self.progress) as u16
        }
    }

    fn get_progress_text(&self) -> String {
        if let Some(ref label) = self.label {
            format!("{} {:.0}%", label, self.progress * 100.0)
        } else {
            format!("{:.0}%", self.progress * 100.0)
        }
    }
}

impl Component for ProgressBar {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let width = area.width.max(2) - 2;
        let fill_width = self.calculate_fill_width(width);

        let mut line_spans = Vec::new();

        if self.style.striped {
            for i in 0..width {
                if i < fill_width {
                    let span_style = if i % 2 == 0 {
                        Style::default().fg(self.style.fill_color)
                    } else {
                        Style::default()
                            .fg(self.style.fill_color)
                            .add_modifier(Modifier::DIM)
                    };
                    line_spans.push(Span::styled("█", span_style));
                } else {
                    line_spans.push(Span::styled(
                        "░",
                        Style::default().fg(self.style.empty_color),
                    ));
                }
            }
        } else {
            let filled = "█".repeat(fill_width as usize);
            let empty = "░".repeat((width - fill_width) as usize);
            line_spans.push(Span::styled(
                filled,
                Style::default().fg(self.style.fill_color),
            ));
            line_spans.push(Span::styled(
                empty,
                Style::default().fg(self.style.empty_color),
            ));
        }

        let mut lines = vec![Line::from(line_spans)];

        if self.indeterminate {
            let label = "加载中...".to_string();
            lines.push(Line::from(Span::styled(
                label,
                Style::default().fg(self.style.text_color),
            )));
        } else if let Some(ref _label) = self.label {
            let progress_text = self.get_progress_text();
            lines.push(Line::from(Span::styled(
                progress_text,
                Style::default().fg(self.style.text_color),
            )));
        }

        let widget = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

        widget.render(area, buf);
    }

    fn handle_event(&mut self, _event: &Event) -> ActionResult {
        if !self.state.visible {
            return ActionResult::Ignored;
        }

        if self.indeterminate {
            self.advance_indeterminate();
        }

        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_progress_bar_creation() {
        let bar = ProgressBar::new();
        assert!(bar.get_state().visible);
        assert_eq!(bar.get_progress(), 0.0);
    }

    #[test]
    fn test_progress_bar_with_style() {
        let style = ProgressBarStyle::default();
        let bar = ProgressBar::new().with_style(style);
        assert!(bar.get_state().visible);
    }

    #[test]
    fn test_progress_bar_with_progress() {
        let bar = ProgressBar::new().with_progress(0.5);
        assert_eq!(bar.get_progress(), 0.5);
    }

    #[test]
    fn test_progress_bar_with_label() {
        let bar = ProgressBar::new().with_label("Downloading");
        assert!(bar.label.is_some());
    }

    #[test]
    fn test_progress_bar_set_progress() {
        let mut bar = ProgressBar::new();
        bar.set_progress(0.75);
        assert_eq!(bar.get_progress(), 0.75);
    }

    #[test]
    fn test_progress_bar_clamping() {
        let bar = ProgressBar::new().with_progress(1.5);
        assert_eq!(bar.get_progress(), 1.0);

        let bar = ProgressBar::new().with_progress(-0.5);
        assert_eq!(bar.get_progress(), 0.0);
    }

    #[test]
    fn test_progress_bar_set_label() {
        let mut bar = ProgressBar::new();
        bar.set_label("Loading");
        assert!(bar.label.is_some());
    }

    #[test]
    fn test_progress_bar_clear_label() {
        let mut bar = ProgressBar::new().with_label("Test");
        assert!(bar.label.is_some());
        bar.clear_label();
        assert!(bar.label.is_none());
    }

    #[test]
    fn test_progress_bar_with_striped() {
        let bar = ProgressBar::new().with_striped();
        assert!(bar.style.striped);
    }

    #[test]
    fn test_progress_bar_with_indeterminate() {
        let bar = ProgressBar::new().with_indeterminate();
        assert!(bar.indeterminate);
    }

    #[test]
    fn test_progress_bar_advance_indeterminate() {
        let mut bar = ProgressBar::new().with_indeterminate();
        let initial_pos = bar.indeterminate_position;
        bar.advance_indeterminate();
        assert_ne!(initial_pos, bar.indeterminate_position);
    }

    #[test]
    fn test_progress_bar_no_indeterminate_advance() {
        let mut bar = ProgressBar::new();
        let initial_pos = bar.indeterminate_position;
        bar.advance_indeterminate();
        assert_eq!(initial_pos, bar.indeterminate_position);
    }

    #[test]
    fn test_progress_bar_render() {
        let bar = ProgressBar::new().with_progress(0.5);
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                bar.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_progress_bar_striped_render() {
        let bar = ProgressBar::new().with_progress(0.5).with_striped();
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                bar.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_progress_bar_indeterminate_render() {
        let bar = ProgressBar::new().with_indeterminate();
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                bar.render(f.area(), f.buffer_mut());
            })
            .unwrap();
    }

    #[test]
    fn test_progress_bar_id_generation() {
        let bar = ProgressBar::new();
        assert!(bar.get_state().id.starts_with("progress_bar_"));
    }

    #[test]
    fn test_progress_bar_with_id() {
        let bar = ProgressBar::new().with_id("custom_bar".to_string());
        assert_eq!(bar.get_state().id, "custom_bar");
    }

    #[test]
    fn test_progress_bar_get_progress_text() {
        let bar = ProgressBar::new().with_label("Test").with_progress(0.75);
        let text = bar.get_progress_text();
        assert!(text.contains("75%"));
        assert!(text.contains("Test"));
    }

    #[test]
    fn test_progress_bar_get_progress_text_no_label() {
        let bar = ProgressBar::new().with_progress(0.5);
        let text = bar.get_progress_text();
        assert!(text.contains("50%"));
        assert!(!text.contains("Test"));
    }
}
