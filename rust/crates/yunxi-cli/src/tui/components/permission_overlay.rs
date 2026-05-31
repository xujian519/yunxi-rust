#![allow(dead_code)]

use runtime::PermissionRequest;

use crate::tui::frame::truncate_ansi_to_width;
use crate::tui::layout::Rect;

/// 权限确认覆盖层。
pub(crate) struct PermissionOverlay;

impl PermissionOverlay {
    pub(crate) fn render(area: Rect, request: &PermissionRequest) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let input_preview = truncate_plain(&request.input, width.saturating_sub(12));

        let lines = vec![
            "\x1b[1;38;5;214m工具调用需要确认\x1b[0m".to_string(),
            String::new(),
            format!("  工具       {}", request.tool_name),
            format!("  当前模式   {}", request.current_mode.as_str()),
            format!("  所需模式   {}", request.required_mode.as_str()),
            format!("  参数       {input_preview}"),
            String::new(),
            "\x1b[38;5;245m  y 允许 · n 拒绝 · Esc 拒绝\x1b[0m".to_string(),
        ];

        let visible = area.height as usize;
        let skip = lines.len().saturating_sub(visible) / 2;
        let end = std::cmp::min(skip + visible, lines.len());
        lines[skip..end]
            .iter()
            .map(|line| truncate_ansi_to_width(line, width))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn truncate_plain(text: &str, max: usize) -> String {
    let mut used = 0usize;
    let mut out = String::new();
    for ch in text.chars() {
        let cw = if ch.is_ascii() { 1 } else { 2 };
        if used + cw > max {
            out.push('…');
            break;
        }
        out.push(ch);
        used += cw;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::PermissionMode;

    #[test]
    fn permission_overlay_renders_tool_name() {
        let request = PermissionRequest {
            tool_name: "bash".to_string(),
            input: r#"{"command":"ls"}"#.to_string(),
            current_mode: PermissionMode::WorkspaceWrite,
            required_mode: PermissionMode::DangerFullAccess,
        };
        let rendered = PermissionOverlay::render(Rect::new(0, 0, 60, 10), &request);
        assert!(rendered.contains("bash"));
        assert!(rendered.contains("y 允许"));
    }
}
