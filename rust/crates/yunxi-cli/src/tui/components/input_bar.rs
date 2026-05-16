#![allow(dead_code)]

use crate::tui::layout::Rect;

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

    /// 设置内容（外部粘贴等）。
    pub(crate) fn set_content(&mut self, text: String) {
        self.cursor = text.len();
        self.content = text;
    }

    /// 渲染为 ANSI 字符串。
    pub(crate) fn render(&self, area: Rect) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let prompt = "> ";
        let prompt_len = prompt.len();
        let available = width.saturating_sub(prompt_len);

        // 显示光标附近的文本
        let display_start = self.cursor.saturating_sub(available);
        let display_end = std::cmp::min(display_start + available, self.content.len());
        let visible = &self.content[display_start..display_end];

        format!(
            "\x1b[38;5;213m{prompt}\x1b[0m{visible}\n\
             \x1b[2mShift+Enter 换行 · Enter 发送 · Esc 取消\x1b[0m"
        )
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
    fn input_bar_cursor_movement() {
        let mut bar = InputBar::new();
        bar.insert('x');
        bar.insert('y');
        bar.move_left();
        bar.insert('z');
        assert_eq!(bar.content(), "xzy");
    }

    #[test]
    fn input_bar_take_clears() {
        let mut bar = InputBar::new();
        bar.insert('h');
        bar.insert('i');
        let text = bar.take();
        assert_eq!(text, "hi");
        assert!(bar.is_empty());
    }

    #[test]
    fn input_bar_delete() {
        let mut bar = InputBar::new();
        bar.set_content("abc".to_string());
        bar.move_left();
        // cursor at 'c', delete 'c'
        bar.delete();
        assert_eq!(bar.content(), "ab");
    }

    #[test]
    fn input_bar_render() {
        let mut bar = InputBar::new();
        bar.set_content("hello".to_string());
        let rendered = bar.render(Rect::new(0, 0, 40, 3));
        assert!(rendered.contains("hello"));
        assert!(rendered.contains("Shift+Enter"));
    }
}
