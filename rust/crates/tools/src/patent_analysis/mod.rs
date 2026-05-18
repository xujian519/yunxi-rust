// patent_analysis.rs - 专利分析工具集
//
// 提供4个专利分析相关工具:
// 1. SemanticCompare - 多模式语义对比
// 2. InfringementAnalysis - 专利侵权分析
// 3. SynergyAnalysis - 技术特征协同测试
// 4. LegalQA - 知识产权法律问答

mod analysis;
mod compare;

pub use analysis::{
    infringement_analysis, legal_qa, synergy_analysis, InfringementAnalysisInput, LegalQAInput,
    SynergyAnalysisInput,
};
pub use compare::{semantic_compare, SemanticCompareInput};

#[cfg(test)]
mod tests {
    use super::analysis::TechUnit;
    use super::compare::CompareDoc;
    use super::*;

    #[test]
    fn test_semantic_compare_lexical() {
        let target = CompareDoc {
            title: Some("深度学习图像识别方法".to_string()),
            abstract_text: Some("本发明涉及一种基于卷积神经网络的图像识别技术".to_string()),
            claims: Some(vec!["一种图像识别方法，包括神经网络处理步骤。".to_string()]),
            features: Some(vec!["高精度".to_string(), "实时处理".to_string()]),
        };

        let prior_art = CompareDoc {
            title: Some("神经网络图像处理".to_string()),
            abstract_text: Some("基于深度学习的图像处理系统".to_string()),
            claims: Some(vec!["使用神经网络进行图像处理的方法。".to_string()]),
            features: Some(vec!["高精度".to_string(), "快速".to_string()]),
        };

        let input = SemanticCompareInput {
            target,
            prior_art,
            compare_mode: Some("lexical".to_string()),
            weights: None,
        };

        let result = semantic_compare(input).unwrap();
        assert!(result["overall_similarity"].as_f64().unwrap() > 0.0);
        assert_eq!(result["mode"], "lexical");
    }

    #[test]
    fn test_infringement_analysis() {
        let input = InfringementAnalysisInput {
            patent_claims: vec![
                "一种通信设备，包括天线、发射机和接收机。".to_string(),
                "根据权利要求1所述的设备，其特征在于还包括信号处理器。".to_string(),
            ],
            accused_product: "该产品包含天线、发射器、接收器和数字信号处理芯片".to_string(),
            analysis_type: Some("full".to_string()),
        };

        let result = infringement_analysis(input).unwrap();
        assert_eq!(result["claims_analyzed"], 2);
        assert!(result["claim_results"].as_array().unwrap().len() == 2);
    }

    #[test]
    fn test_synergy_analysis() {
        let input = SynergyAnalysisInput {
            units: vec![
                TechUnit {
                    id: "1".to_string(),
                    name: "天线模块".to_string(),
                    source_text: "天线模块用于发送和接收无线信号".to_string(),
                    technical_function: Some("信号收发".to_string()),
                    technical_effect: Some("提高信号强度".to_string()),
                },
                TechUnit {
                    id: "2".to_string(),
                    name: "信号处理模块".to_string(),
                    source_text: "所述信号处理模块对天线接收的信号进行处理".to_string(),
                    technical_function: Some("信号处理".to_string()),
                    technical_effect: Some("提高信号质量".to_string()),
                },
            ],
            apply_merge: Some(false),
        };

        let result = synergy_analysis(input).unwrap();
        assert_eq!(result["total_units"], 2);
        assert_eq!(result["pairs_tested"], 1);
    }

    #[test]
    fn test_legal_qa() {
        let input = LegalQAInput {
            question: "什么情况下构成专利侵权？".to_string(),
            domain: Some("patent".to_string()),
            context: None,
        };

        let result = legal_qa(input).unwrap();
        assert_eq!(result["question"], "什么情况下构成专利侵权？");
        assert!(!result["answer"]["applicable_rules"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(result["confidence"].as_f64().unwrap() > 0.0);
    }
}
