//! 撰写质量评估
//!
//! 提供专利撰写质量的自动化评估能力，
//! 涵盖多维度的质量检查与评分。

use serde::{Deserialize, Serialize};

/// 质量维度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDimension {
    pub name: String,
    pub score: f64,
    pub max_score: f64,
    pub description: String,
}

/// 撰写质量报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftQualityReport {
    pub overall_score: f64,
    pub is_acceptable: bool,
    pub dimensions: Vec<QualityDimension>,
    pub critical_issues: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for DraftQualityReport {
    fn default() -> Self {
        Self {
            overall_score: 0.0,
            is_acceptable: false,
            dimensions: vec![
                QualityDimension {
                    name: "权利要求清晰性".into(),
                    score: 0.0,
                    max_score: 10.0,
                    description: "权利要求表述是否清晰明确".into(),
                },
                QualityDimension {
                    name: "权利要求层次".into(),
                    score: 0.0,
                    max_score: 10.0,
                    description: "独立与从属权利要求的层次结构".into(),
                },
                QualityDimension {
                    name: "技术方案完整性".into(),
                    score: 0.0,
                    max_score: 10.0,
                    description: "技术方案是否完整描述".into(),
                },
                QualityDimension {
                    name: "说明书充分公开".into(),
                    score: 0.0,
                    max_score: 10.0,
                    description: "说明书是否充分公开发明内容".into(),
                },
                QualityDimension {
                    name: "实施例充分性".into(),
                    score: 0.0,
                    max_score: 10.0,
                    description: "实施例数量和详尽程度".into(),
                },
                QualityDimension {
                    name: "附图引用".into(),
                    score: 0.0,
                    max_score: 10.0,
                    description: "说明书与附图的一致性".into(),
                },
                QualityDimension {
                    name: "形式规范".into(),
                    score: 0.0,
                    max_score: 10.0,
                    description: "格式是否符合审查指南要求".into(),
                },
            ],
            critical_issues: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl DraftQualityReport {
    /// 根据各维度分数重新计算总分
    pub fn recalculate_overall_score(&mut self) {
        if self.dimensions.is_empty() {
            self.overall_score = 0.0;
            return;
        }
        let sum: f64 = self.dimensions.iter().map(|d| d.score).sum();
        self.overall_score = sum / self.dimensions.len() as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_report_has_seven_dimensions() {
        let report = DraftQualityReport::default();
        assert_eq!(report.dimensions.len(), 7);
    }

    #[test]
    fn default_report_is_not_acceptable() {
        let report = DraftQualityReport::default();
        assert!(!report.is_acceptable);
        assert!((report.overall_score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn recalculate_averages_dimensions() {
        let mut report = DraftQualityReport::default();
        report.dimensions[0].score = 8.0;
        report.dimensions[1].score = 6.0;
        for d in &mut report.dimensions.iter_mut().skip(2) {
            d.score = 7.0;
        }
        report.recalculate_overall_score();
        let expected: f64 = (8.0 + 6.0 + 7.0_f64 * 5.0) / 7.0;
        assert!((report.overall_score - expected).abs() < 0.01);
    }

    #[test]
    fn recalculate_empty_dimensions_yields_zero() {
        let mut report = DraftQualityReport {
            dimensions: vec![],
            ..DraftQualityReport::default()
        };
        report.recalculate_overall_score();
        assert!((report.overall_score).abs() < f64::EPSILON);
    }

    #[test]
    fn dimension_max_score_is_ten() {
        let report = DraftQualityReport::default();
        for d in &report.dimensions {
            assert!((d.max_score - 10.0).abs() < f64::EPSILON);
        }
    }
}
