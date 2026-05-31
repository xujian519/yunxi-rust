use serde::{Deserialize, Serialize};

/// 默认阈值配置。
///
/// 针对不同评估类型的默认阈值。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultThresholds {
    /// 专利质量评分阈值
    pub patent_quality: PatentQualityThresholds,
    /// 代码质量评分阈值
    pub code_quality: CodeQualityThresholds,
    /// LLM 评估阈值
    pub llm_eval: LLMEvalThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentQualityThresholds {
    pub overall_quality_min: f64,
    pub claims_quality_min: f64,
    pub specification_quality_min: f64,
    pub language_quality_min: f64,
    pub legal_quality_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeQualityThresholds {
    pub correctness_min: f64,
    pub efficiency_min: f64,
    pub maintainability_min: f64,
    pub documentation_min: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMEvalThresholds {
    pub g_eval_min: f64,
    pub faithfulness_min: f64,
    pub consistency_min: f64,
}

impl Default for DefaultThresholds {
    fn default() -> Self {
        Self {
            patent_quality: PatentQualityThresholds {
                overall_quality_min: 75.0,
                claims_quality_min: 70.0,
                specification_quality_min: 70.0,
                language_quality_min: 80.0,
                legal_quality_min: 75.0,
            },
            code_quality: CodeQualityThresholds {
                correctness_min: 75.0,
                efficiency_min: 70.0,
                maintainability_min: 70.0,
                documentation_min: 60.0,
            },
            llm_eval: LLMEvalThresholds {
                g_eval_min: 70.0,
                faithfulness_min: 80.0,
                consistency_min: 75.0,
            },
        }
    }
}

impl DefaultThresholds {
    /// 获取专利质量门禁配置。
    pub fn patent_quality_gate(&self) -> crate::quality_gate::QualityGateConfig {
        let mut config = crate::quality_gate::QualityGateConfig::new("专利质量门禁".to_string());
        config.thresholds = vec![
            crate::quality_gate::Threshold {
                metric_name: "overall_quality".to_string(),
                operator: crate::quality_gate::ThresholdOperator::GreaterThanOrEqual,
                value: self.patent_quality.overall_quality_min,
            },
            crate::quality_gate::Threshold {
                metric_name: "quality_scores.claims.overall".to_string(),
                operator: crate::quality_gate::ThresholdOperator::GreaterThanOrEqual,
                value: self.patent_quality.claims_quality_min,
            },
            crate::quality_gate::Threshold {
                metric_name: "quality_scores.specification.overall".to_string(),
                operator: crate::quality_gate::ThresholdOperator::GreaterThanOrEqual,
                value: self.patent_quality.specification_quality_min,
            },
        ];
        config
    }

    /// 获取代码质量门禁配置。
    pub fn code_quality_gate(&self) -> crate::quality_gate::QualityGateConfig {
        let mut config = crate::quality_gate::QualityGateConfig::new("代码质量门禁".to_string());
        config.thresholds = vec![
            crate::quality_gate::Threshold {
                metric_name: "correctness_score".to_string(),
                operator: crate::quality_gate::ThresholdOperator::GreaterThanOrEqual,
                value: self.code_quality.correctness_min,
            },
            crate::quality_gate::Threshold {
                metric_name: "maintainability_score".to_string(),
                operator: crate::quality_gate::ThresholdOperator::GreaterThanOrEqual,
                value: self.code_quality.maintainability_min,
            },
        ];
        config
    }

    /// 获取 LLM 评估门禁配置。
    pub fn llm_eval_gate(&self) -> crate::quality_gate::QualityGateConfig {
        let mut config = crate::quality_gate::QualityGateConfig::new("LLM 评估门禁".to_string());
        config.thresholds = vec![
            crate::quality_gate::Threshold {
                metric_name: "score".to_string(),
                operator: crate::quality_gate::ThresholdOperator::GreaterThanOrEqual,
                value: self.llm_eval.g_eval_min,
            },
            crate::quality_gate::Threshold {
                metric_name: "faithfulness_score".to_string(),
                operator: crate::quality_gate::ThresholdOperator::GreaterThanOrEqual,
                value: self.llm_eval.faithfulness_min,
            },
        ];
        config
    }
}
