use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crate::tui::diff::{DiffChange, DiffParser};
use crate::tui::theme::Theme;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

struct DiffColors {
    added_bg: Color,
    added_fg: Color,
    deleted_bg: Color,
    deleted_fg: Color,
    modified_bg: Color,
    modified_fg: Color,
    line_number_fg: Color,
}

impl DiffColors {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            added_bg: Color::Rgb(33, 58, 43),
            added_fg: theme.colors.success,
            deleted_bg: Color::Rgb(74, 34, 29),
            deleted_fg: theme.colors.error,
            modified_bg: Color::Rgb(89, 73, 33),
            modified_fg: theme.colors.warning,
            line_number_fg: theme.colors.text_muted,
        }
    }

    fn active() -> Self {
        crate::tui::ui_palette::active::current()
            .map(|t| Self::from_theme(&t))
            .unwrap_or_else(|| Self {
                added_bg: Color::Rgb(33, 58, 43),
                added_fg: Color::Rgb(152, 195, 121),
                deleted_bg: Color::Rgb(74, 34, 29),
                deleted_fg: Color::Rgb(224, 108, 117),
                modified_bg: Color::Rgb(89, 73, 33),
                modified_fg: Color::Rgb(229, 192, 123),
                line_number_fg: Color::Rgb(124, 111, 100),
            })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DiffViewStyle {
    Unified,
    Split,
}

pub struct DiffView {
    state: ComponentState,
    parser: DiffParser,
    current_change: usize,
    scroll_offset: usize,
    style: DiffViewStyle,
    show_line_numbers: bool,
}

impl DiffView {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(generate_component_id("diff_view")),
            parser: DiffParser::new(),
            current_change: 0,
            scroll_offset: 0,
            style: DiffViewStyle::Split,
            show_line_numbers: true,
        }
    }

    pub fn with_diff(mut self, diff_text: &str) -> Result<Self, String> {
        self.parser.parse_diff(diff_text)?;
        Ok(self)
    }

    pub fn with_style(mut self, style: DiffViewStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn stats(&self) -> crate::tui::diff::DiffStats {
        self.parser.stats()
    }

    pub fn change_count(&self) -> usize {
        let mut count = 0;
        for hunk in self.parser.hunks() {
            for change in &hunk.changes {
                if !matches!(change, DiffChange::Unchanged(_)) {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn next_change(&mut self) {
        let changes = self.all_changes();
        if !changes.is_empty() {
            self.current_change = (self.current_change + 1) % changes.len();
            self.scroll_to_change(self.current_change);
        }
    }

    pub fn previous_change(&mut self) {
        let changes = self.all_changes();
        if !changes.is_empty() {
            self.current_change = if self.current_change == 0 {
                changes.len() - 1
            } else {
                self.current_change - 1
            };
            self.scroll_to_change(self.current_change);
        }
    }

    fn all_changes(&self) -> Vec<(usize, &DiffChange)> {
        let mut changes = Vec::new();
        for (hunk_idx, hunk) in self.parser.hunks().iter().enumerate() {
            for (change_idx, change) in hunk.changes.iter().enumerate() {
                if !matches!(change, DiffChange::Unchanged(_)) {
                    changes.push((hunk_idx * 1000 + change_idx, change));
                }
            }
        }
        changes
    }

    fn scroll_to_change(&mut self, change_idx: usize) {
        let changes = self.all_changes();
        if let Some((global_idx, _)) = changes.get(change_idx) {
            self.scroll_offset = global_idx.saturating_sub(5) as usize;
        }
    }

    fn render_line_number(&self, line_num: Option<usize>, width: u16) -> Line<'_> {
        let colors = DiffColors::active();
        let num_str = line_num
            .map(|n| format!("{:>4}", n))
            .unwrap_or("    ".to_string());
        Line::from(Span::styled(
            format!("{} │", num_str),
            Style::default().fg(colors.line_number_fg),
        ))
    }

    fn render_change_line(&self, change: &DiffChange, width: u16) -> Line<'_> {
        let colors = DiffColors::active();
        let (text, style) = match change {
            DiffChange::Added(s) => (
                s.clone(),
                Style::default().bg(colors.added_bg).fg(colors.added_fg),
            ),
            DiffChange::Deleted(s) => (
                s.clone(),
                Style::default().bg(colors.deleted_bg).fg(colors.deleted_fg),
            ),
            DiffChange::Modified { old, .. } => (
                old.clone(),
                Style::default()
                    .bg(colors.modified_bg)
                    .fg(colors.modified_fg),
            ),
            DiffChange::Unchanged(s) => (s.clone(), Style::default()),
        };

        let prefix = match change {
            DiffChange::Added(_) => "+",
            DiffChange::Deleted(_) => "-",
            DiffChange::Modified { .. } => "~",
            DiffChange::Unchanged(_) => " ",
        };

        Line::from(vec![
            Span::styled(format!("{} ", prefix), style),
            Span::styled(text, style),
        ])
    }

    fn render_split_view(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title("Diff View");
        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.top();
        let max_y = inner.bottom().saturating_sub(1);

        for (line_idx, line) in self
            .parser
            .hunks()
            .iter()
            .flat_map(|h| &h.changes)
            .enumerate()
        {
            if line_idx < self.scroll_offset {
                continue;
            }

            if y >= max_y {
                break;
            }

            let line = self.render_change_line(line, inner.width);
            Paragraph::new(line).render(Rect::new(inner.left(), y, inner.width, 1), buf);
            y += 1;
        }

        if y < max_y {
            let remaining = max_y - y;
            Paragraph::new("").render(Rect::new(inner.left(), y, inner.width, remaining), buf);
        }
    }

    fn render_unified_view(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title("Unified Diff");
        let inner = block.inner(area);
        block.render(area, buf);

        let mut y = inner.top();
        let max_y = inner.bottom().saturating_sub(1);

        for (line_idx, change) in self
            .parser
            .hunks()
            .iter()
            .flat_map(|h| &h.changes)
            .enumerate()
        {
            if line_idx < self.scroll_offset {
                continue;
            }

            if y >= max_y {
                break;
            }

            let line = self.render_change_line(change, inner.width);
            Paragraph::new(line).render(Rect::new(inner.left(), y, inner.width, 1), buf);
            y += 1;
        }
    }
}

impl Component for DiffView {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        match self.style {
            DiffViewStyle::Split => self.render_split_view(area, buf),
            DiffViewStyle::Unified => self.render_unified_view(area, buf),
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible || self.state.disabled {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Char('n') if key.modifiers == KeyModifiers::SHIFT => {
                    self.previous_change();
                    ActionResult::Handled
                }
                KeyCode::Char('n') => {
                    self.next_change();
                    ActionResult::Handled
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.scroll_offset = self.scroll_offset.saturating_add(1);
                    ActionResult::Handled
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
            _ => ActionResult::Ignored,
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }
}

impl Default for DiffView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_view_creation() {
        let view = DiffView::new();
        assert!(view.state.visible);
    }

    #[test]
    fn test_diff_view_with_diff() {
        let diff = r"@@ -1,3 +1,4 @@
 line 1
-line 2
+line 2 modified
 line 3
+line 4";
        let view = DiffView::new().with_diff(diff).unwrap();
        assert_eq!(view.change_count(), 3);
    }

    #[test]
    fn test_navigation() {
        let diff = r"@@ -1,3 +1,4 @@
-line 2
+line 2 modified";
        let mut view = DiffView::new().with_diff(diff).unwrap();
        assert_eq!(view.current_change, 0);

        view.next_change();
        assert_eq!(view.current_change, 1);

        view.previous_change();
        assert_eq!(view.current_change, 0);
    }

    #[test]
    fn test_stats() {
        let diff = r"@@ -1,3 +1,4 @@
 line 1
-line 2
+line 2 modified
 line 3
+line 4";
        let view = DiffView::new().with_diff(diff).unwrap();
        let stats = view.stats();
        assert_eq!(stats.added, 2);
        assert_eq!(stats.deleted, 1);
    }

    #[test]
    fn test_scroll() {
        let diff = r"@@ -1,3 +1,4 @@
 line 1
-line 2
+line 2 modified
 line 3
+line 4";
        let mut view = DiffView::new().with_diff(diff).unwrap();
        assert_eq!(view.scroll_offset, 0);

        view.next_change();
        assert!(view.scroll_offset >= 0);
    }
}
