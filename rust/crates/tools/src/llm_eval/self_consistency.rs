use crate::llm_eval::{AnswerVariant, LLMJudgeEngine, SelfConsistencyResult};
use std::collections::HashMap;

impl LLMJudgeEngine {
    /// Self-Consistency 评估。
    ///
    /// 多次采样，评估 LLM 输出的一致性。
    pub fn eval_self_consistency(
        &self,
        _input_text: &str,
        num_samples: usize,
    ) -> SelfConsistencyResult {
        let mut answers = Vec::new();
        let mut answer_counts: HashMap<String, usize> = HashMap::new();

        // 模拟多次采样（实际应调用 LLM）
        for i in 0..num_samples {
            // 在实际实现中，这里应该调用 LLM 多次
            // 使用不同的采样参数（temperature 等）
            let answer = format!("答案 {}：基于输入的标准化回答", i + 1);
            answers.push(answer.clone());
            *answer_counts.entry(answer).or_insert(0) += 1;
        }

        // 找到最常见的答案
        let majority_answer = answer_counts
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(answer, _)| answer.clone())
            .unwrap_or_default();

        let majority_count = answer_counts.get(&majority_answer).unwrap_or(&0);
        let consistency_score = (*majority_count as f64 / num_samples as f64) * 100.0;

        let answer_distribution: Vec<AnswerVariant> = answer_counts
            .into_iter()
            .map(|(answer, count)| AnswerVariant {
                answer,
                count,
                percentage: count as f64 / num_samples as f64 * 100.0,
            })
            .collect();

        let confidence = consistency_score / 100.0;

        SelfConsistencyResult {
            consistency_score,
            majority_answer,
            answer_distribution,
            confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_self_consistency() {
        let engine = LLMJudgeEngine::with_default();
        let result = engine.eval_self_consistency("测试输入", 5);

        assert!(result.consistency_score >= 0.0);
        assert!(result.consistency_score <= 100.0);
        assert!(!result.majority_answer.is_empty());
        assert_eq!(result.answer_distribution.len(), 5);
        assert!(result.confidence >= 0.0);
        assert!(result.confidence <= 1.0);
    }

    #[test]
    fn test_answer_distribution() {
        let engine = LLMJudgeEngine::with_default();
        let result = engine.eval_self_consistency("测试输入", 3);

        let total_percentage: f64 = result
            .answer_distribution
            .iter()
            .map(|v| v.percentage)
            .sum();

        // 百分比总和应该约为 100%
        assert!((total_percentage - 100.0).abs() < 0.01);
    }
}
