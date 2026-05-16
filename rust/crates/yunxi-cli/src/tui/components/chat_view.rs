#![allow(dead_code)]

use crate::tui::layout::Rect;

/// 对话消息条目。
#[derive(Debug, Clone)]
pub(crate) struct ChatEntry {
    pub role: ChatRole,
    pub text: String,
}

/// 消息角色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChatRole {
    User,
    Assistant,
    System,
}

impl ChatRole {
    /// 显示标签。
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::User => "你",
            Self::Assistant => "云熙",
            Self::System => "系统",
        }
    }
}

/// 对话历史视图（可滚动）。
pub(crate) struct ChatView {
    entries: Vec<ChatEntry>,
    scroll_offset: usize,
}

impl ChatView {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            scroll_offset: 0,
        }
    }

    /// 追加一条消息。
    pub(crate) fn push(&mut self, entry: ChatEntry) {
        self.entries.push(entry);
    }

    /// 追加纯文本到最新 assistant 条目末尾（用于流式增量）。
    pub(crate) fn append_to_last(&mut self, text: &str) {
        if let Some(last) = self.entries.last_mut() {
            last.text.push_str(text);
        }
    }

    /// 向下滚动一行。
    pub(crate) fn scroll_down(&mut self, visible_lines: usize) {
        let max = self.total_lines().saturating_sub(visible_lines);
        if self.scroll_offset < max {
            self.scroll_offset += 1;
        }
    }

    /// 向上滚动一行。
    pub(crate) fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// 滚动到顶部。
    pub(crate) fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// 滚动到最新内容。
    pub(crate) fn scroll_to_bottom(&mut self, visible_lines: usize) {
        let max = self.total_lines().saturating_sub(visible_lines);
        self.scroll_offset = max;
    }

    /// 估算总行数（简单按换行符计数）。
    fn total_lines(&self) -> usize {
        self.entries
            .iter()
            .map(|e| {
                let line_count = e.text.lines().count().max(1);
                // +1 给角色标签行
                line_count + 1
            })
            .sum()
    }

    /// 渲染为 ANSI 字符串。
    pub(crate) fn render(&self, area: Rect) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let mut lines: Vec<String> = Vec::new();

        for entry in &self.entries {
            let label_color = match entry.role {
                ChatRole::User => "\x1b[38;5;183m",
                ChatRole::Assistant => "\x1b[38;5;213m",
                ChatRole::System => "\x1b[38;5;246m",
            };
            lines.push(format!(
                "{label_color}{}\x1b[0m: {}",
                entry.role.label(),
                entry.text.lines().next().unwrap_or("")
            ));
            for line in entry.text.lines().skip(1) {
                // 缩进对齐
                let indent = "  ";
                let wrapped = wrap_line(line, width.saturating_sub(2));
                for wl in wrapped {
                    lines.push(format!("{indent}{wl}"));
                }
            }
        }

        // 应用滚动偏移
        let visible_height = area.height as usize;
        let start = self
            .scroll_offset
            .min(lines.len().saturating_sub(visible_height));
        let end = std::cmp::min(start + visible_height, lines.len());

        lines[start..end].join("\n")
    }

    /// 条目数量。
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 清空所有条目。
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }
}

/// 简易文本折行。
pub(crate) fn wrap_line(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    let mut current = String::new();
    // 按字符宽度粗略计算（ASCII 字符宽度 1，CJK 宽度 2）
    let mut width: usize = 0;
    for ch in text.chars() {
        let cw = if ch.is_ascii() { 1 } else { 2 };
        if width + cw > max_width && !current.is_empty() {
            result.push(current.clone());
            current.clear();
            width = 0;
        }
        current.push(ch);
        width += cw;
    }
    if !current.is_empty() {
        result.push(current);
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_view_push_and_render() {
        let mut view = ChatView::new();
        view.push(ChatEntry {
            role: ChatRole::User,
            text: "你好".to_string(),
        });
        view.push(ChatEntry {
            role: ChatRole::Assistant,
            text: "你好！有什么可以帮你的？".to_string(),
        });
        assert_eq!(view.len(), 2);

        let rendered = view.render(Rect::new(0, 0, 40, 10));
        assert!(rendered.contains("你"));
        assert!(rendered.contains("云熙"));
    }

    #[test]
    fn chat_view_scroll() {
        let mut view = ChatView::new();
        for i in 0..50 {
            view.push(ChatEntry {
                role: ChatRole::User,
                text: format!("消息 {i}"),
            });
        }
        view.scroll_down(5);
        assert_eq!(view.scroll_offset, 1);
        view.scroll_up();
        assert_eq!(view.scroll_offset, 0);
        view.scroll_to_bottom(5);
        assert!(view.scroll_offset > 0);
    }

    #[test]
    fn chat_view_append_to_last() {
        let mut view = ChatView::new();
        view.push(ChatEntry {
            role: ChatRole::Assistant,
            text: "你好".to_string(),
        });
        view.append_to_last("世界");
        assert_eq!(view.entries[0].text, "你好世界");
    }

    #[test]
    fn wrap_line_splits_long_text() {
        let wrapped = wrap_line("abcdefghij", 5);
        assert_eq!(wrapped, vec!["abcde", "fghij"]);
    }

    #[test]
    fn wrap_line_handles_empty() {
        let wrapped = wrap_line("", 10);
        assert_eq!(wrapped, vec![""]);
    }
}
