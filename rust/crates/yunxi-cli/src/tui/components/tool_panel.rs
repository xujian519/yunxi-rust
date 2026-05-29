#![allow(dead_code)]

use crate::tui::layout::Rect;

/// 工具调用记录。
#[derive(Debug, Clone)]
pub(crate) struct ToolEntry {
    pub name: String,
    pub detail: String,
    pub is_error: bool,
    pub collapsed: bool,
}

/// 工具输出面板。
pub(crate) struct ToolPanel {
    entries: Vec<ToolEntry>,
    scroll_offset: usize,
}

impl ToolPanel {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            scroll_offset: 0,
        }
    }

    /// 追加工具条目。
    pub(crate) fn push(&mut self, entry: ToolEntry) {
        self.entries.push(entry);
    }

    /// 折叠/展开指定条目。
    pub(crate) fn toggle_collapse(&mut self, index: usize) {
        if let Some(entry) = self.entries.get_mut(index) {
            entry.collapsed = !entry.collapsed;
        }
    }

    /// 向下滚动。
    pub(crate) fn scroll_down(&mut self, visible_lines: usize, wrap_width: usize) {
        self.scroll_down_by(1, visible_lines, wrap_width);
    }

    /// 向上滚动。
    pub(crate) fn scroll_up(&mut self) {
        self.scroll_up_by(1);
    }

    /// 向下滚动多行。
    pub(crate) fn scroll_down_by(
        &mut self,
        amount: usize,
        visible_lines: usize,
        wrap_width: usize,
    ) {
        let max = self.total_lines(wrap_width).saturating_sub(visible_lines);
        self.scroll_offset = (self.scroll_offset + amount).min(max);
    }

    /// 向上滚动多行。
    pub(crate) fn scroll_up_by(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// 根据渲染行号定位工具条目（含 scroll 偏移后的绝对行号）。
    pub(crate) fn entry_at_rendered_line(&self, line: usize, wrap_width: usize) -> Option<usize> {
        let mut offset = 0;
        for (idx, entry) in self.entries.iter().enumerate() {
            let lines = self.entry_line_count(entry, wrap_width);
            if line < offset + lines {
                return Some(idx);
            }
            offset += lines;
        }
        None
    }

    fn entry_line_count(&self, entry: &ToolEntry, wrap_width: usize) -> usize {
        let mut count = 1;
        if entry.collapsed {
            count += 1;
        } else {
            let inner_width = wrap_width.saturating_sub(2);
            for line in entry.detail.lines() {
                let wrapped = crate::tui::components::chat_view::wrap_line(line, inner_width);
                count += wrapped.len().max(1);
            }
            if entry.detail.is_empty() {
                count += 0;
            }
        }
        count
    }

    /// 估算总行数。
    fn total_lines(&self, wrap_width: usize) -> usize {
        self.entries
            .iter()
            .map(|e| self.entry_line_count(e, wrap_width))
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
            let icon = if entry.is_error {
                "\x1b[31m✗\x1b[0m"
            } else {
                "\x1b[32m✓\x1b[0m"
            };

            let name_truncated = truncate_str(&entry.name, width.saturating_sub(4));
            lines.push(format!("{icon} {name_truncated}"));

            if entry.collapsed {
                let line_count = entry.detail.lines().count();
                lines.push(format!("  \x1b[2m▸ 展开输出 ({line_count} 行)\x1b[0m"));
            } else {
                for line in entry.detail.lines() {
                    let wrapped =
                        crate::tui::components::chat_view::wrap_line(line, width.saturating_sub(2));
                    for wl in wrapped {
                        lines.push(format!("  {wl}"));
                    }
                }
            }
        }

        let visible_height = area.height as usize;
        let start = self
            .scroll_offset
            .min(lines.len().saturating_sub(visible_height));
        let end = std::cmp::min(start + visible_height, lines.len());

        if lines.is_empty() {
            "\x1b[2m工具输出面板 (F2 切换显示)\x1b[0m".to_string()
        } else {
            lines[start..end].join("\n")
        }
    }

    /// 条目数量。
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub(crate) fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// 是否为空。
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 清空。
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }

    /// 获取工具条目列表。
    pub(crate) fn entries(&self) -> &[ToolEntry] {
        &self.entries
    }
}

/// 截断字符串到指定字符宽度。
fn truncate_str(s: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut result = String::new();
    for ch in s.chars() {
        let cw = if ch.is_ascii() { 1 } else { 2 };
        if width + cw > max_width {
            result.push('…');
            break;
        }
        result.push(ch);
        width += cw;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_panel_push_and_render() {
        let mut panel = ToolPanel::new();
        panel.push(ToolEntry {
            name: "bash".to_string(),
            detail: "$ cargo test\nrunning 10 tests".to_string(),
            is_error: false,
            collapsed: false,
        });
        let rendered = panel.render(Rect::new(0, 0, 40, 10));
        assert!(rendered.contains("bash"));
        assert!(rendered.contains("cargo test"));
    }

    #[test]
    fn tool_panel_collapse() {
        let mut panel = ToolPanel::new();
        panel.push(ToolEntry {
            name: "read_file".to_string(),
            detail: "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11"
                .to_string(),
            is_error: false,
            collapsed: true,
        });
        let rendered = panel.render(Rect::new(0, 0, 40, 10));
        assert!(rendered.contains("展开输出"));
    }

    #[test]
    fn tool_panel_empty_message() {
        let panel = ToolPanel::new();
        let rendered = panel.render(Rect::new(0, 0, 40, 10));
        assert!(rendered.contains("工具输出面板"));
    }

    #[test]
    fn tool_panel_error_icon() {
        let mut panel = ToolPanel::new();
        panel.push(ToolEntry {
            name: "bash".to_string(),
            detail: "error".to_string(),
            is_error: true,
            collapsed: false,
        });
        let rendered = panel.render(Rect::new(0, 0, 40, 5));
        assert!(rendered.contains("✗"));
    }

    #[test]
    fn truncate_str_works() {
        assert_eq!(truncate_str("hello world", 5), "hello…");
        assert_eq!(truncate_str("hi", 10), "hi");
    }

    #[test]
    fn entry_at_rendered_line_finds_entry() {
        let mut panel = ToolPanel::new();
        panel.push(ToolEntry {
            name: "a".to_string(),
            detail: "one".to_string(),
            is_error: false,
            collapsed: false,
        });
        panel.push(ToolEntry {
            name: "b".to_string(),
            detail: "two".to_string(),
            is_error: false,
            collapsed: true,
        });
        assert_eq!(panel.entry_at_rendered_line(0, 40), Some(0));
        assert_eq!(panel.entry_at_rendered_line(2, 40), Some(1));
    }
}
