#![allow(dead_code)]

use crate::tui::components::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crate::tui::frame::{truncate_ansi_to_width, visible_width};
use crate::tui::layout::Rect;
use crate::tui::slash_complete::SlashCompletion;
use crate::tui::ui_palette::{input_bold, input_faint, input_line_padded, input_text};
use crossterm::event::KeyCode;

/// 输入行提示符可见宽度（`❯ `）。
pub(crate) const INPUT_PROMPT_WIDTH: u16 = 2;

/// Undo/Redo 栈深度上限。
const MAX_UNDO_DEPTH: usize = 50;

/// 输入框组件。
pub(crate) struct InputBar {
    /// 当前输入内容。
    content: String,
    /// 光标位置（字节偏移）。
    cursor: usize,
    /// Undo 栈 — 保存编辑前的 (content, cursor) 快照。
    undo_stack: Vec<(String, usize)>,
    /// Redo 栈 — undo 后可重做的快照。
    redo_stack: Vec<(String, usize)>,
    state: ComponentState,
}

impl InputBar {
    pub(crate) fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            state: ComponentState::new(generate_component_id("input_bar")),
        }
    }

    /// 保存当前状态到 undo 栈，清空 redo 栈。
    fn save_undo(&mut self) {
        if self.undo_stack.len() >= MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push((self.content.clone(), self.cursor));
        self.redo_stack.clear();
    }

    /// 撤销上一次编辑。返回 `true` 如果成功。
    pub(crate) fn undo(&mut self) -> bool {
        if let Some((content, cursor)) = self.undo_stack.pop() {
            if self.redo_stack.len() >= MAX_UNDO_DEPTH {
                self.redo_stack.remove(0);
            }
            self.redo_stack
                .push((std::mem::take(&mut self.content), self.cursor));
            self.content = content;
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    /// 重做上一次撤销。返回 `true` 如果成功。
    pub(crate) fn redo(&mut self) -> bool {
        if let Some((content, cursor)) = self.redo_stack.pop() {
            if self.undo_stack.len() >= MAX_UNDO_DEPTH {
                self.undo_stack.remove(0);
            }
            self.undo_stack
                .push((std::mem::take(&mut self.content), self.cursor));
            self.content = content;
            self.cursor = cursor;
            true
        } else {
            false
        }
    }

    /// 输入字符。
    pub(crate) fn insert(&mut self, ch: char) {
        self.save_undo();
        self.content.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    /// 删除光标前一个字符。
    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.save_undo();
            let prev = self.content[..self.cursor]
                .chars()
                .last()
                .map_or(0, char::len_utf8);
            let new_cursor = self.cursor - prev;
            self.content.drain(new_cursor..self.cursor);
            self.cursor = new_cursor;
        }
    }

    /// 删除光标后一个字符。
    pub(crate) fn delete(&mut self) {
        if self.cursor < self.content.len() {
            self.save_undo();
            let next_len = self.content[self.cursor..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
            self.content.drain(self.cursor..self.cursor + next_len);
        }
    }

    /// 光标左移一个字符。
    pub(crate) fn move_left(&mut self) {
        if self.cursor > 0 {
            let prev = self.content[..self.cursor]
                .chars()
                .last()
                .map_or(0, char::len_utf8);
            self.cursor -= prev;
        }
    }

    /// 光标右移一个字符。
    pub(crate) fn move_right(&mut self) {
        if self.cursor < self.content.len() {
            let next = self.content[self.cursor..]
                .chars()
                .next()
                .map_or(0, char::len_utf8);
            self.cursor += next;
        }
    }

    pub(crate) fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub(crate) fn move_end(&mut self) {
        self.cursor = self.content.len();
    }

    /// 取出当前内容并清空。
    pub(crate) fn take(&mut self) -> String {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.cursor = 0;
        std::mem::take(&mut self.content)
    }

    /// 当前内容（只读）。
    pub(crate) fn content(&self) -> &str {
        &self.content
    }

    /// 是否为空。
    pub(crate) fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// 光标在输入行内的可见列偏移（不含提示符）。
    pub(crate) fn cursor_visible_col(&self) -> u16 {
        let end = self.cursor.min(self.content.len());
        visible_width(&self.content[..end])
    }

    /// 输入行在输入块内的行索引（补全菜单 + 分隔线之后）。
    pub(crate) fn input_line_index(completion: Option<&SlashCompletion>) -> u16 {
        completion
            .map(|menu| menu.matches.len().min(6))
            .unwrap_or(0) as u16
            + 1
    }

    /// 设置内容（外部粘贴等）。
    pub(crate) fn set_content(&mut self, text: String) {
        self.save_undo();
        self.cursor = text.len();
        self.content = text;
    }

    /// 渲染为 ANSI 字符串（含可选斜杠补全菜单）。
    pub(crate) fn render(&self, area: Rect) -> String {
        self.render_with_completion(area, None)
    }

    pub(crate) fn render_with_completion(
        &self,
        area: Rect,
        completion: Option<&SlashCompletion>,
    ) -> String {
        self.render_with_options(area, completion, false)
    }

    pub(crate) fn render_plain(&self, area: Rect, completion: Option<&SlashCompletion>) -> String {
        self.render_with_options(area, completion, true)
    }

    fn render_with_options(
        &self,
        area: Rect,
        completion: Option<&SlashCompletion>,
        plain: bool,
    ) -> String {
        if !area.is_valid() {
            return String::new();
        }
        let width = area.width as usize;
        let mut lines: Vec<String> = Vec::new();

        if let Some(menu) = completion {
            lines.extend(menu.render_menu_lines(width));
        }

        let dashes = "─".repeat(width);
        let rule = input_line_padded(&input_faint(&dashes), width);
        lines.push(rule);

        let prompt = if plain {
            input_text("> ")
        } else {
            input_bold("❯ ")
        };
        let body = if self.content.is_empty() {
            input_faint("在此输入消息…")
        } else {
            let mut joined = String::new();
            for (index, line) in self.content.lines().enumerate() {
                if index > 0 {
                    joined.push('\n');
                    joined.push_str("  ");
                }
                joined.push_str(line);
            }
            input_text(&joined)
        };
        let input_line = input_line_padded(
            &truncate_ansi_to_width(&format!("{prompt}{body}"), width),
            width,
        );
        lines.push(input_line);

        let hint = completion
            .map(SlashCompletion::hint_line)
            .unwrap_or_else(|| "Shift+Enter 换行 · Enter 发送 · Tab 补全 · Esc 取消".to_string());
        lines.push(input_line_padded(
            &truncate_ansi_to_width(&input_faint(&hint), width),
            width,
        ));

        lines.join("\n")
    }
}

impl Component for InputBar {
    fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.state.visible {
            return;
        }
        use crate::tui::widgets::input_bar_ratatui::InputBarWidget;
        use ratatui::prelude::Widget;
        InputBarWidget {
            content: &self.content,
            slash_completion_count: 0,
            slash_completion: None,
        }
        .render(area, buf);
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }
        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Char(c) => {
                    self.insert(c);
                    ActionResult::Handled
                }
                KeyCode::Backspace => {
                    self.backspace();
                    ActionResult::Handled
                }
                KeyCode::Delete => {
                    self.delete();
                    ActionResult::Handled
                }
                KeyCode::Left => {
                    self.move_left();
                    ActionResult::Handled
                }
                KeyCode::Right => {
                    self.move_right();
                    ActionResult::Handled
                }
                KeyCode::Home => {
                    self.move_home();
                    ActionResult::Handled
                }
                KeyCode::End => {
                    self.move_end();
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
            Event::Input(InputEvent::Paste(text)) => {
                self.set_content(text.clone());
                ActionResult::Handled
            }
            _ => ActionResult::Ignored,
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    fn on_resize(&mut self, area: ratatui::layout::Rect) {
        self.state.bounds = area;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_bar_insert_and_backspace() {
        let mut bar = InputBar::new();
        bar.insert('a');
        bar.insert('b');
        bar.insert('c');
        assert_eq!(bar.content(), "abc");
        bar.backspace();
        assert_eq!(bar.content(), "ab");
    }

    #[test]
    fn input_bar_render_minimal_style() {
        let mut bar = InputBar::new();
        bar.set_content("hello".to_string());
        let rendered = bar.render(Rect::new(0, 0, 40, 3));
        assert!(rendered.contains("hello"));
        assert!(rendered.contains('─'));
        assert!(!rendered.contains('╭'));
    }

    #[test]
    fn input_bar_home_and_end() {
        let mut bar = InputBar::new();
        bar.insert('a');
        bar.insert('b');
        bar.insert('c');
        assert_eq!(bar.cursor, 3);
        bar.move_home();
        assert_eq!(bar.cursor, 0);
        bar.move_end();
        assert_eq!(bar.cursor, 3);
    }
}
