//! 无效决定工具包装层

use serde::Deserialize;
use serde_json::Value;

use patent_domain::invalid_decision::{DecisionType, InvalidDecision, InvalidDecisionStore};

/// 无效决定输入
#[derive(Debug, Deserialize)]
pub struct InvalidDecisionInput {
    /// 操作类型
    pub operation: String,
    /// 决定记录（添加时使用）
    #[serde(default)]
    pub decision: Option<DecisionData>,
    /// 专利号（搜索时使用）
    #[serde(default)]
    pub patent_number: Option<String>,
    /// 无效理由关键词（搜索时使用）
    #[serde(default)]
    pub ground: Option<String>,
}

/// 决定数据
#[derive(Debug, Deserialize)]
pub struct DecisionData {
    /// 决定 ID
    pub id: String,
    /// 专利号
    pub patent_number: String,
    /// 决定书号
    pub decision_number: String,
    /// 决定日期
    pub decision_date: String,
    /// 决定类型：invalid/partial_invalid/maintain
    #[serde(rename = "decisionType")]
    pub decision_type: String,
    /// 无效理由列表
    pub grounds: Vec<String>,
    /// 结论
    pub conclusion: String,
}

lazy_static::lazy_static! {
    static ref DECISION_STORE: std::sync::Mutex<InvalidDecisionStore> =
        std::sync::Mutex::new(InvalidDecisionStore::new());
}

/// 执行无效决定操作
pub fn invalid_decision_tool(input: InvalidDecisionInput) -> Result<Value, String> {
    match input.operation.as_str() {
        "add" => {
            let decision_data = input
                .decision
                .ok_or("decision required for add operation")?;
            let decision_type = match decision_data.decision_type.as_str() {
                "invalid" => DecisionType::Invalid,
                "partial_invalid" => DecisionType::PartialInvalid,
                "maintain" => DecisionType::Maintain,
                _ => {
                    return Err(format!(
                        "Invalid decision type: {}",
                        decision_data.decision_type
                    ))
                }
            };

            let decision = InvalidDecision {
                id: decision_data.id,
                patent_number: decision_data.patent_number,
                decision_number: decision_data.decision_number,
                decision_date: decision_data.decision_date,
                decision_type,
                grounds: decision_data.grounds,
                conclusion: decision_data.conclusion,
            };

            let mut store = DECISION_STORE.lock().map_err(|e| e.to_string())?;
            store.add(decision);

            Ok(serde_json::json!({
                "success": true,
                "message": "无效决定已添加"
            }))
        }
        "search_by_patent" => {
            let patent_number = input.patent_number.ok_or("patent_number required")?;
            let store = DECISION_STORE.lock().map_err(|e| e.to_string())?;
            let results = store.search_by_patent(&patent_number);
            Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
        }
        "search_by_ground" => {
            let ground = input.ground.ok_or("ground required")?;
            let store = DECISION_STORE.lock().map_err(|e| e.to_string())?;
            let results = store.search_by_ground(&ground);
            Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
        }
        "list_all" => {
            let store = DECISION_STORE.lock().map_err(|e| e.to_string())?;
            let results = store.all();
            Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
        }
        _ => Err(format!("Unknown operation: {}", input.operation)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_decision() {
        let input = InvalidDecisionInput {
            operation: "add".into(),
            decision: Some(DecisionData {
                id: "1".into(),
                patent_number: "CN123456".into(),
                decision_number: "WX2024-001".into(),
                decision_date: "2024-01-01".into(),
                decision_type: "invalid".into(),
                grounds: vec!["新颖性".into()],
                conclusion: "全部无效".into(),
            }),
            patent_number: None,
            ground: None,
        };

        let result = invalid_decision_tool(input).unwrap();
        assert_eq!(result["success"], true);
    }

    #[test]
    fn test_search_by_patent() {
        let add_input = InvalidDecisionInput {
            operation: "add".into(),
            decision: Some(DecisionData {
                id: "2".into(),
                patent_number: "CN789012".into(),
                decision_number: "WX2024-002".into(),
                decision_date: "2024-02-01".into(),
                decision_type: "maintain".into(),
                grounds: vec![],
                conclusion: "维持有效".into(),
            }),
            patent_number: None,
            ground: None,
        };
        invalid_decision_tool(add_input).unwrap();

        let search_input = InvalidDecisionInput {
            operation: "search_by_patent".into(),
            decision: None,
            patent_number: Some("CN789012".into()),
            ground: None,
        };

        let result = invalid_decision_tool(search_input).unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_search_by_ground() {
        let add_input = InvalidDecisionInput {
            operation: "add".into(),
            decision: Some(DecisionData {
                id: "3".into(),
                patent_number: "CN345678".into(),
                decision_number: "WX2024-003".into(),
                decision_date: "2024-03-01".into(),
                decision_type: "partial_invalid".into(),
                grounds: vec!["创造性不足".into()],
                conclusion: "部分无效".into(),
            }),
            patent_number: None,
            ground: None,
        };
        invalid_decision_tool(add_input).unwrap();

        let search_input = InvalidDecisionInput {
            operation: "search_by_ground".into(),
            decision: None,
            patent_number: None,
            ground: Some("创造性".into()),
        };

        let result = invalid_decision_tool(search_input).unwrap();
        assert!(result.is_array());
        assert!(!result.as_array().unwrap().is_empty());
    }
}
