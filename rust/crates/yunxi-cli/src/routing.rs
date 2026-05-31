//! Workflow 路由：状态栏、工具白名单合并、轮次上下文

use std::collections::BTreeSet;

use embedding::semantic_enabled;
use knowledge::KnowledgePaths;
use router::types::{RoutingDecision, WorkflowType};
use router::workflow_router::WorkflowRouter;
use tools::mvp_tool_specs;

use crate::cli_action::AllowedToolSet;

/// 最近一次路由快照（供 `/status` 与状态栏）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoutingSnapshot {
    pub label: String,
    pub reasoning: String,
    pub suggested_tools: Vec<String>,
    pub confidence: f64,
    #[serde(default)]
    pub intent_name: String,
    #[serde(default)]
    pub intent_confidence: f64,
}

impl From<RoutingDecision> for RoutingSnapshot {
    fn from(d: RoutingDecision) -> Self {
        Self::from_decision(&d)
    }
}

impl RoutingSnapshot {
    pub(crate) fn from_decision(d: &RoutingDecision) -> Self {
        Self {
            label: format_route_label(d),
            reasoning: d.reasoning.clone(),
            suggested_tools: d.suggested_tools.clone(),
            confidence: d.confidence,
            intent_name: d.intent_name.clone(),
            intent_confidence: d.intent_confidence,
        }
    }
}

/// 对用户输入做 workflow 路由。
#[must_use]
pub fn route(input: &str) -> RoutingDecision {
    WorkflowRouter::default().route(input.trim())
}

/// 状态栏短标签（如 `专利·复杂·规划+人机`）。
#[must_use]
#[allow(dead_code)]
pub fn route_status_line(input: &str) -> Option<String> {
    let text = input.trim();
    if text.is_empty() {
        return None;
    }
    Some(format_route_label(&route(text)))
}

#[must_use]
pub fn format_route_label(d: &RoutingDecision) -> String {
    format!(
        "{}·{}·{}",
        d.domain,
        d.complexity,
        workflow_label(d.workflow)
    )
}

fn workflow_label(w: WorkflowType) -> &'static str {
    match w {
        WorkflowType::Direct => "直接",
        WorkflowType::Hitl => "人机协同",
        WorkflowType::PlanPlusHitl => "规划+人机",
    }
}

/// 将路由推荐工具并入白名单；若新增工具返回 `true`（`None` 表示未限制，无需合并）。
pub fn merge_suggested_tools(
    allowed: &mut Option<AllowedToolSet>,
    decision: &RoutingDecision,
) -> bool {
    let Some(set) = allowed.as_mut() else {
        return false;
    };
    let canonical: BTreeSet<String> = mvp_tool_specs()
        .into_iter()
        .map(|spec| spec.name.to_string())
        .collect();
    let mut changed = false;
    for name in &decision.suggested_tools {
        if canonical.contains(name) && set.insert(name.clone()) {
            changed = true;
        }
    }
    if !decision.suggested_agents.is_empty()
        && canonical.contains("Agent")
        && set.insert("Agent".to_string())
    {
        changed = true;
    }
    changed
}

/// 注入到用户消息前的路由上下文（引导模型选用推荐工具）。
#[must_use]
pub fn routing_user_prefix(decision: &RoutingDecision) -> String {
    let tools = decision.suggested_tools.join(", ");
    let agents = if decision.suggested_agents.is_empty() {
        "无".to_string()
    } else {
        decision.suggested_agents.join("、")
    };
    let intent_line = if decision.intent_confidence > 0.3 {
        format!(
            "\n意图: {} ({:.0}%)",
            decision.intent_name,
            decision.intent_confidence * 100.0
        )
    } else {
        String::new()
    };
    format!(
        "<yunxi_routing>\n{}{}\n建议工具: {}\n建议子智能体: {}\n工作流: {}\n</yunxi_routing>\n\n",
        decision.reasoning,
        intent_line,
        tools,
        agents,
        workflow_label(decision.workflow)
    )
}

/// `/status` 中的 Athena / 语义 / 路由段落
#[must_use]
pub fn format_athena_status_section(
    routing: Option<&RoutingSnapshot>,
    allowed_tools: Option<&AllowedToolSet>,
) -> String {
    let paths = KnowledgePaths::discover();
    let sem = embedding::global::status_json();
    let sem_on = semantic_enabled();
    let index = paths.semantic_index_db.as_deref().unwrap_or("—");
    let whitelist = match allowed_tools {
        None => "全部工具".to_string(),
        Some(set) => format!("{} 项白名单", set.len()),
    };
    let mut lines = vec![
        "Athena / 知识库".to_string(),
        format!(
            "  语义嵌入       {} (backend: {})",
            if sem_on { "已启用" } else { "未启用" },
            sem.get("backend").and_then(|v| v.as_str()).unwrap_or("—")
        ),
        format!(
            "  嵌入可用       {}",
            sem.get("available")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        ),
        format!("  语义索引       {index}"),
        format!("  工具策略       {whitelist}"),
    ];
    if let Some(r) = routing {
        lines.push(format!("  最近路由       {}", r.label));
        lines.push(format!("  路由置信度     {:.0}%", r.confidence * 100.0));
        lines.push(format!("  推荐工具       {}", r.suggested_tools.join(", ")));
        lines.push(format!("  路由说明       {}", r.reasoning));
    } else {
        lines.push("  最近路由       （本轮尚未发送消息）".to_string());
    }
    lines.join("\n")
}

/// 路由决策 JSON（`/route` 调试）
#[must_use]
pub fn route_debug_json(input: &str) -> serde_json::Value {
    let decision = route(input);
    serde_json::json!({
        "domain": format!("{}", decision.domain),
        "complexity": format!("{}", decision.complexity),
        "workflow": workflow_label(decision.workflow),
        "confidence": decision.confidence,
        "reasoning": decision.reasoning,
        "suggestedTools": decision.suggested_tools,
        "suggestedAgents": decision.suggested_agents,
        "intentName": decision.intent_name,
        "intentConfidence": decision.intent_confidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_patent_query() {
        let line = route_status_line("请分析这件专利的新颖性和创造性").unwrap();
        assert!(line.contains('专'));
    }

    #[test]
    fn merge_adds_patent_tools_to_whitelist() {
        let mut allowed = Some(BTreeSet::from([("read_file".to_string())]));
        let decision = route("请分析权利要求1的新颖性和创造性");
        let changed = merge_suggested_tools(&mut allowed, &decision);
        assert!(changed);
        let set = allowed.expect("set");
        assert!(set.len() > 1, "expected patent tools merged: {set:?}");
    }

    #[test]
    fn merge_adds_agent_tool_when_agents_suggested() {
        let mut allowed = Some(BTreeSet::from(["read_file".to_string()]));
        let decision = route("撰写专利申请文件");
        if !decision.suggested_agents.is_empty() {
            let changed = merge_suggested_tools(&mut allowed, &decision);
            assert!(changed);
            assert!(allowed.unwrap().contains("Agent"));
        }
    }
}
