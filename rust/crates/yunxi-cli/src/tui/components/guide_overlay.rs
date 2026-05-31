//! 人机引导迷你面板（Ctrl+G）。

use crate::tui::frame::truncate_ansi_to_width;
use crate::tui::layout::Rect;

/// 预填到输入框的引导模板（用户可编辑后 Enter 发送）。
pub(crate) const GUIDE_INPUT_TEMPLATE: &str = "\
【人机引导】请暂停当前阶段，按以下要求重新推进：

1. （在此说明要修改/补充的要点）
2. （可选：文件路径，或 /import、/preview 等命令）
";

/// 人机引导覆盖层（显示在输入区上方）。
pub(crate) struct GuideOverlay;

impl GuideOverlay {
    pub(crate) fn render(area: Rect, thinking: bool) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let status = if thinking {
            "当前有进行中的轮次；发送后将尝试中断并注入新指示"
        } else {
            "在底栏编辑模板后 Enter 发送；Esc 关闭本面板"
        };

        let lines = vec![
            "\x1b[1;38;5;183m人机引导 (Ctrl+G)\x1b[0m".to_string(),
            String::new(),
            "\x1b[38;5;252m  用于在关键节点主动打断、纠正方向或补充材料。\x1b[0m".to_string(),
            format!("\x1b[38;5;245m  {status}\x1b[0m"),
            String::new(),
            "\x1b[38;5;213m  Ctrl+G\x1b[0m  打开/刷新模板".to_string(),
            "\x1b[38;5;213m  Ctrl+I\x1b[0m  中断轮次并打开引导".to_string(),
            "\x1b[38;5;213m  Ctrl+U\x1b[0m  预填 /import 导入材料".to_string(),
            "\x1b[38;5;213m  y / n\x1b[0m     工作流挂起时：继续 / 稍后".to_string(),
            "\x1b[38;5;213m  Esc\x1b[0m      关闭本面板".to_string(),
        ];

        let visible = area.height as usize;
        let end = visible.min(lines.len());
        lines[..end]
            .iter()
            .map(|line| truncate_ansi_to_width(line, width))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guide_overlay_renders_title() {
        let rendered = GuideOverlay::render(Rect::new(0, 0, 70, 8), false);
        assert!(rendered.contains("人机引导"));
        assert!(rendered.contains("Ctrl+G"));
    }

    #[test]
    fn guide_template_has_marker() {
        assert!(GUIDE_INPUT_TEMPLATE.contains("【人机引导】"));
    }
}
