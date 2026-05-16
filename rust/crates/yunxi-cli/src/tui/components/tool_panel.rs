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
    pub(crate) fn scroll_down(&mut self, visible_lines: usize) {
        let max = self.total_lines().saturating_sub(visible_lines);
        if self.scroll_offset < max {
            self.scroll_offset += 1;
        }
    }

    /// 向上滚动。
    pub(crate) fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// 估算总行数。
    fn total_lines(&self) -> usize {
        self.entries
            .iter()
            .map(|e| {
                if e.collapsed {
                    1
                } else {
                    e.detail.lines().count().max(1) + 1
                }
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

    /// 是否为空。
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 清空。
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
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
}
