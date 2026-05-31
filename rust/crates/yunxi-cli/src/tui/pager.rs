//! TUI 内可滚动分页器（长输出）。

use crate::render::TerminalRenderer;
use crate::tui::components::chat_view::wrap_line;
use crate::tui::frame::wrap_ansi_to_width;

/// 分页器状态。
#[derive(Debug, Clone)]
pub(crate) struct Pager {
    lines: Vec<String>,
    scroll_offset: usize,
    title: String,
}

impl Pager {
    pub(crate) fn new(title: impl Into<String>, text: &str, width: usize) -> Self {
        let width = width.max(20);
        let body = if text.contains("\x1b[") {
            text.to_string()
        } else {
            TerminalRenderer::new().markdown_to_ansi(text)
        };
        let mut lines = Vec::new();
        for line in body.lines() {
            if line.is_empty() {
                lines.push(String::new());
            } else if line.contains('\x1b') {
                lines.extend(wrap_ansi_to_width(line, width));
            } else {
                lines.extend(wrap_line(line, width));
            }
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
        Self {
            lines,
            scroll_offset: 0,
            title: title.into(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub(crate) fn scroll_down(&mut self, amount: usize, visible_lines: usize) {
        let max = self.lines.len().saturating_sub(visible_lines);
        self.scroll_offset = (self.scroll_offset + amount).min(max);
    }

    pub(crate) fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub(crate) fn scroll_to_bottom(&mut self, visible_lines: usize) {
        let max = self.lines.len().saturating_sub(visible_lines);
        self.scroll_offset = max;
    }

    #[must_use]
    pub(crate) fn lines(&self) -> &[String] {
        &self.lines
    }

    #[must_use]
    pub(crate) fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// 分页器全文（去 ANSI），供剪贴板使用。
    #[must_use]
    pub(crate) fn plain_text(&self) -> String {
        use crate::tui::clipboard::strip_ansi;

        self.lines
            .iter()
            .map(|line| strip_ansi(line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub(crate) fn render(&self, visible_lines: usize) -> String {
        let visible_lines = visible_lines.max(1);
        let start = self
            .scroll_offset
            .min(self.lines.len().saturating_sub(visible_lines));
        let end = (start + visible_lines).min(self.lines.len());
        let mut out = vec![format!(
            "\x1b[1;38;5;183m{}\x1b[0m  \x1b[2m(j/k 滚动 · q 关闭 · {}/{} 行)\x1b[0m",
            self.title,
            start + 1,
            self.lines.len()
        )];
        out.extend(self.lines[start..end].iter().cloned());
        out.push("\x1b[2m— 分页器 —\x1b[0m".to_string());
        out.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pager_wraps_and_scrolls() {
        let mut pager = Pager::new("Status", "line one\nline two\nline three", 10);
        assert!(pager.line_count() >= 3);
        pager.scroll_down(1, 2);
        assert_eq!(pager.scroll_offset, 1);
        let rendered = pager.render(2);
        assert!(rendered.contains("分页器"));
    }
}
