#![allow(dead_code)]

use crate::session_meta::SuspendedFlowRecord;
use crate::tui::frame::truncate_ansi_to_width;
use crate::tui::layout::Rect;

/// 工作流 HITL 确认覆盖层。
pub(crate) struct FlowHitlOverlay;

impl FlowHitlOverlay {
    pub(crate) fn render(area: Rect, record: &SuspendedFlowRecord) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let mut lines = vec![
            "\x1b[1;38;5;214m工作流等待人工确认\x1b[0m".to_string(),
            String::new(),
        ];

        if let Some(name) = &record.flow_name {
            lines.push(format!(
                "  流程       {}",
                truncate_plain(name, width.saturating_sub(12))
            ));
        }
        if let Some(title) = &record.step_title {
            lines.push(format!(
                "  检查点     {}",
                truncate_plain(title, width.saturating_sub(12))
            ));
        }
        if let Some(desc) = &record.step_description {
            lines.push(format!(
                "  说明       {}",
                truncate_plain(desc, width.saturating_sub(12))
            ));
        }
        if let Some(step) = record.current_step {
            lines.push(format!("  步骤序号   {step}"));
        }

        lines.push(format!(
            "  flow_id    {}",
            truncate_plain(&record.flow_id, width.saturating_sub(12))
        ));
        lines.push(format!(
            "  run_id     {}",
            truncate_plain(&record.run_id, width.saturating_sub(12))
        ));
        lines.push(String::new());
        lines.push(
            "\x1b[38;5;245m  y 恢复并继续 · n/Esc 稍后 · Ctrl+G 改道引导 · /flow resume\x1b[0m"
                .to_string(),
        );

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

    #[test]
    fn flow_hitl_overlay_renders_step_title() {
        let record = SuspendedFlowRecord {
            flow_id: "patent-review".to_string(),
            run_id: "run-abc".to_string(),
            noted_at: String::new(),
            flow_name: Some("专利答复".into()),
            current_step: Some(2),
            step_title: Some("确认检索结果".into()),
            step_description: Some("step-2".into()),
        };
        let rendered = FlowHitlOverlay::render(Rect::new(0, 0, 60, 12), &record);
        assert!(rendered.contains("确认检索结果"));
        assert!(rendered.contains("专利答复"));
        assert!(rendered.contains("y 恢复"));
    }
}
