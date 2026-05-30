#![allow(dead_code)]

use crate::tui::frame::{truncate_ansi_to_width, visible_width};
use crate::tui::layout::Rect;
use crate::tui::slash_complete::SlashCompletion;
use crate::tui::ui_palette::{input_bold, input_faint, input_line_padded, input_text};

/// 输入行提示符可见宽度（`❯ `）。
pub(crate) const INPUT_PROMPT_WIDTH: u16 = 2;

/// 输入框组件。
pub(crate) struct InputBar {
    /// 当前输入内容。
    content: String,
    /// 光标位置（字节偏移）。
    cursor: usize,
}

impl InputBar {
    pub(crate) fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
        }
    }

    /// 输入字符。
    pub(crate) fn insert(&mut self, ch: char) {
        self.content.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    /// 删除光标前一个字符。
    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
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
