//! 宪法引擎 IPC 命令。

use constitutional_engine::{ConstitutionalEngine, RuleLoader};
use serde_json::Value;
use std::path::Path;

/// 执行合规性检查
#[tauri::command]
pub fn check_compliance(
    text: String,
    rule_types: Option<Vec<String>>,
    context: Option<String>,
) -> Result<String, String> {
    // 加载规则
    let rules_path = Path::new("assets/constitutional");
    let rules = RuleLoader::load_dir(&rules_path).map_err(|e| format!("规则加载失败: {e}"))?;

    // 创建引擎
    let engine = ConstitutionalEngine::new(rules);

    // 执行检查
    let rule_type = rule_types.as_deref().unwrap_or(&[]).join(",");
    let phase = context.as_deref().unwrap_or("撰写");

    let results = engine.check_all(&rule_type, &text, None, phase);

    // 格式化结果
    let output = serde_json::json!({
        "passed": results.iter().all(|r| r.passed),
        "total": results.len(),
        "failed": results.iter().filter(|r| !r.passed).count(),
        "results": results.iter().map(|r| {
            serde_json::json!({
                "rule_id": r.rule_id,
                "rule_name": r.rule_name,
                "passed": r.passed,
                "severity": format!("{:?}", r.severity),
                "action": format!("{:?}", r.action),
                "legal_basis": r.legal_basis,
                "details": r.details,
            })
        }).collect::<Vec<_>>()
    });

    serde_json::to_string(&output).map_err(|e| format!("结果序列化失败: {e}"))
}

/// 获取可用规则类型列表
#[tauri::command]
pub fn list_rule_types() -> Result<Vec<String>, String> {
    let rules_path = Path::new("assets/constitutional");
    let rules = RuleLoader::load_dir(&rules_path).map_err(|e| format!("规则加载失败: {e}"))?;

    let types: std::collections::HashSet<String> = rules
        .values()
        .flat_map(|rule_set| rule_set.rules.values())
        .map(|r| r.id.clone())
        .collect();

    let mut type_list: Vec<_> = types.into_iter().collect();
    type_list.sort();
    Ok(type_list)
}
