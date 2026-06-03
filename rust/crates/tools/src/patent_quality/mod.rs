//! YunXi 专利质量评分与检查工具（纯规则型）。
//!
//! 合并了质量评分工具（`quality_scorer`）和质量检查工具（`quality_checker`）。
//! 12 条质量规则 + 四维评分算法 + 百分位排名，零 LLM 依赖。
//! 移植自 TS `QualityRules.ts` + `QualityScorer.ts` + `QCQualityCheckerAgent`。

#![allow(dead_code)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::doc_markdown)]

mod checker;
mod scorer;

mod types;
mod rules;
mod dimensions;
mod recommendations;

pub use checker::*;
pub use scorer::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    // --- Quality Scorer tests ---

    fn scorer_sample_input() -> QualityScorerInput {
        QualityScorerInput {
            claims: vec![
                QualityClaim {
                    r#type: "independent".to_string(),
                    number: 1,
                    content: "一种图像识别装置，其特征在于，包括图像采集模块和图像处理模块。".to_string(),
                    depends_on: None,
                },
                QualityClaim {
                    r#type: "dependent".to_string(),
                    number: 2,
                    content: "根据权利要求1所述的图像识别装置，其特征在于，所述图像处理模块包括卷积神经网络。".to_string(),
                    depends_on: Some(1),
                },
                QualityClaim {
                    r#type: "dependent".to_string(),
                    number: 3,
                    content: "根据权利要求2所述的图像识别装置，其特征在于，还包括特征提取单元。".to_string(),
                    depends_on: Some(2),
                },
            ],
            specification: QualitySpec {
                technical_field: Some("本发明涉及计算机视觉和图像处理技术领域".to_string()),
                background_art: Some("现有的图像识别技术存在准确率低、处理速度慢等问题。传统方法采用手工特征提取，无法适应复杂场景。".to_string()),
                invention_content: Some("本发明提供一种基于深度学习的图像识别装置，通过卷积神经网络实现端到端的图像特征提取和分类，显著提高了识别准确率和处理效率。".to_string()),
                embodiment: Some("实施例1：如图1所示，本发明的图像识别装置包括图像采集模块和图像处理模块。图像采集模块采用CMOS传感器采集原始图像数据，图像处理模块包括预处理单元、特征提取单元和分类单元。预处理单元对原始图像进行归一化处理，特征提取单元采用ResNet-50网络提取特征向量，分类单元采用全连接层输出分类结果。该装置在ImageNet数据集上达到了95%的准确率。".to_string()),
            },
            patent_type: "invention".to_string(),
            invention_title: "图像识别装置".to_string(),
            drawings: vec![ScorerDrawing {
                figure_number: "图1".to_string(),
                description: "系统结构图".to_string(),
            }],
            check_level: 2,
        }
    }

    #[test]
    fn scorer_test_completeness_score() {
        let input = scorer_sample_input();
        let score = scorer_check_completeness(&input);
        assert!(score > 80.0, "completeness should be high: {score}");
    }

    #[test]
    fn scorer_test_claims_quality() {
        let input = scorer_sample_input();
        let scores = scorer_assess_claims_quality(&input);
        assert!(scores.overall > 50.0, "claims overall: {}", scores.overall);
        assert!(scores.breadth > 0.0);
    }

    #[test]
    fn scorer_test_specification_quality() {
        let input = scorer_sample_input();
        let scores = scorer_assess_specification_quality(&input);
        assert!(scores.overall > 50.0, "spec overall: {}", scores.overall);
    }

    #[test]
    fn scorer_test_language_quality() {
        let input = scorer_sample_input();
        let scores = scorer_assess_language_quality(&input);
        assert!(scores.overall > 0.0);
    }

    #[test]
    fn scorer_test_legal_quality() {
        let input = scorer_sample_input();
        let scores = scorer_assess_legal_quality(&input);
        assert!(scores.formality > 50.0);
        assert_eq!(scores.risk_level, "low");
    }

    #[test]
    fn scorer_test_rules_no_issues() {
        let input = scorer_sample_input();
        let issues = scorer_apply_rules(&input, 2);
        // 样例输入应该是比较好的，问题应该很少
        assert!(issues.len() <= 3, "issues: {issues:?}");
    }

    #[test]
    fn scorer_test_rules_invalid_reference() {
        let input = QualityScorerInput {
            claims: vec![QualityClaim {
                r#type: "dependent".to_string(),
                number: 1,
                content: "测试。".to_string(),
                depends_on: Some(5), // 无效引用
            }],
            specification: QualitySpec::default(),
            patent_type: "invention".to_string(),
            invention_title: "测试".to_string(),
            drawings: vec![],
            check_level: 2,
        };
        let issues = scorer_apply_rules(&input, 2);
        assert!(issues
            .iter()
            .any(|i| i.description.contains("引用关系无效")));
    }

    #[test]
    fn scorer_test_rules_vague_expression() {
        let input = QualityScorerInput {
            claims: vec![QualityClaim {
                r#type: "independent".to_string(),
                number: 1,
                content: "一种装置，其特征在于，大约包括模块A。".to_string(),
                depends_on: None,
            }],
            specification: QualitySpec::default(),
            patent_type: "invention".to_string(),
            invention_title: "测试".to_string(),
            drawings: vec![],
            check_level: 3,
        };
        let issues = scorer_apply_rules(&input, 3);
        assert!(issues.iter().any(|i| i.description.contains("模糊")));
    }

    #[test]
    fn scorer_test_quality_level() {
        assert_eq!(scorer_get_quality_level(95.0), "excellent");
        assert_eq!(scorer_get_quality_level(80.0), "good");
        assert_eq!(scorer_get_quality_level(65.0), "fair");
        assert_eq!(scorer_get_quality_level(50.0), "poor");
    }

    #[test]
    fn scorer_test_keyword_overlap() {
        let kw1 = vec!["图像".to_string(), "识别".to_string(), "装置".to_string()];
        let kw2 = vec!["图像".to_string(), "处理".to_string(), "装置".to_string()];
        let overlap = calculate_keyword_overlap(&kw1, &kw2);
        assert!(overlap > 0.0 && overlap < 1.0);
    }

    #[test]
    fn scorer_test_execute_full_scoring() {
        let input = QualityScorerInput {
            claims: vec![
                QualityClaim {
                    r#type: "independent".to_string(),
                    number: 1,
                    content: "一种图像识别装置，其特征在于，包括图像采集模块和图像处理模块。"
                        .to_string(),
                    depends_on: None,
                },
                QualityClaim {
                    r#type: "dependent".to_string(),
                    number: 2,
                    content: "根据权利要求1所述的装置，其特征在于，包括卷积神经网络。".to_string(),
                    depends_on: Some(1),
                },
            ],
            specification: QualitySpec {
                technical_field: Some("计算机视觉和图像处理技术领域".to_string()),
                background_art: Some("现有图像识别技术准确率低、速度慢的问题".to_string()),
                invention_content: Some("基于深度学习的图像识别装置，显著提高准确率".to_string()),
                embodiment: Some(
                    "图像采集模块采用CMOS传感器，图像处理模块包括预处理、特征提取和分类单元"
                        .to_string(),
                ),
            },
            patent_type: "invention".to_string(),
            invention_title: "图像识别装置".to_string(),
            drawings: vec![],
            check_level: 2,
        };
        let result = execute_quality_scorer(&input).unwrap();
        assert!(result["overallQuality"].as_f64().unwrap() > 0.0);
        assert!(!result["qualityLevel"].as_str().unwrap().is_empty());
        assert!(result["comparison"]["percentile"].as_f64().unwrap() > 0.0);
    }

    // --- Quality Checker tests ---

    fn checker_minimal_input() -> QualityCheckerInput {
        QualityCheckerInput {
            claims: vec![QualityClaim {
                r#type: "independent".to_string(),
                number: 1,
                content: "一种数据处理装置，其特征在于包括处理单元。".to_string(),
                depends_on: None,
            }],
            specification: QualitySpec {
                technical_field: Some("本发明涉及数据处理技术领域，具体涉及一种基于深度学习的图像识别方法".to_string()),
                background_art: Some("现有技术中存在多种图像识别方案，但准确率有待提高。传统的图像识别技术主要依赖手工特征提取，泛化能力差。".to_string()),
                invention_content: Some("本发明提供一种基于深度学习的图像识别方法和装置，通过卷积神经网络实现端到端的特征学习和分类，显著提高识别准确率和泛化能力。".to_string()),
                embodiment: Some("实施例一：如图1所示，本实施例的图像识别装置包括输入模块、特征提取模块和分类模块。输入模块接收待识别图像，特征提取模块使用预训练的卷积神经网络提取深层特征，分类模块基于Softmax分类器输出识别结果。".to_string()),
            },
            patent_type: "invention".to_string(),
            invention_title: "一种图像识别方法及装置".to_string(),
            check_level: 2,
        }
    }

    #[test]
    fn checker_test_valid_input_passes() {
        let result = execute_quality_checker(&checker_minimal_input()).unwrap();
        assert!(result["overallQuality"].as_f64().unwrap() > 0.0);
        let level = result["qualityLevel"].as_str().unwrap();
        assert!(level == "fair" || level == "good" || level == "excellent");
    }

    #[test]
    fn checker_test_empty_claims_critical_issue() {
        let input = QualityCheckerInput {
            claims: vec![],
            specification: QualitySpec::default(),
            patent_type: "invention".to_string(),
            invention_title: "测试".to_string(),
            check_level: 2,
        };
        let result = execute_quality_checker(&input).unwrap();
        let issues = result["issues"].as_array().unwrap();
        assert!(issues
            .iter()
            .any(|i| i["ruleId"].as_str() == Some("LEGAL_001")));
    }

    #[test]
    fn checker_test_first_claim_not_independent() {
        let mut input = checker_minimal_input();
        input.claims[0].r#type = "dependent".to_string();
        let result = execute_quality_checker(&input).unwrap();
        let issues = result["issues"].as_array().unwrap();
        assert!(issues
            .iter()
            .any(|i| i["ruleId"].as_str() == Some("CLAIM_001")));
    }

    #[test]
    fn checker_test_vague_expression_detected() {
        let mut input = checker_minimal_input();
        input.claims[0].content = "一种数据处理装置，大约包括处理单元。".to_string();
        let result = execute_quality_checker(&input).unwrap();
        let issues = result["issues"].as_array().unwrap();
        assert!(issues
            .iter()
            .any(|i| i["ruleId"].as_str() == Some("LANG_003")));
    }

    #[test]
    fn checker_test_check_level_filters_rules() {
        let mut input = checker_minimal_input();
        input.check_level = 1;
        let result = execute_quality_checker(&input).unwrap();
        let applied = result["rulesApplied"].as_array().unwrap();
        // Level 1 should not include medium-severity rules (CLAIM_003, LANG_002, LANG_003)
        assert!(!applied.iter().any(|r| r.as_str() == Some("CLAIM_003")));
    }

    #[test]
    fn checker_test_incomplete_claim_end() {
        let mut input = checker_minimal_input();
        // 内容以中文逗号结尾，触发 LANG_002
        input.claims[0].content = "一种数据处理装置，其特征在于包括处理单元，".to_string();
        let result = execute_quality_checker(&input).unwrap();
        let issues = result["issues"].as_array().unwrap();
        assert!(issues
            .iter()
            .any(|i| i["ruleId"].as_str() == Some("LANG_002")));
    }
}
