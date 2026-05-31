#![allow(dead_code)]

use crate::tui::layout::Rect;

/// 帮助覆盖层。
pub(crate) struct HelpOverlay;

impl HelpOverlay {
    /// 快捷键列表。
    pub(crate) fn shortcuts() -> &'static [(&'static str, &'static str)] {
        &[
            ("Enter", "发送消息"),
            ("Shift+Enter", "换行"),
            ("Ctrl+C / Esc", "清空输入或退出"),
            ("Ctrl+G", "人机引导（预填模板）"),
            ("Ctrl+I", "中断轮次并打开引导"),
            ("Ctrl+U", "预填 /import 导入材料"),
            ("Ctrl+F", "预填 /search 检索对话"),
            ("Ctrl+H / F1", "显示帮助"),
            ("F2", "切换工具面板"),
            ("j / ↓", "向下滚动"),
            ("k / ↑", "向上滚动"),
            ("g", "滚动到顶部"),
            ("G", "滚动到底部"),
            ("/", "输入斜杠命令"),
            ("鼠标拖选", "选中文字后 Cmd/Ctrl+C 复制"),
            ("Ctrl+Shift+C", "复制对话到剪贴板"),
            ("q", "退出 TUI 模式"),
        ]
    }

    /// 渲染帮助覆盖层。
    pub(crate) fn render(area: Rect) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let mut lines = Vec::new();

        lines.push("\x1b[1m\x1b[38;5;183m云熙智能体 — 快捷键\x1b[0m".to_string());
        lines.push(String::new());

        for (key, desc) in Self::shortcuts() {
            let padding = 18usize.saturating_sub(key.len());
            let pad = " ".repeat(padding);
            let desc_truncated = if desc.len() > width.saturating_sub(22) {
                &desc[..width.saturating_sub(23)]
            } else {
                desc
            };
            lines.push(format!("  \x1b[38;5;213m{key}\x1b[0m{pad}{desc_truncated}"));
        }

        lines.push(String::new());
        lines.push("\x1b[2m斜杠命令：输入 /help 打开完整命令列表（分页器）\x1b[0m".to_string());
        lines.push("\x1b[2m按任意键关闭帮助\x1b[0m".to_string());

        // 居中到可用区域
        let visible = area.height as usize;
        let skip = lines.len().saturating_sub(visible) / 2;
        let end = std::cmp::min(skip + visible, lines.len());
        lines[skip..end].join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_overlay_renders() {
        let rendered = HelpOverlay::render(Rect::new(0, 0, 80, 40));
        assert!(rendered.contains("快捷键"));
        assert!(rendered.contains("发送消息"));
        assert!(rendered.contains("Shift+Enter"));
        assert!(rendered.contains("关闭帮助"));
    }

    #[test]
    fn help_overlay_invalid_area() {
        let rendered = HelpOverlay::render(Rect::ZERO);
        assert!(rendered.is_empty());
    }

    #[test]
    fn shortcuts_not_empty() {
        assert!(!HelpOverlay::shortcuts().is_empty());
    }
}
