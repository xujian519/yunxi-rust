//! Flow HITL 接线与用户主动干预（引导模板、消息包装）。

use crate::session_meta::flow_suspend_from_tool_output;
use crate::tui::app::TuiApp;
use crate::tui::components::guide_overlay::GUIDE_INPUT_TEMPLATE;
use crate::tui::runner::TuiState;

/// 从会话挂起列表同步到覆盖层（取最新一条）。
pub(crate) fn sync_pending_flow_overlay(app: &mut TuiApp, state: &mut TuiState) {
    for record in &mut state.suspended_flows {
        record.enrich_from_checkpoint();
    }
    let record = state.suspended_flows.last().cloned();
    app.set_pending_flow_hitl(record);
}

/// 处理工具结果中的 Flow 挂起。
pub(crate) fn ingest_flow_tool_result(
    app: &mut TuiApp,
    state: &mut TuiState,
    tool_name: &str,
    output: &str,
    is_error: bool,
) {
    if is_error || tool_name != "FlowTool" {
        return;
    }
    let Some(record) = flow_suspend_from_tool_output(output) else {
        return;
    };
    if let Some(existing) = state
        .suspended_flows
        .iter_mut()
        .find(|f| f.flow_id == record.flow_id && f.run_id == record.run_id)
    {
        *existing = record.clone();
    } else {
        state.suspended_flows.push(record.clone());
    }
    app.set_pending_flow_hitl(Some(record.clone()));
    let checkpoint = record.step_title.as_deref().unwrap_or("人工确认");
    app.push_system_message(&format!(
        "\x1b[38;5;214m工作流已挂起：{checkpoint}\x1b[0m 按 \x1b[38;5;213my\x1b[0m 继续，\
         \x1b[38;5;213mn\x1b[0m 稍后，\x1b[38;5;213mCtrl+G\x1b[0m 改道，或 /flow resume {} {}",
        record.flow_id, record.run_id
    ));
}

/// 预填 `/import` 便于快速附加案件材料。
pub(crate) fn open_import_prefill(app: &mut TuiApp) {
    app.set_show_guide(false);
    app.set_input_content("/import ".to_string());
}

/// 打开人机引导：预填模板并显示迷你面板。
pub(crate) fn open_human_guide(app: &mut TuiApp) {
    app.set_show_guide(true);
    if app.input_content().trim().is_empty() {
        app.set_input_content(GUIDE_INPUT_TEMPLATE.to_string());
    }
}

/// 关闭人机引导面板（不清空输入，便于继续编辑）。
pub(crate) fn close_human_guide(app: &mut TuiApp) {
    app.set_show_guide(false);
}

/// 用户主动干预消息包装（注入会话，提示模型暂停原路径）。
#[must_use]
pub(crate) fn wrap_human_intervention(user_text: &str, interrupted_turn: bool) -> String {
    let lead = if interrupted_turn {
        "用户中断了进行中的轮次，并提交以下新指示。请暂停原任务路径，按新指示重新引导。"
    } else {
        "用户主动发起人机引导。请按以下指示调整当前阶段的方向、材料或检查点。"
    };
    format!("<yunxi_human_intervention>\n{lead}\n</yunxi_human_intervention>\n\n{user_text}")
}

#[must_use]
pub(crate) fn is_human_intervention_message(text: &str) -> bool {
    text.contains("【人机引导】") || text.contains("<yunxi_human_intervention>")
}

/// Flow 覆盖层：y 恢复所需 ID。
#[must_use]
pub(crate) fn pending_flow_resume_ids(app: &TuiApp) -> Option<(String, String)> {
    app.pending_flow_hitl()
        .map(|r| (r.flow_id.clone(), r.run_id.clone()))
}

/// 清除覆盖层（挂起记录仍保留在会话，供 /flow resume）。
pub(crate) fn defer_flow_hitl_overlay(app: &mut TuiApp) {
    app.set_pending_flow_hitl(None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_intervention_includes_marker() {
        let wrapped = wrap_human_intervention("【人机引导】测试", true);
        assert!(wrapped.contains("<yunxi_human_intervention>"));
        assert!(wrapped.contains("中断"));
    }

    #[test]
    fn detects_guide_marker() {
        assert!(is_human_intervention_message("【人机引导】x"));
        assert!(!is_human_intervention_message("普通问题"));
    }
}
