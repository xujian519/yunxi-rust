//! 规则引擎工具包装层

use serde::Deserialize;
use serde_json::Value;

use patent_domain::rules::{evaluate, load_rules, PatentDocument};

/// 规则引擎输入
#[derive(Debug, Deserialize)]
pub struct RulesEngineInput {
    /// 操作类型
    pub operation: String,
    /// YAML 规则文本（load 操作使用）
    #[serde(default)]
    pub rules_yaml: Option<String>,
    /// 专利文档（evaluate 操作使用）
    #[serde(default)]
    pub document: Option<DocumentData>,
    /// 已加载的规则 ID 列表（evaluate 操作使用）
    #[serde(default)]
    pub rule_ids: Option<Vec<String>>,
}

/// 专利文档数据
#[derive(Debug, Deserialize)]
pub struct DocumentData {
    /// 标题
    #[serde(default)]
    pub title: Option<String>,
    /// 摘要
    #[serde(rename = "abstractText", default)]
    pub abstract_text: Option<String>,
    /// 权利要求列表
    #[serde(default)]
    pub claims: Vec<String>,
    /// 说明书
    #[serde(default)]
    pub specification: Option<String>,
    /// 附图列表
    #[serde(default)]
    pub drawings: Vec<String>,
}

lazy_static::lazy_static! {
    static ref RULES_REGISTRY: std::sync::Mutex<Vec<(String, Vec<patent_domain::rules::Rule>)>> =
        std::sync::Mutex::new(Vec::new());
}

/// 执行规则引擎操作
pub fn rules_engine_tool(input: RulesEngineInput) -> Result<Value, String> {
    match input.operation.as_str() {
        "load" => {
            let yaml = input
                .rules_yaml
                .ok_or("rules_yaml required for load operation")?;
            let rules = load_rules(&yaml)?;

            let rule_id = format!("rule_{}", uuid::Uuid::new_v4());

            let mut registry = RULES_REGISTRY.lock().map_err(|e| e.to_string())?;
            registry.push((rule_id.clone(), rules.clone()));

            let rule_count = rules.len();

            Ok(serde_json::json!({
                "rule_id": rule_id,
                "rule_count": rule_count,
                "message": "规则加载成功"
            }))
        }
        "evaluate" => {
            let doc_data = input
                .document
                .ok_or("document required for evaluate operation")?;
            let rule_ids = input
                .rule_ids
                .ok_or("rule_ids required for evaluate operation")?;

            let document = PatentDocument {
                title: doc_data.title,
                abstract_text: doc_data.abstract_text,
                claims: doc_data.claims,
                specification: doc_data.specification,
                drawings: doc_data.drawings,
            };

            let registry = RULES_REGISTRY.lock().map_err(|e| e.to_string())?;
            let mut all_violations = Vec::new();

            for rule_id in rule_ids {
                if let Some((_, rules)) = registry.iter().find(|(id, _)| id == &rule_id) {
                    let violations = evaluate(&document, rules);
                    all_violations.extend(violations);
                }
            }

            let error_count = all_violations
                .iter()
                .filter(|v| matches!(v.severity, patent_domain::rules::Severity::Error))
                .count();
            let warning_count = all_violations
                .iter()
                .filter(|v| matches!(v.severity, patent_domain::rules::Severity::Warning))
                .count();
            let info_count = all_violations
                .iter()
                .filter(|v| matches!(v.severity, patent_domain::rules::Severity::Info))
                .count();

            Ok(serde_json::json!({
                "violations": all_violations,
                "summary": {
                    "total": all_violations.len(),
                    "errors": error_count,
                    "warnings": warning_count,
                    "info": info_count
                },
                "passed": all_violations.is_empty()
            }))
        }
        "list_rules" => {
            let registry = RULES_REGISTRY.lock().map_err(|e| e.to_string())?;
            let rule_ids: Vec<_> = registry
                .iter()
                .map(|(id, rules)| {
                    serde_json::json!({
                        "rule_id": id,
                        "rule_count": rules.len()
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "rule_sets": rule_ids
            }))
        }
        _ => Err(format!("Unknown operation: {}", input.operation)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_rules() {
        let yaml = r#"
rules:
  - id: R001
    name: 标题必填
    target: specification
    severity: error
    check:
      type: required
      field: title
  - id: R002
    name: 摘要长度
    target: abstract
    severity: warning
    check:
      type: min_length
      field: abstract_text
      value: 50
"#;

        let input = RulesEngineInput {
            operation: "load".into(),
            rules_yaml: Some(yaml.into()),
            document: None,
            rule_ids: None,
        };

        let result = rules_engine_tool(input).unwrap();
        assert!(result["rule_id"].is_string());
        assert_eq!(result["rule_count"], 2);
    }

    #[test]
    fn test_evaluate_document() {
        let yaml = r#"
rules:
  - id: R001
    name: 标题必填
    target: specification
    severity: error
    check:
      type: required
      field: title
"#;

        let load_input = RulesEngineInput {
            operation: "load".into(),
            rules_yaml: Some(yaml.into()),
            document: None,
            rule_ids: None,
        };
        let load_result = rules_engine_tool(load_input).unwrap();
        let rule_id = load_result["rule_id"].as_str().unwrap();

        let eval_input = RulesEngineInput {
            operation: "evaluate".into(),
            rules_yaml: None,
            document: Some(DocumentData {
                title: None,
                abstract_text: Some("摘要内容".into()),
                claims: vec![],
                specification: None,
                drawings: vec![],
            }),
            rule_ids: Some(vec![rule_id.into()]),
        };

        let result = rules_engine_tool(eval_input).unwrap();
        assert_eq!(result["summary"]["total"], 1);
        assert_eq!(result["summary"]["errors"], 1);
        assert_eq!(result["passed"], false);
    }

    #[test]
    fn test_list_rules() {
        let input = RulesEngineInput {
            operation: "list_rules".into(),
            rules_yaml: None,
            document: None,
            rule_ids: None,
        };

        let result = rules_engine_tool(input).unwrap();
        assert!(result["rule_sets"].is_array());
    }
}
