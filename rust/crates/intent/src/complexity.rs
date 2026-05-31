//! 任务复杂度评估
//!
//! 基于 Athena 的复杂度评估算法重写。
//! 评估输入文本的认知负荷和任务复杂度。

/// 复杂度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum ComplexityLevel {
    Simple,
    Medium,
    Complex,
}

/// 复杂度评估结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplexityAssessment {
    pub level: ComplexityLevel,
    pub score: f64,
    pub factors: ComplexityFactors,
}

/// 各维度因子
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComplexityFactors {
    pub word_count: usize,
    pub sentence_count: usize,
    pub domain_term_density: f64,
    pub logical_operator_count: usize,
    pub avg_sentence_length: f64,
}

/// 复杂度评估器
pub struct ComplexityAssessor {
    /// 专利法律领域术语
    domain_terms: Vec<&'static str>,
}

impl ComplexityAssessor {
    pub fn new() -> Self {
        Self {
            domain_terms: vec![
                "新颖性",
                "创造性",
                "实用性",
                "权利要求",
                "说明书",
                "独立权利要求",
                "从属权利要求",
                "审查意见",
                "无效宣告",
                "侵权",
                "等同原则",
                "全部要素",
                "现有技术",
                "区别特征",
                "技术效果",
                "技术方案",
                "实施例",
                "背景技术",
                "发明内容",
                "具体实施方式",
                "权利要求书",
                "摘要",
                "附图说明",
                "驳回",
                "复审",
                "行政诉讼",
                "专利法",
                "实施细则",
                "审查指南",
                "IPC分类",
                "优先权",
                "国际申请",
            ],
        }
    }

    /// 评估文本的复杂度
    pub fn assess(&self, text: &str) -> ComplexityAssessment {
        let factors = self.compute_factors(text);
        let score = self.compute_score(&factors);
        let level = if score < 0.3 {
            ComplexityLevel::Simple
        } else if score < 0.7 {
            ComplexityLevel::Medium
        } else {
            ComplexityLevel::Complex
        };

        ComplexityAssessment {
            level,
            score,
            factors,
        }
    }

    fn compute_factors(&self, text: &str) -> ComplexityFactors {
        let word_count = text.chars().filter(|c| !c.is_whitespace()).count();

        let sentence_count = text
            .split(['。', '？', '！', '.', '?'])
            .filter(|s| !s.trim().is_empty())
            .count()
            .max(1);

        // 领域术语密度
        let domain_term_count = self
            .domain_terms
            .iter()
            .filter(|term| text.contains(*term))
            .count();
        let domain_term_density = if word_count > 0 {
            domain_term_count as f64 / (word_count as f64 / 10.0).max(1.0)
        } else {
            0.0
        };

        // 逻辑运算符计数
        let logical_operator_count = [
            "并且", "或者", "以及", "如果", "那么", "但是", "然而", "同时",
        ]
        .iter()
        .map(|op| text.matches(*op).count())
        .sum();

        let avg_sentence_length = word_count as f64 / sentence_count as f64;

        ComplexityFactors {
            word_count,
            sentence_count,
            domain_term_density,
            logical_operator_count,
            avg_sentence_length,
        }
    }

    fn compute_score(&self, factors: &ComplexityFactors) -> f64 {
        let mut score = 0.0;

        // 文本长度因子（0-0.25）
        if factors.word_count > 200 {
            score += 0.25;
        } else if factors.word_count > 50 {
            score += 0.15;
        } else {
            score += 0.05;
        }

        // 领域术语密度因子（0-0.3）
        score += (factors.domain_term_density * 0.3).min(0.3);

        // 逻辑复杂度因子（0-0.25）
        if factors.logical_operator_count > 5 {
            score += 0.25;
        } else if factors.logical_operator_count > 2 {
            score += 0.15;
        } else {
            score += 0.05;
        }

        // 平均句长因子（0-0.2）
        if factors.avg_sentence_length > 80.0 {
            score += 0.2;
        } else if factors.avg_sentence_length > 40.0 {
            score += 0.1;
        }

        score.min(1.0)
    }
}

impl Default for ComplexityAssessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_query() {
        let assessor = ComplexityAssessor::new();
        let result = assessor.assess("查专利");
        assert_eq!(result.level, ComplexityLevel::Simple);
    }

    #[test]
    fn test_complex_query() {
        let assessor = ComplexityAssessor::new();
        let result = assessor.assess(
            "请分析该专利的新颖性和创造性，对比现有技术中的区别特征，\
             并且评估技术效果是否显著，如果存在审查意见，那么制定答复策略",
        );
        assert_eq!(result.level, ComplexityLevel::Complex);
        assert!(result.factors.domain_term_density > 0.0);
    }

    #[test]
    fn test_medium_query() {
        let assessor = ComplexityAssessor::new();
        let result = assessor.assess("帮我分析这个专利的侵权风险");
        assert!(matches!(
            result.level,
            ComplexityLevel::Simple | ComplexityLevel::Medium
        ));
    }
}
