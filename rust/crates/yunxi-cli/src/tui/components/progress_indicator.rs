use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressType {
    Spinner,
    Bar,
    Dots,
    Arrow,
    Text,
}

#[derive(Debug, Clone)]
pub struct ProgressStyle {
    pub bg: Color,
    pub fg: Color,
    pub border: bool,
    pub border_color: Color,
    pub title_style: Style,
    pub bar_filled_color: Color,
    pub bar_empty_color: Color,
    pub spinner_color: Color,
    pub text_color: Color,
    pub percentage_style: Style,
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(26, 35, 50),
            fg: Color::Rgb(232, 232, 237),
            border: true,
            border_color: Color::Rgb(42, 42, 58),
            title_style: Style::default()
                .fg(Color::Rgb(139, 176, 240))
                .add_modifier(Modifier::BOLD),
            bar_filled_color: Color::Rgb(139, 176, 240),
            bar_empty_color: Color::Rgb(42, 42, 58),
            spinner_color: Color::Rgb(139, 176, 240),
            text_color: Color::Rgb(160, 160, 176),
            percentage_style: Style::default()
                .fg(Color::Rgb(139, 176, 240))
                .add_modifier(Modifier::BOLD),
        }
    }
}

const SPINNER_FRAMES: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠇"];

const DOTS_FRAMES: [&str; 4] = ["⣾", "⣽", "⣻", "⢿"];

const ARROW_FRAMES: [&str; 4] = ["←", "↑", "→", "↓"];

pub struct ProgressIndicator {
    progress_type: ProgressType,
    title: String,
    current: f32,
    total: f32,
    message: Option<String>,
    style: ProgressStyle,
    start_time: Instant,
    animate: bool,
}

impl ProgressIndicator {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            progress_type: ProgressType::Spinner,
            title: title.into(),
            current: 0.0,
            total: 100.0,
            message: None,
            style: ProgressStyle::default(),
            start_time: Instant::now(),
            animate: true,
        }
    }

    pub fn with_type(mut self, progress_type: ProgressType) -> Self {
        self.progress_type = progress_type;
        self
    }

    pub fn with_progress(mut self, current: f32, total: f32) -> Self {
        self.current = current;
        self.total = total;
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    pub fn set_progress(&mut self, current: f32) {
        self.current = current;
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = Some(message.into());
    }

    pub fn get_percentage(&self) -> f32 {
        if self.total <= 0.0 {
            return 0.0;
        }
        (self.current / self.total) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.total
    }

    pub fn get_elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn render_spinner(&self, area: Rect, buf: &mut Buffer) {
        let frame = if self.animate {
            let elapsed = self.start_time.elapsed().as_millis() as usize;
            SPINNER_FRAMES[elapsed / 100 % SPINNER_FRAMES.len()]
        } else {
            SPINNER_FRAMES[0]
        };

        let inner_area = self.render_frame(area, buf);

        let line = Line::from(vec![
            Span::styled(frame, Style::default().fg(self.style.spinner_color)),
            Span::raw(" "),
            Span::styled(&self.title, self.style.title_style),
        ]);

        ratatui::widgets::Paragraph::new(line).render(inner_area, buf);
    }

    fn render_bar(&self, area: Rect, buf: &mut Buffer) {
        let inner_area = self.render_frame(area, buf);

        let percentage = self.get_percentage();
        let bar_width = (percentage / 100.0 * inner_area.width as f32) as usize;

        let filled = "█".repeat(bar_width);
        let empty = "░".repeat((inner_area.width as usize).saturating_sub(bar_width));

        let mut spans = vec![
            Span::styled(&self.title, self.style.title_style),
            Span::raw(" "),
        ];

        if !filled.is_empty() {
            spans.push(Span::styled(
                filled,
                Style::default().fg(self.style.bar_filled_color),
            ));
        }

        if !empty.is_empty() {
            spans.push(Span::styled(
                empty,
                Style::default().fg(self.style.bar_empty_color),
            ));
        }

        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!("{:.0}%", percentage),
            self.style.percentage_style,
        ));

        ratatui::widgets::Paragraph::new(Line::from(spans)).render(inner_area, buf);
    }

    fn render_dots(&self, area: Rect, buf: &mut Buffer) {
        let frame = if self.animate {
            let elapsed = self.start_time.elapsed().as_millis() as usize;
            DOTS_FRAMES[elapsed / 200 % DOTS_FRAMES.len()]
        } else {
            DOTS_FRAMES[0]
        };

        let inner_area = self.render_frame(area, buf);

        let line = Line::from(vec![
            Span::styled(frame, Style::default().fg(self.style.spinner_color)),
            Span::raw(" "),
            Span::styled(&self.title, self.style.title_style),
        ]);

        ratatui::widgets::Paragraph::new(line).render(inner_area, buf);
    }

    fn render_arrow(&self, area: Rect, buf: &mut Buffer) {
        let frame = if self.animate {
            let elapsed = self.start_time.elapsed().as_millis() as usize;
            ARROW_FRAMES[elapsed / 200 % ARROW_FRAMES.len()]
        } else {
            ARROW_FRAMES[0]
        };

        let inner_area = self.render_frame(area, buf);

        let line = Line::from(vec![
            Span::styled(frame, Style::default().fg(self.style.spinner_color)),
            Span::raw(" "),
            Span::styled(&self.title, self.style.title_style),
        ]);

        ratatui::widgets::Paragraph::new(line).render(inner_area, buf);
    }

    fn render_text(&self, area: Rect, buf: &mut Buffer) {
        let inner_area = self.render_frame(area, buf);

        let percentage = self.get_percentage();
        let elapsed = self.get_elapsed_time().as_secs();

        let mut spans = vec![
            Span::styled(&self.title, self.style.title_style),
            Span::raw(" "),
            Span::styled(format!("{:.0}%", percentage), self.style.percentage_style),
        ];

        if let Some(message) = &self.message {
            spans.push(Span::raw(" - "));
            spans.push(Span::styled(
                message,
                Style::default().fg(self.style.text_color),
            ));
        }

        spans.push(Span::raw(format!(" ({}s)", elapsed)));

        ratatui::widgets::Paragraph::new(Line::from(spans)).render(inner_area, buf);
    }

    fn render_frame(&self, area: Rect, buf: &mut Buffer) -> Rect {
        if self.style.border {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.style.border_color))
                .border_type(ratatui::widgets::BorderType::Rounded);
            block.render(area, buf);

            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        }
    }
}

impl Widget for ProgressIndicator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.progress_type {
            ProgressType::Spinner => self.render_spinner(area, buf),
            ProgressType::Bar => self.render_bar(area, buf),
            ProgressType::Dots => self.render_dots(area, buf),
            ProgressType::Arrow => self.render_arrow(area, buf),
            ProgressType::Text => self.render_text(area, buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_indicator_creation() {
        let indicator = ProgressIndicator::new("Test Progress");
        assert_eq!(indicator.title, "Test Progress");
        assert_eq!(indicator.current, 0.0);
        assert_eq!(indicator.total, 100.0);
    }

    #[test]
    fn test_with_type() {
        let indicator = ProgressIndicator::new("Test").with_type(ProgressType::Bar);
        assert_eq!(indicator.progress_type, ProgressType::Bar);
    }

    #[test]
    fn test_with_progress() {
        let indicator = ProgressIndicator::new("Test").with_progress(50.0, 200.0);
        assert_eq!(indicator.current, 50.0);
        assert_eq!(indicator.total, 200.0);
    }

    #[test]
    fn test_get_percentage() {
        let indicator = ProgressIndicator::new("Test").with_progress(50.0, 100.0);
        assert_eq!(indicator.get_percentage(), 50.0);
    }

    #[test]
    fn test_get_percentage_zero_total() {
        let indicator = ProgressIndicator::new("Test").with_progress(50.0, 0.0);
        assert_eq!(indicator.get_percentage(), 0.0);
    }

    #[test]
    fn test_is_complete() {
        let indicator = ProgressIndicator::new("Test").with_progress(100.0, 100.0);
        assert!(indicator.is_complete());
    }

    #[test]
    fn test_is_not_complete() {
        let indicator = ProgressIndicator::new("Test").with_progress(50.0, 100.0);
        assert!(!indicator.is_complete());
    }

    #[test]
    fn test_set_progress() {
        let mut indicator = ProgressIndicator::new("Test");
        indicator.set_progress(75.0);
        assert_eq!(indicator.current, 75.0);
    }

    #[test]
    fn test_set_message() {
        let mut indicator = ProgressIndicator::new("Test");
        indicator.set_message("Processing...");
        assert_eq!(indicator.message.as_ref().unwrap(), "Processing...");
    }

    #[test]
    fn test_with_animate() {
        let indicator = ProgressIndicator::new("Test").with_animate(false);
        assert!(!indicator.animate);
    }

    #[test]
    fn test_default_style() {
        let style = ProgressStyle::default();
        assert!(style.border);
        assert_eq!(style.bg, Color::Rgb(26, 35, 50));
        assert_eq!(style.fg, Color::Rgb(232, 232, 237));
    }

    #[test]
    fn test_elapsed_time() {
        let indicator = ProgressIndicator::new("Test");
        std::thread::sleep(Duration::from_millis(100));
        let elapsed = indicator.get_elapsed_time();
        assert!(elapsed.as_millis() >= 100);
    }

    #[test]
    fn test_with_message() {
        let indicator = ProgressIndicator::new("Test").with_message("Custom message");
        assert_eq!(indicator.message.as_ref().unwrap(), "Custom message");
    }

    #[test]
    fn test_with_style() {
        let style = ProgressStyle::default();
        let indicator = ProgressIndicator::new("Test").with_style(style.clone());
        assert_eq!(indicator.style.bg, style.bg);
    }
}
