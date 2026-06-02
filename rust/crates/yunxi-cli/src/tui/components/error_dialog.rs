use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, InputEvent};
use crate::tui::error::{ErrorReport, YunXiError};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarState, Wrap};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorDialogAction {
    Close,
    Copy,
    Send,
}

pub struct ErrorDialog {
    state: ComponentState,
    error_report: ErrorReport,
    show_backtrace: bool,
    scroll_offset: usize,
    scroll_state: ScrollbarState,
    selected_action: ErrorDialogAction,
}

impl ErrorDialog {
    pub fn new(error_report: ErrorReport) -> Self {
        let content_length = error_report.generate_text_report().lines().count();
        Self {
            state: ComponentState::new(generate_component_id("error_dialog")),
            error_report,
            show_backtrace: true,
            scroll_offset: 0,
            scroll_state: ScrollbarState::new(content_length),
            selected_action: ErrorDialogAction::Close,
        }
    }

    pub fn with_backtrace(mut self, show: bool) -> Self {
        self.show_backtrace = show;
        self
    }

    pub fn is_visible(&self) -> bool {
        self.state.visible
    }

    pub fn show(&mut self) {
        self.state.visible = true;
        self.scroll_offset = 0;
        self.selected_action = ErrorDialogAction::Close;
    }

    pub fn hide(&mut self) {
        self.state.visible = false;
    }

    pub fn toggle_backtrace(&mut self) {
        self.show_backtrace = !self.show_backtrace;
    }

    fn generate_display_content(&self) -> String {
        let mut content = self.error_report.generate_text_report();

        if !self.show_backtrace {
            let lines: Vec<&str> = content.lines().collect();
            let mut filtered = Vec::new();
            let mut in_backtrace = false;

            for line in lines {
                if line.contains("堆栈跟踪") {
                    in_backtrace = true;
                    continue;
                }
                if in_backtrace && line.contains("────────────────")
                {
                    in_backtrace = false;
                    filtered.push(line);
                    continue;
                }
                if !in_backtrace {
                    filtered.push(line);
                }
            }
            content = filtered.join("\n");
        }

        content
    }

    fn handle_key_event(&mut self, key: KeyCode) -> ActionResult {
        match key {
            KeyCode::Esc => {
                self.hide();
                ActionResult::Handled
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
                self.selected_action = ErrorDialogAction::Copy;
                ActionResult::Action(Action::Custom("copy_error".to_string()))
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.selected_action = ErrorDialogAction::Send;
                ActionResult::Action(Action::Custom("send_error".to_string()))
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                self.toggle_backtrace();
                ActionResult::Handled
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let content_lines = self.generate_display_content().lines().count();
                if self.scroll_offset < content_lines.saturating_sub(1) {
                    self.scroll_offset += 1;
                }
                ActionResult::Handled
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
                ActionResult::Handled
            }
            KeyCode::PageDown => {
                let content_lines = self.generate_display_content().lines().count();
                self.scroll_offset = self
                    .scroll_offset
                    .saturating_add(10)
                    .min(content_lines.saturating_sub(1));
                ActionResult::Handled
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                ActionResult::Handled
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
                ActionResult::Handled
            }
            KeyCode::End => {
                let content_lines = self.generate_display_content().lines().count();
                self.scroll_offset = content_lines.saturating_sub(1);
                ActionResult::Handled
            }
            _ => ActionResult::Ignored,
        }
    }

    fn get_level_color(&self) -> Color {
        match self.error_report.error.level {
            crate::tui::error::ErrorLevel::Info => Color::Rgb(139, 176, 240),
            crate::tui::error::ErrorLevel::Warning => Color::Rgb(240, 187, 139),
            crate::tui::error::ErrorLevel::Error => Color::Rgb(240, 139, 139),
            crate::tui::error::ErrorLevel::Fatal => Color::Rgb(255, 0, 0),
        }
    }
}

impl Component for ErrorDialog {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let content = self.generate_display_content();
        let lines: Vec<Line> = content.lines().map(Line::from).collect();
        let display_lines: Vec<Line> = lines.into_iter().skip(self.scroll_offset).collect();

        let error_style = Style::default().fg(self.get_level_color());
        let border_style = error_style.add_modifier(Modifier::BOLD);

        let content_widget = Paragraph::new(display_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(Span::styled("错误详情", error_style)),
            )
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset as u16, 0));

        content_widget.render(chunks[0], buf);

        let action_style = Style::default().fg(Color::Rgb(150, 150, 150));

        let mut action_spans = vec![
            Span::raw("操作: "),
            Span::styled("[B]", action_style),
            Span::raw("切换堆栈 "),
            Span::styled("[C]", action_style),
            Span::raw("复制 "),
            Span::styled("[S]", action_style),
            Span::raw("发送 "),
            Span::styled("[ESC]", action_style),
            Span::raw("关闭"),
        ];

        let actions_line = Line::from(action_spans);
        let actions_widget = Paragraph::new(actions_line).alignment(Alignment::Center);

        actions_widget.render(chunks[1], buf);

        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        let mut scrollbar_state = self.scroll_state.clone();
        scrollbar_state = scrollbar_state.position(self.scroll_offset);
        StatefulWidget::render(scrollbar.clone(), chunks[0], buf, &mut scrollbar_state);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(KeyEvent {
                code,
                modifiers: _,
                kind: _,
                state: _,
            })) => self.handle_key_event(*code),
            _ => ActionResult::Ignored,
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    fn on_resize(&mut self, area: Rect) {
        self.state.bounds = area;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::error::{ErrorLevel, ErrorType};

    #[test]
    fn test_error_dialog_creation() {
        let error = YunXiError::new(
            ErrorType::IO("测试".to_string()),
            ErrorLevel::Error,
            "测试错误",
        );
        let report = ErrorReport::new(error);
        let dialog = ErrorDialog::new(report);
        assert!(dialog.is_visible());
    }

    #[test]
    fn test_error_dialog_show_hide() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        assert!(dialog.is_visible());
        dialog.hide();
        assert!(!dialog.is_visible());
        dialog.show();
        assert!(dialog.is_visible());
    }

    #[test]
    fn test_error_dialog_toggle_backtrace() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        assert!(dialog.show_backtrace);
        dialog.toggle_backtrace();
        assert!(!dialog.show_backtrace);
        dialog.toggle_backtrace();
        assert!(dialog.show_backtrace);
    }

    #[test]
    fn test_error_dialog_with_backtrace() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let dialog = ErrorDialog::new(report).with_backtrace(false);
        assert!(!dialog.show_backtrace);
    }

    #[test]
    fn test_error_dialog_handle_esc() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        assert!(!dialog.is_visible());
    }

    #[test]
    fn test_error_dialog_handle_scroll_down() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        assert_eq!(dialog.scroll_offset, 1);
    }

    #[test]
    fn test_error_dialog_handle_scroll_up() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        dialog.scroll_offset = 5;
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        assert_eq!(dialog.scroll_offset, 4);
    }

    #[test]
    fn test_error_dialog_handle_page_down() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::PageDown,
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        assert_eq!(dialog.scroll_offset, 10);
    }

    #[test]
    fn test_error_dialog_handle_page_up() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        dialog.scroll_offset = 20;
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::PageUp,
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        assert_eq!(dialog.scroll_offset, 10);
    }

    #[test]
    fn test_error_dialog_handle_home() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        dialog.scroll_offset = 10;
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Home,
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        assert_eq!(dialog.scroll_offset, 0);
    }

    #[test]
    fn test_error_dialog_handle_end() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::End,
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        let content_length = dialog.generate_display_content().lines().count();
        assert_eq!(dialog.scroll_offset, content_length.saturating_sub(1));
    }

    #[test]
    fn test_error_dialog_handle_toggle_backtrace_key() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        let event = Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Char('b'),
            KeyModifiers::NONE,
        )));
        dialog.handle_event(&event);
        assert!(!dialog.show_backtrace);
    }

    #[test]
    fn test_error_dialog_get_level_color() {
        for level in [
            ErrorLevel::Info,
            ErrorLevel::Warning,
            ErrorLevel::Error,
            ErrorLevel::Fatal,
        ] {
            let error = YunXiError::new(ErrorType::Runtime("测试".to_string()), level, "测试");
            let report = ErrorReport::new(error);
            let dialog = ErrorDialog::new(report);
            let color = dialog.get_level_color();
            match level {
                ErrorLevel::Info => assert_eq!(color, Color::Rgb(139, 176, 240)),
                ErrorLevel::Warning => assert_eq!(color, Color::Rgb(240, 187, 139)),
                ErrorLevel::Error => assert_eq!(color, Color::Rgb(240, 139, 139)),
                ErrorLevel::Fatal => assert_eq!(color, Color::Rgb(255, 0, 0)),
            }
        }
    }

    #[test]
    fn test_error_dialog_generate_display_content() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let dialog = ErrorDialog::new(report);
        let content = dialog.generate_display_content();
        assert!(content.contains("错误报告"));
        assert!(content.contains("堆栈跟踪"));
    }

    #[test]
    fn test_error_dialog_generate_display_content_without_backtrace() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let dialog = ErrorDialog::new(report).with_backtrace(false);
        let content = dialog.generate_display_content();
        assert!(content.contains("错误报告"));
        assert!(!content.contains("堆栈跟踪"));
    }

    #[test]
    fn test_error_dialog_state_update_on_resize() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        let area = Rect::new(10, 10, 80, 20);
        dialog.on_resize(area);
        assert_eq!(dialog.state.bounds, area);
    }

    #[test]
    fn test_error_dialog_state_update_on_focus() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        dialog.on_focus(true);
        assert!(dialog.state.focused);
        dialog.on_focus(false);
        assert!(!dialog.state.focused);
    }

    #[test]
    fn test_error_dialog_scroll_boundaries() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let mut dialog = ErrorDialog::new(report);
        dialog.scroll_offset = 0;
        dialog.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE,
        ))));
        assert_eq!(dialog.scroll_offset, 0);

        let content_length = dialog.generate_display_content().lines().count();
        dialog.scroll_offset = content_length;
        dialog.handle_event(&Event::Input(InputEvent::Key(KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::NONE,
        ))));
        assert_eq!(dialog.scroll_offset, content_length);
    }

    #[test]
    fn test_error_dialog_id_generation() {
        let error = YunXiError::io("测试");
        let report = ErrorReport::new(error);
        let dialog = ErrorDialog::new(report);
        assert!(dialog.get_state().id.starts_with("error_dialog_"));
    }
}
