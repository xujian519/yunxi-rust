//! `YunXi` OA 答复策略工具。
//!
//! 合并自 `strategy_scorer.rs` 与 `strategy_argument_generator.rs`。
//! 提供策略评分与论点生成两个功能。

use serde::{Deserialize, Serialize};

mod arguments;
mod scoring;

pub use arguments::*;
pub use scoring::*;
/// 答复策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseStrategy {
    Argue,
    Amend,
    Both,
    Abandon,
    Appeal,
}

impl ResponseStrategy {
    pub(crate) fn all() -> Vec<ResponseStrategy> {
        vec![
            ResponseStrategy::Argue,
            ResponseStrategy::Amend,
            ResponseStrategy::Both,
            ResponseStrategy::Abandon,
            ResponseStrategy::Appeal,
        ]
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ResponseStrategy::Argue => "argue",
            ResponseStrategy::Amend => "amend",
            ResponseStrategy::Both => "both",
            ResponseStrategy::Abandon => "abandon",
            ResponseStrategy::Appeal => "appeal",
        }
    }
}

pub(crate) fn strategy_name(strategy: ResponseStrategy) -> &'static str {
    match &strategy {
        ResponseStrategy::Argue => "争辩策略",
        ResponseStrategy::Amend => "修改策略",
        ResponseStrategy::Both => "混合策略",
        ResponseStrategy::Abandon => "放弃策略",
        ResponseStrategy::Appeal => "复审策略",
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    // ---- 策略评分器测试 ----

    #[test]
    fn test_base_preference_scores() {
        assert_eq!(
            base_preference_score(UserPreference::Aggressive, ResponseStrategy::Argue),
            85.0
        );
        assert_eq!(
            base_preference_score(UserPreference::Conservative, ResponseStrategy::Amend),
            85.0
        );
    }

    #[test]
    fn test_best_strategies() {
        let strategies = best_strategies_for_rejection("novelty");
        assert!(strategies.contains(&ResponseStrategy::Amend));
        assert!(strategies.contains(&ResponseStrategy::Argue));
    }

    #[test]
    fn test_rejection_match_with_match() {
        let reasons = vec![ScoreRejectionReason {
            rejection_type: "novelty".to_string(),
            severity: "high".to_string(),
            suggested_response: Some("amend".to_string()),
        }];
        let score = calculate_rejection_match(&reasons, ResponseStrategy::Amend);
        assert!(score > 50.0);
    }

    #[test]
    fn test_rejection_match_empty() {
        let score = calculate_rejection_match(&[], ResponseStrategy::Argue);
        assert_eq!(score, 50.0);
    }

    #[test]
    fn test_risk_adjustment() {
        let reasons = vec![ScoreRejectionReason {
            rejection_type: "novelty".to_string(),
            severity: "high".to_string(),
            suggested_response: None,
        }];
        let score =
            calculate_risk_adjustment(&reasons, &[1, 2, 3, 4, 5, 6], ResponseStrategy::Argue, 0.5);
        assert!(score < 100.0);
    }

    #[test]
    fn test_evaluate_strategies() {
        let input = StrategyScoreInput {
            rejection_reasons: vec![ScoreRejectionReason {
                rejection_type: "novelty".to_string(),
                severity: "high".to_string(),
                suggested_response: Some("amend".to_string()),
            }],
            rejection_types: vec!["novelty".to_string()],
            affected_claims: vec![1, 2],
            cited_references: vec![],
            patent_title: String::new(),
            user_preference: Some(UserPreference::Moderate),
            risk_tolerance: Some(0.5),
            historical_cases: vec![],
        };
        let scores = evaluate_strategies(&input);
        assert_eq!(scores.len(), 5);
        // 分数应降序排列
        for i in 1..scores.len() {
            assert!(scores[i - 1].score >= scores[i].score);
        }
    }

    #[test]
    fn test_tool_execute_score() {
        let input: StrategyScoreInput = serde_json::from_value(serde_json::json!({
            "rejectionReasons": [
                { "type": "inventiveness", "severity": "high", "suggestedResponse": "argue" }
            ],
            "rejectionTypes": ["inventiveness"],
            "affectedClaims": [1, 2, 3],
            "userPreference": "aggressive"
        }))
        .unwrap();

        let result = execute_strategy_score(&input).unwrap();

        let scores = result["scores"].as_array().unwrap();
        assert_eq!(scores.len(), 5);
        assert!(!result["recommendedStrategy"].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_case_similarity() {
        let case = ScoreHistoricalCase {
            id: "case1".to_string(),
            strategy: "amend".to_string(),
            outcome: "success".to_string(),
            rejection_reasons: vec![ScoreRejectionReason {
                rejection_type: "novelty".to_string(),
                severity: "high".to_string(),
                suggested_response: None,
            }],
            technical_field: "数据处理".to_string(),
            granted_claims: vec![1, 2],
        };
        let sim =
            calculate_case_similarity(&["novelty".to_string()], &[1, 2], 1, "数据处理", &case);
        assert!(sim > 0.0);
    }

    // ---- 论点生成器测试 ----

    fn make_parse_result() -> ArgumentOAParseResult {
        ArgumentOAParseResult {
            rejection_reasons: vec![ArgumentRejectionReason {
                rejection_type: ArgumentRejectionType::Novelty,
                severity: "high".to_string(),
                affected_claims: vec![1, 2],
                related_references: vec!["CN123456".to_string()],
            }],
            rejection_types: vec![ArgumentRejectionType::Novelty],
            affected_claims: vec![1, 2],
            cited_references: vec![ArgumentCitedReference {
                publication_number: "CN123456".to_string(),
                title: "一种数据处理方法".to_string(),
            }],
            patent_title: "测试专利".to_string(),
        }
    }

    #[test]
    fn test_generate_key_arguments() {
        let pr = make_parse_result();
        let args = generate_key_arguments(&pr, ResponseStrategy::Argue);
        assert!(!args.is_empty());
        assert!(args[0].argument.contains("区别技术特征"));
        assert_eq!(args[0].target_rejection, "novelty");
    }

    #[test]
    fn test_generate_key_arguments_inventiveness() {
        let pr = ArgumentOAParseResult {
            rejection_reasons: vec![ArgumentRejectionReason {
                rejection_type: ArgumentRejectionType::Inventiveness,
                severity: "medium".to_string(),
                affected_claims: vec![1],
                related_references: vec![],
            }],
            rejection_types: vec![ArgumentRejectionType::Inventiveness],
            affected_claims: vec![1],
            cited_references: vec![],
            patent_title: String::new(),
        };
        let args = generate_key_arguments(&pr, ResponseStrategy::Argue);
        assert_eq!(args.len(), 3);
        assert!(args[0].argument.contains("技术启示") || args[0].argument.contains("显而易见"));
    }

    #[test]
    fn test_customize_template_claims() {
        let rejection = ArgumentRejectionReason {
            rejection_type: ArgumentRejectionType::Novelty,
            severity: "high".to_string(),
            affected_claims: vec![1, 3],
            related_references: vec!["US789".to_string()],
        };
        let pr = ArgumentOAParseResult {
            rejection_reasons: vec![],
            rejection_types: vec![],
            affected_claims: vec![],
            cited_references: vec![],
            patent_title: String::new(),
        };
        let result = customize_template("权利要求{claims}与{reference}对比", &rejection, &pr);
        assert!(result.contains("1, 3"));
        assert!(result.contains("US789"));
    }

    #[test]
    fn test_generate_amendment_suggestions_argue() {
        let pr = make_parse_result();
        let suggestions = generate_amendment_suggestions(&pr, ResponseStrategy::Argue);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_generate_amendment_suggestions_amend() {
        let pr = make_parse_result();
        let suggestions = generate_amendment_suggestions(&pr, ResponseStrategy::Amend);
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].claim_number, 1);
    }

    #[test]
    fn test_identify_risks() {
        let pr = make_parse_result();
        let risks = identify_risks(&pr, ResponseStrategy::Amend);
        assert!(risks.iter().any(|r| r.contains("保护范围缩小")));
        assert!(risks.iter().any(|r| r.contains("高严重程度")));
    }

    #[test]
    fn test_suggest_additional_evidence() {
        let pr = ArgumentOAParseResult {
            rejection_reasons: vec![ArgumentRejectionReason {
                rejection_type: ArgumentRejectionType::Inventiveness,
                severity: "high".to_string(),
                affected_claims: vec![1],
                related_references: vec![],
            }],
            rejection_types: vec![],
            affected_claims: vec![1],
            cited_references: vec![],
            patent_title: String::new(),
        };
        let evidence = suggest_additional_evidence(&pr);
        assert!(evidence.iter().any(|e| e.contains("实验数据")));
        assert!(evidence.iter().any(|e| e.contains("技术对比")));
    }

    #[test]
    fn test_generate_alternatives() {
        let scores = vec![
            ScoreItem {
                strategy: ResponseStrategy::Argue,
                score: 80.0,
                details: ScoreItemDetails {
                    rejection_match: 80.0,
                    historical_success: 70.0,
                    risk_adjustment: 75.0,
                },
            },
            ScoreItem {
                strategy: ResponseStrategy::Amend,
                score: 70.0,
                details: ScoreItemDetails {
                    rejection_match: 60.0,
                    historical_success: 80.0,
                    risk_adjustment: 50.0,
                },
            },
        ];
        let alts = generate_alternatives(&scores, ResponseStrategy::Argue);
        assert_eq!(alts.len(), 1);
        assert_eq!(alts[0].strategy, ResponseStrategy::Amend);
    }

    #[test]
    fn test_generate_rationale() {
        let score = ScoreItem {
            strategy: ResponseStrategy::Amend,
            score: 85.0,
            details: ScoreItemDetails {
                rejection_match: 80.0,
                historical_success: 75.0,
                risk_adjustment: 80.0,
            },
        };
        let r = generate_rationale(&score, 5);
        assert!(r.contains("修改策略"));
        assert!(r.contains("高度匹配"));
        assert!(r.contains("风险可控"));
    }

    #[test]
    fn test_tool_execute_arguments() {
        let input: StrategyArgumentInput = serde_json::from_value(serde_json::json!({
            "parseResult": {
                "rejectionReasons": [{
                    "type": "novelty",
                    "severity": "high",
                    "affectedClaims": [1],
                    "relatedReferences": ["CN123"]
                }],
                "citedReferences": [{ "publicationNumber": "CN123", "title": "测试" }],
                "affectedClaims": [1]
            },
            "strategy": "both",
            "scores": [{ "strategy": "both", "score": 80, "details": { "rejectionMatch": 80, "historicalSuccess": 70, "riskAdjustment": 75 } }]
        })).unwrap();

        let result = execute_strategy_arguments(&input).unwrap();

        assert!(!result["arguments"].as_array().unwrap().is_empty());
        assert!(!result["amendmentSuggestions"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(result["rationale"].as_str().unwrap().contains("混合策略"));
    }
}
