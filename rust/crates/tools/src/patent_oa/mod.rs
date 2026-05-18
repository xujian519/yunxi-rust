//! OA (Office Action) 分析工具集
//!
//! 提供审查意见书解析、答复模板库、成功率预测等能力。

mod parse;
mod predictor;
mod template;

pub use parse::*;
pub use predictor::*;
pub use template::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_oa_parse_basic() {
        let input = OaParseInput {
            content: "权利要求1-3不具备新颖性，对比文件D1公开了技术特征X".to_string(),
            application_number: Some("CN202310000000.0".to_string()),
            patent_title: None,
            document_type: "cn".to_string(),
            examiner: None,
            notification_date: None,
            deadline: None,
        };

        let result = execute_oa_parse(&input).expect("OA parse should succeed");
        assert_eq!(result["application_number"], "CN202310000000.0");
        assert!(!result["rejection_reasons"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_oa_parse_multiple_rejections() {
        let content = r"
权利要求1不具备新颖性，已被对比文件D1公开。
权利要求2-3不具备创造性，是显而易见的。
权利要求4不清楚，限定不确切。
";

        let input = OaParseInput {
            content: content.to_string(),
            application_number: None,
            patent_title: None,
            document_type: "cn".to_string(),
            examiner: None,
            notification_date: None,
            deadline: None,
        };

        let result = execute_oa_parse(&input).expect("OA parse should succeed");
        let reasons = result["rejection_reasons"].as_array().unwrap();
        assert!(reasons.len() >= 3);
    }

    #[test]
    fn test_response_template_list() {
        let input = ResponseTemplateInput {
            operation: "list".to_string(),
            rejection_type: None,
            strategy: None,
            template_id: None,
            variables: None,
        };

        let result = execute_response_template(&input).expect("list should succeed");
        let templates = result["templates"].as_array().unwrap();
        assert_eq!(templates.len(), 6);
    }

    #[test]
    fn test_response_template_filter() {
        let input = ResponseTemplateInput {
            operation: "filter".to_string(),
            rejection_type: Some("Novelty".to_string()),
            strategy: Some("argue".to_string()),
            template_id: None,
            variables: None,
        };

        let result = execute_response_template(&input).expect("filter should succeed");
        let filtered = result["filtered_templates"].as_array().unwrap();
        assert!(filtered.len() >= 2); // CN and PCT novelty templates
    }

    #[test]
    fn test_response_template_render() {
        let mut variables = HashMap::new();
        variables.insert("claim_numbers".to_string(), "1-3".to_string());
        variables.insert("reference_id".to_string(), "D1".to_string());
        variables.insert("distinguishing_feature".to_string(), "特征X".to_string());

        let input = ResponseTemplateInput {
            operation: "render".to_string(),
            rejection_type: None,
            strategy: None,
            template_id: Some("cn-novelty-argue".to_string()),
            variables: Some(variables),
        };

        let result = execute_response_template(&input).expect("render should succeed");
        let content = result["rendered_content"].as_str().unwrap();
        assert!(content.contains("1-3"));
        assert!(content.contains("D1"));
        assert!(content.contains("特征X"));
    }

    #[test]
    fn test_success_predictor_basic() {
        let parse_result = serde_json::json!({
            "rejection_reasons": [{
                "type": "Novelty",
                "severity": "medium",
                "affected_claims": [1, 2],
                "cited_references": ["D1"]
            }],
            "overall_severity": "medium",
            "total_affected_claims": [1, 2]
        });

        let input = SuccessPredictorInput {
            parse_result: Some(parse_result),
            strategy: "argue".to_string(),
            round: 1,
            confidence_level: "95%".to_string(),
            historical_cases: None,
        };

        let result = execute_success_predictor(&input).expect("prediction should succeed");
        assert!(result["predicted_success_rate"].as_f64().unwrap() > 0.0);
        assert!(result["predicted_success_rate"].as_f64().unwrap() < 1.0);
    }

    #[test]
    fn test_success_predictor_with_history() {
        let parse_result = serde_json::json!({
            "rejection_reasons": [{"type": "Novelty"}],
            "overall_severity": "medium",
            "total_affected_claims": [1]
        });

        let cases = vec![
            HistoricalCase {
                rejection_types: vec!["Novelty".to_string()],
                strategy: "argue".to_string(),
                claim_count: 1,
                round: 1,
                success: true,
            },
            HistoricalCase {
                rejection_types: vec!["Inventiveness".to_string()],
                strategy: "amend".to_string(),
                claim_count: 2,
                round: 1,
                success: false,
            },
        ];

        let input = SuccessPredictorInput {
            parse_result: Some(parse_result),
            strategy: "argue".to_string(),
            round: 1,
            confidence_level: "90%".to_string(),
            historical_cases: Some(cases),
        };

        let result =
            execute_success_predictor(&input).expect("prediction with history should succeed");
        assert!(result["historical_boost"].is_object() || result["historical_boost"].is_number());
    }
}
