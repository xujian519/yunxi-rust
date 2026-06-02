use super::collapsible::Collapsible;
use crate::tui::core::action::ActionResult;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ThinkingStep {
    pub step_number: usize,
    pub title: String,
    pub content: String,
    pub completed: bool,
}

impl ThinkingStep {
    pub fn new(step_number: usize, title: impl Into<String>) -> Self {
        Self {
            step_number,
            title: title.into(),
            content: String::new(),
            completed: false,
        }
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn with_completed(mut self, completed: bool) -> Self {
        self.completed = completed;
        self
    }
}

pub struct ThinkingBlock {
    title: String,
    steps: Vec<ThinkingStep>,
    collapsibles: Vec<Collapsible>,
    current_step: Option<usize>,
    auto_expand_completed: bool,
    style: ThinkingBlockStyle,
    on_step_complete: Option<Arc<dyn Fn(usize) -> ActionResult + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct ThinkingBlockStyle {
    pub bg: Color,
    pub fg: Color,
    pub title_style: Style,
    pub step_number_style: Style,
    pub step_title_style: Style,
    pub step_completed_style: Style,
    pub step_incomplete_style: Style,
    pub separator_color: Color,
}

impl Default for ThinkingBlockStyle {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(26, 35, 50),
            fg: Color::Rgb(232, 232, 237),
            title_style: Style::default()
                .fg(Color::Rgb(139, 176, 240))
                .add_modifier(Modifier::BOLD),
            step_number_style: Style::default()
                .fg(Color::Rgb(139, 176, 240))
                .add_modifier(Modifier::BOLD),
            step_title_style: Style::default().fg(Color::Rgb(232, 232, 237)),
            step_completed_style: Style::default().fg(Color::Rgb(123, 200, 156)),
            step_incomplete_style: Style::default().fg(Color::Rgb(232, 132, 124)),
            separator_color: Color::Rgb(42, 42, 58),
        }
    }
}

impl ThinkingBlock {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            steps: Vec::new(),
            collapsibles: Vec::new(),
            current_step: None,
            auto_expand_completed: true,
            style: ThinkingBlockStyle::default(),
            on_step_complete: None,
        }
    }

    pub fn with_steps(mut self, steps: Vec<ThinkingStep>) -> Self {
        self.steps = steps;
        self.rebuild_collapsibles();
        self
    }

    pub fn with_auto_expand(mut self, auto_expand: bool) -> Self {
        self.auto_expand_completed = auto_expand;
        self.rebuild_collapsibles();
        self
    }

    pub fn with_style(mut self, style: ThinkingBlockStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_on_step_complete<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize) -> ActionResult + Send + Sync + 'static,
    {
        self.on_step_complete = Some(Arc::new(callback));
        self
    }

    fn rebuild_collapsibles(&mut self) {
        self.collapsibles = self
            .steps
            .iter()
            .map(|step| {
                let status_indicator = if step.completed { "✓" } else { "○" };
                let title = format!("{}. {} {}", step.step_number, status_indicator, step.title);

                let mut collapsible = Collapsible::new(title)
                    .with_content(&step.content)
                    .with_expanded(self.auto_expand_completed && step.completed);

                collapsible
            })
            .collect();
    }

    pub fn add_step(&mut self, step: ThinkingStep) {
        self.steps.push(step.clone());
        self.rebuild_collapsibles();
    }

    pub fn update_step(
        &mut self,
        step_number: usize,
        title: Option<String>,
        content: Option<String>,
    ) {
        if let Some(step) = self.steps.get_mut(step_number) {
            if let Some(new_title) = title {
                step.title = new_title;
            }
            if let Some(new_content) = content {
                step.content = new_content;
            }
            self.rebuild_collapsibles();
        }
    }

    pub fn complete_step(&mut self, step_number: usize) -> ActionResult {
        if let Some(step) = self.steps.get_mut(step_number) {
            step.completed = true;
            self.rebuild_collapsibles();

            if let Some(callback) = &self.on_step_complete {
                return callback(step_number);
            }
        }
        ActionResult::Handled
    }

    pub fn uncomplete_step(&mut self, step_number: usize) -> ActionResult {
        if let Some(step) = self.steps.get_mut(step_number) {
            step.completed = false;
            self.rebuild_collapsibles();
        }
        ActionResult::Handled
    }

    pub fn set_current_step(&mut self, step_number: usize) {
        self.current_step = Some(step_number);
    }

    pub fn get_step(&self, step_number: usize) -> Option<&ThinkingStep> {
        self.steps.get(step_number)
    }

    pub fn get_steps(&self) -> &[ThinkingStep] {
        &self.steps
    }

    pub fn get_step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn is_all_completed(&self) -> bool {
        self.steps.iter().all(|step| step.completed)
    }

    pub fn get_completion_percentage(&self) -> f32 {
        if self.steps.is_empty() {
            return 0.0;
        }
        let completed = self.steps.iter().filter(|step| step.completed).count() as f32;
        (completed / self.steps.len() as f32) * 100.0
    }
}

impl Widget for ThinkingBlock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.steps.is_empty() {
            return;
        }

        let total_steps = self.steps.len() as u16;
        let step_height = area.height.saturating_sub(2) / total_steps.max(1);
        let mut current_y = area.y;

        let title_line = Line::from(vec![
            Span::styled("🧠 ", Style::default()),
            Span::styled(&self.title, self.style.title_style),
        ]);

        let paragraph = ratatui::widgets::Paragraph::new(title_line)
            .style(Style::default().bg(self.style.bg).fg(self.style.fg));
        paragraph.render(
            Rect {
                x: area.x,
                y: current_y,
                width: area.width,
                height: 1,
            },
            buf,
        );
        current_y += 1;

        for (i, collapsible) in self.collapsibles.iter().enumerate() {
            let collapsible_area = Rect {
                x: area.x,
                y: current_y,
                width: area.width,
                height: step_height.max(3),
            };

            collapsible.render(collapsible_area, buf);
            current_y += step_height;

            if i < self.collapsibles.len() - 1 && current_y < area.y + area.height {
                let separator = ratatui::widgets::Paragraph::new("─".repeat(area.width as usize))
                    .style(Style::default().fg(self.style.separator_color));
                separator.render(
                    Rect {
                        x: area.x,
                        y: current_y,
                        width: area.width,
                        height: 1,
                    },
                    buf,
                );
                current_y += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thinking_step_creation() {
        let step = ThinkingStep::new(1, "Step 1");
        assert_eq!(step.step_number, 1);
        assert_eq!(step.title, "Step 1");
        assert!(!step.completed);
    }

    #[test]
    fn test_thinking_step_with_content() {
        let step = ThinkingStep::new(1, "Step 1")
            .with_content("Test content")
            .with_completed(true);
        assert_eq!(step.content, "Test content");
        assert!(step.completed);
    }

    #[test]
    fn test_thinking_block_creation() {
        let block = ThinkingBlock::new("Thinking Process");
        assert_eq!(block.title, "Thinking Process");
        assert!(block.steps.is_empty());
    }

    #[test]
    fn test_thinking_block_with_steps() {
        let steps = vec![
            ThinkingStep::new(1, "Step 1"),
            ThinkingStep::new(2, "Step 2"),
        ];
        let block = ThinkingBlock::new("Thinking Process").with_steps(steps);
        assert_eq!(block.get_step_count(), 2);
    }

    #[test]
    fn test_add_step() {
        let mut block = ThinkingBlock::new("Thinking Process");
        block.add_step(ThinkingStep::new(1, "Step 1"));
        assert_eq!(block.get_step_count(), 1);
    }

    #[test]
    fn test_complete_step() {
        let mut block =
            ThinkingBlock::new("Thinking Process").with_steps(vec![ThinkingStep::new(1, "Step 1")]);

        assert!(!block.get_step(0).unwrap().completed);
        block.complete_step(0);
        assert!(block.get_step(0).unwrap().completed);
    }

    #[test]
    fn test_uncomplete_step() {
        let mut block = ThinkingBlock::new("Thinking Process")
            .with_steps(vec![ThinkingStep::new(1, "Step 1").with_completed(true)]);

        assert!(block.get_step(0).unwrap().completed);
        block.uncomplete_step(0);
        assert!(!block.get_step(0).unwrap().completed);
    }

    #[test]
    fn test_update_step() {
        let mut block =
            ThinkingBlock::new("Thinking Process").with_steps(vec![ThinkingStep::new(1, "Step 1")]);

        block.update_step(
            0,
            Some("Updated Step 1".to_string()),
            Some("Updated content".to_string()),
        );

        let step = block.get_step(0).unwrap();
        assert_eq!(step.title, "Updated Step 1");
        assert_eq!(step.content, "Updated content");
    }

    #[test]
    fn test_is_all_completed() {
        let steps = vec![
            ThinkingStep::new(1, "Step 1").with_completed(true),
            ThinkingStep::new(2, "Step 2").with_completed(true),
        ];
        let block = ThinkingBlock::new("Thinking Process").with_steps(steps);
        assert!(block.is_all_completed());
    }

    #[test]
    fn test_not_all_completed() {
        let steps = vec![
            ThinkingStep::new(1, "Step 1").with_completed(true),
            ThinkingStep::new(2, "Step 2"),
        ];
        let block = ThinkingBlock::new("Thinking Process").with_steps(steps);
        assert!(!block.is_all_completed());
    }

    #[test]
    fn test_completion_percentage() {
        let steps = vec![
            ThinkingStep::new(1, "Step 1").with_completed(true),
            ThinkingStep::new(2, "Step 2"),
            ThinkingStep::new(3, "Step 3").with_completed(true),
        ];
        let block = ThinkingBlock::new("Thinking Process").with_steps(steps);

        let percentage = block.get_completion_percentage();
        assert_eq!(percentage, 66.66667);
    }

    #[test]
    fn test_completion_percentage_empty() {
        let block = ThinkingBlock::new("Thinking Process");
        assert_eq!(block.get_completion_percentage(), 0.0);
    }

    #[test]
    fn test_set_current_step() {
        let mut block =
            ThinkingBlock::new("Thinking Process").with_steps(vec![ThinkingStep::new(1, "Step 1")]);

        block.set_current_step(0);
        assert_eq!(block.current_step, Some(0));
    }

    #[test]
    fn test_default_style() {
        let style = ThinkingBlockStyle::default();
        assert_eq!(style.bg, Color::Rgb(26, 35, 50));
        assert_eq!(style.fg, Color::Rgb(232, 232, 237));
    }
}
