//! `YunXi` 专利形式检查工具集。
//!
//! 合并自 `YunXi` 的四个形式检查工具：
//! - 权利要求形式检查器
//! - 说明书形式检查器
//! - 保护客体检查器
//! - 单一性检查器

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

mod claim_check;
mod spec_check;
mod subject_matter;
mod unity_check;

// 公开类型 re-export
pub use claim_check::{execute_claim_formality_check, ClaimFormalityInput};
pub use spec_check::{execute_spec_formality_check, SpecFormalityInput};
pub use subject_matter::{execute_subject_matter_check, SubjectMatterInput};
pub use unity_check::{execute_unity_check, UnityCheckInput};

// ==================== 测试 ====================

#[cfg(test)]
mod tests {
    use super::claim_check::{
        identify_unnecessary_features, is_concise, is_unclear, perform_claim_formality_check,
        CfClaimInput,
    };
    use super::unity_check::calculate_similarity;
    use super::*;
    use serde_json::{json, Value};

    // --- 权利要求形式检查测试 ---

    #[test]
    fn test_is_unclear() {
        assert!(is_unclear("温度大约50度"));
        assert!(is_unclear("速度左右100"));
        assert!(is_unclear("可能有效"));
        assert!(!is_unclear("温度为50度"));
    }

    #[test]
    fn test_is_concise() {
        // 短文本应该通过
        assert!(is_concise("一种数据处理方法"));
        // 超长文本应不通过
        let long_text: String = "一".repeat(400);
        assert!(!is_concise(&long_text));
        // 太多细节模式
        let detailed = "其中所述装置具体来说优选地更优选地还包括";
        assert!(!is_concise(detailed));
    }

    #[test]
    fn test_identify_unnecessary_features() {
        let result = identify_unnecessary_features("采用常规技术实现", 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].claim_number, 1);
        assert_eq!(result[0].feature, "采用常规技术");

        let result2 = identify_unnecessary_features("创新的技术方案", 1);
        assert!(result2.is_empty());
    }

    #[test]
    fn test_formality_check_clean_claims() {
        let claims = vec![CfClaimInput {
            claim_number: 1,
            full_text: "一种数据处理方法，包括以下步骤：获取数据；处理数据；输出结果".to_string(),
            content: "一种数据处理方法，包括以下步骤：获取数据；处理数据；输出结果".to_string(),
            claim_type: "independent".to_string(),
        }];
        let result = perform_claim_formality_check(&claims);
        assert!(result.passed);
        assert!(result.clarity_issues.is_empty());
        assert!(result.unnecessary_features.is_empty());
    }

    #[test]
    fn test_formality_check_unclear_claim() {
        let claims = vec![CfClaimInput {
            claim_number: 1,
            full_text: "一种方法，温度大约50度，速度左右100".to_string(),
            content: "一种方法，温度大约50度，速度左右100".to_string(),
            claim_type: "independent".to_string(),
        }];
        let result = perform_claim_formality_check(&claims);
        assert!(!result.passed);
        assert!(!result.clarity_issues.is_empty());
    }

    #[test]
    fn test_formality_check_unnecessary_feature() {
        let claims = vec![CfClaimInput {
            claim_number: 1,
            full_text: "一种采用常规技术的通信系统".to_string(),
            content: "一种采用常规技术的通信系统".to_string(),
            claim_type: "independent".to_string(),
        }];
        let result = perform_claim_formality_check(&claims);
        assert!(!result.passed);
        assert!(!result.unnecessary_features.is_empty());
        assert_eq!(result.unnecessary_features[0].claim_number, 1);
    }

    #[test]
    fn test_claim_formality_execute() {
        let input: ClaimFormalityInput = serde_json::from_value(json!({
            "claims": [
                { "claimNumber": 1, "fullText": "一种方法", "content": "一种方法", "type": "independent" }
            ]
        })).unwrap();
        let result = execute_claim_formality_check(&input).unwrap();
        assert!(result["passed"].as_bool().unwrap());
    }

    // --- 说明书形式检查测试 ---

    fn complete_spec_input() -> Value {
        json!({
            "specification": {
                "technicalField": "本发明涉及电子通信技术领域，特别是涉及一种基于5G网络的低延迟数据传输方法",
                "backgroundArt": "现有技术中存在数据传输延迟高、丢包率高等不足，无法满足实时应用需求",
                "inventionContent": "本发明提供一种低延迟数据传输方法，采用自适应编码技术和多路径聚合方案，包括发送端处理模块和接收端处理模块",
                "drawingsDescription": "图1为系统架构示意图；图2为数据传输流程图",
                "embodiment": "实施例一：如图1所示，本发明的系统包括发送端处理模块、接收端处理模块和传输控制单元。发送端处理模块配置为对数据进行编码和封装，接收端处理模块配置为对接收到的数据进行解码和重组。传输控制单元根据网络状况动态调整传输参数，确保数据传输的低延迟和高可靠性。"
            },
            "claims": [{
                "type": "independent",
                "number": 1,
                "content": "一种低延迟数据传输方法，其特征在于包括发送端处理模块和接收端处理模块。"
            }],
            "patentType": "invention"
        })
    }

    #[test]
    fn test_complete_spec_passes() {
        let input: SpecFormalityInput = serde_json::from_value(complete_spec_input()).unwrap();
        let result = execute_spec_formality_check(&input).unwrap();
        assert!(result["overallReport"]["passed"].as_bool().unwrap());
        assert_eq!(result["overallReport"]["totalIssues"].as_u64().unwrap(), 0);
    }

    #[test]
    fn test_missing_components_detected() {
        let input: SpecFormalityInput = serde_json::from_value(json!({
            "specification": {
                "technicalField": "电子技术"
            },
            "patentType": "invention"
        }))
        .unwrap();
        let result = execute_spec_formality_check(&input).unwrap();
        assert!(!result["overallReport"]["passed"].as_bool().unwrap());
        let missing = result["rule17Components"]["missingComponents"]
            .as_array()
            .expect("rule17Components.missingComponents should be array");
        assert!(missing.contains(&json!("背景技术")));
        assert!(missing.contains(&json!("发明内容")));
        assert!(missing.contains(&json!("具体实施方式")));
    }

    #[test]
    fn test_unclear_expressions_detected() {
        let mut val = complete_spec_input();
        val["specification"]["technicalField"] = json!("大约在电子技术领域，可能涉及左右5G技术");
        let input: SpecFormalityInput = serde_json::from_value(val).unwrap();
        let result = execute_spec_formality_check(&input).unwrap();
        let issues = result["article264Clarity"]["issues"].as_array().unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_utility_model_requires_structure_features() {
        let mut val = complete_spec_input();
        val["patentType"] = json!("utilityModel");
        val["specification"]["embodiment"] = json!("实施例一：通过算法优化实现性能提升。");
        let input: SpecFormalityInput = serde_json::from_value(val).unwrap();
        let result = execute_spec_formality_check(&input).unwrap();
        let issues = result["rule19Embodiment"]["issues"].as_array().unwrap();
        let has_structure_issue = issues
            .iter()
            .any(|i| i["issue"].as_str().unwrap().contains("形状、构造特征"));
        assert!(has_structure_issue);
    }

    #[test]
    fn test_claims_consistency_detects_missing_feature() {
        let mut val = complete_spec_input();
        val["claims"] = json!([{
            "type": "independent",
            "number": 1,
            "content": "一种数据传输方法，包括\"量子纠缠通信模块\"。"
        }]);
        let input: SpecFormalityInput = serde_json::from_value(val).unwrap();
        let result = execute_spec_formality_check(&input).unwrap();
        let unsupported = result["claimsConsistency"]["unsupportedClaims"]
            .as_array()
            .unwrap();
        assert!(!unsupported.is_empty());
    }

    #[test]
    fn test_short_technical_field_detected() {
        let mut val = complete_spec_input();
        val["specification"]["technicalField"] = json!("电子");
        let input: SpecFormalityInput = serde_json::from_value(val).unwrap();
        let result = execute_spec_formality_check(&input).unwrap();
        let issues = result["article263Disclosure"]["issues"].as_array().unwrap();
        let has_short_issue = issues
            .iter()
            .any(|i| i["issue"].as_str().unwrap().contains("过于简单"));
        assert!(has_short_issue);
    }

    // --- 保护客体检查测试 ---

    fn valid_sm_input() -> Value {
        json!({
            "inventionTitle": "一种低延迟数据传输装置",
            "claims": [{
                "type": "independent",
                "number": 1,
                "content": "一种低延迟数据传输装置，包括发送端处理模块和接收端处理模块，通过基于5G网络进行数据传输。"
            }],
            "specification": {
                "technicalField": "电子通信技术",
                "backgroundArt": "现有技术存在延迟高、丢包率高等不足",
                "inventionContent": "本发明提高了传输效率，改善了精度，增强了稳定性"
            },
            "patentType": "invention"
        })
    }

    #[test]
    fn test_valid_invention_passes() {
        let input: SubjectMatterInput = serde_json::from_value(valid_sm_input()).unwrap();
        let result = execute_subject_matter_check(&input).unwrap();
        assert!(result["overallReport"]["passed"].as_bool().unwrap());
        assert!(result["article2InventionDefinition"]["isTechnicalSolution"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_intellectual_activity_detected() {
        let mut val = valid_sm_input();
        val["claims"] = json!([{
            "type": "independent",
            "number": 1,
            "content": "一种商业模式的经营方法策略"
        }]);
        let input: SubjectMatterInput = serde_json::from_value(val).unwrap();
        let result = execute_subject_matter_check(&input).unwrap();
        assert!(
            result["intellectualActivityCheck"]["hasIntellectualActivityRules"]
                .as_bool()
                .unwrap()
        );
    }

    #[test]
    fn test_illegal_content_detected() {
        let mut val = valid_sm_input();
        val["claims"] = json!([{
            "type": "independent",
            "number": 1,
            "content": "一种用于赌博的博彩赌场装置，包括控制器。"
        }]);
        let input: SubjectMatterInput = serde_json::from_value(val).unwrap();
        let result = execute_subject_matter_check(&input).unwrap();
        assert!(!result["legalityCheck"]["passed"].as_bool().unwrap());
    }

    #[test]
    fn test_medical_diagnosis_detected() {
        let mut val = valid_sm_input();
        val["claims"] = json!([{
            "type": "independent",
            "number": 1,
            "content": "一种疾病的诊断检查筛查方法，包括检测装置。"
        }]);
        let input: SubjectMatterInput = serde_json::from_value(val).unwrap();
        let result = execute_subject_matter_check(&input).unwrap();
        let matters = result["article25Exclusions"]["nonProtectableMatters"]
            .as_array()
            .unwrap();
        assert!(matters
            .iter()
            .any(|m| m["type"].as_str().unwrap() == "medical_diagnosis_treatment"));
    }

    #[test]
    fn test_nuclear_detected() {
        let mut val = valid_sm_input();
        val["claims"] = json!([{
            "type": "independent",
            "number": 1,
            "content": "一种原子核变换裂变方法"
        }]);
        let input: SubjectMatterInput = serde_json::from_value(val).unwrap();
        let result = execute_subject_matter_check(&input).unwrap();
        let matters = result["article25Exclusions"]["nonProtectableMatters"]
            .as_array()
            .unwrap();
        assert!(matters
            .iter()
            .any(|m| m["type"].as_str().unwrap() == "nuclear_transformation"));
    }

    // --- 单一性检查测试 ---

    fn single_independent_claim() -> Value {
        json!({
            "claims": [{
                "type": "independent",
                "number": 1,
                "content": "一种\"数据传输装置\"，包括控制器和传感器。"
            }],
            "patentType": "invention"
        })
    }

    fn unified_claims() -> Value {
        json!({
            "claims": [{
                "type": "independent",
                "number": 1,
                "content": "一种\"数据传输控制器\"，包括处理器和传感器。"
            }, {
                "type": "independent",
                "number": 2,
                "content": "一种\"信号处理控制器\"，包括处理器和传感器。"
            }, {
                "type": "dependent",
                "number": 3,
                "content": "根据权利要求1所述的装置，其特征在于还包括\"放大器\"。",
                "dependsOn": 1
            }],
            "patentType": "invention"
        })
    }

    fn unrelated_claims() -> Value {
        json!({
            "claims": [{
                "type": "independent",
                "number": 1,
                "content": "一种\"自行车座椅\"调节装置。"
            }, {
                "type": "independent",
                "number": 2,
                "content": "一种\"咖啡研磨\"方法。"
            }],
            "patentType": "invention"
        })
    }

    #[test]
    fn test_single_claim_auto_passes() {
        let input: UnityCheckInput = serde_json::from_value(single_independent_claim()).unwrap();
        let result = execute_unity_check(&input).unwrap();
        assert!(result["rule43Unity"]["passed"].as_bool().unwrap());
        assert!(result["unityAnalysis"]["hasUnity"].as_bool().unwrap());
    }

    #[test]
    fn test_unified_claims_pass() {
        let input: UnityCheckInput = serde_json::from_value(unified_claims()).unwrap();
        let result = execute_unity_check(&input).unwrap();
        // 有共同组件特征（处理器、传感器）→ 应通过
        assert!(result["rule43Unity"]["passed"].as_bool().unwrap());
        assert!(
            !result["featureAnalysis"]["commonFeatures"]
                .as_array()
                .unwrap()
                .is_empty()
                || !result["featureAnalysis"]["correspondingFeatures"]
                    .as_array()
                    .unwrap()
                    .is_empty()
        );
    }

    #[test]
    fn test_unrelated_claims_fail() {
        let input: UnityCheckInput = serde_json::from_value(unrelated_claims()).unwrap();
        let result = execute_unity_check(&input).unwrap();
        assert!(!result["rule43Unity"]["passed"].as_bool().unwrap());
        assert!(!result["unityAnalysis"]["hasUnity"].as_bool().unwrap());
        let issues = result["rule43Unity"]["issues"].as_array().unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_unity_score_range() {
        let input: UnityCheckInput = serde_json::from_value(unified_claims()).unwrap();
        let result = execute_unity_check(&input).unwrap();
        let score = result["overallReport"]["unityScore"].as_u64().unwrap();
        assert!(score <= 100);
    }

    #[test]
    fn test_single_claim_has_general_concept() {
        let input: UnityCheckInput = serde_json::from_value(single_independent_claim()).unwrap();
        let result = execute_unity_check(&input).unwrap();
        assert!(result["rule44GeneralConcept"]["hasGeneralConcept"]
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_calculate_similarity_same() {
        let sim = calculate_similarity("数据传输控制器", "数据传输控制器");
        assert!(
            sim > 0.5,
            "same string similarity should be > 0.5, got {sim}"
        );
    }

    #[test]
    fn test_calculate_similarity_different() {
        let sim = calculate_similarity("自行车座椅", "咖啡研磨方法");
        assert!(sim < 0.5);
    }
}
