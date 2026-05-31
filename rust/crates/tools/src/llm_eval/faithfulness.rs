use crate::llm_eval::{FaithfulnessResult, Hallucination, LLMJudgeEngine};

impl LLMJudgeEngine {
    /// Faithfulness 评估（事实一致性）。
    ///
    /// 检查 LLM 输出是否与检索到的事实一致。
    pub fn eval_faithfulness(
        &self,
        llm_output: &str,
        source_documents: &[&str],
    ) -> FaithfulnessResult {
        let mut hallucinations = Vec::new();
        let mut supported_claims = Vec::new();

        // 简化的实现：逐句检查是否在源文档中找到支持
        for sentence in llm_output.split('。') {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            let is_supported = source_documents.iter().any(|doc| doc.contains(sentence));

            if is_supported {
                supported_claims.push(sentence.to_string());
            } else {
                // 检查是否包含关键事实
                if self.is_fact_claim(sentence) {
                    hallucinations.push(Hallucination {
                        claim: sentence.to_string(),
                        severity: "medium".to_string(),
                        explanation: "未在源文档中找到该事实".to_string(),
                    });
                }
            }
        }

        let hallucination_rate = if !supported_claims.is_empty() {
            hallucinations.len() as f64 / (supported_claims.len() + hallucinations.len()) as f64
        } else {
            1.0
        };

        let faithfulness_score = (1.0 - hallucination_rate) * 100.0;

        let source_alignment = if source_documents.is_empty() {
            0.0
        } else {
            let matched = source_documents
                .iter()
                .filter(|doc| supported_claims.iter().any(|claim| doc.contains(claim)))
                .count();
            matched as f64 / source_documents.len() as f64 * 100.0
        };

        FaithfulnessResult {
            faithfulness_score,
            hallucinations,
            supported_claims,
            source_alignment,
        }
    }

    fn is_fact_claim(&self, sentence: &str) -> bool {
        // 简单的启发式规则：包含数字、专有名词或特定事实陈述
        sentence.contains('是')
            || sentence.contains('为')
            || sentence.contains('有')
            || sentence.contains('在')
            || sentence.chars().any(|c| c.is_ascii_digit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_faithfulness() {
        let engine = LLMJudgeEngine::with_default();
        let output = "这是一段测试文本。它包含一些事实。这是另一个句子。";
        let sources = vec!["这是一段测试文本。它包含一些事实。", "这是源文档的内容。"];

        let result = engine.eval_faithfulness(output, &sources);

        assert!(result.faithfulness_score >= 0.0);
        assert!(result.faithfulness_score <= 100.0);
        assert!(!result.supported_claims.is_empty());
    }

    #[test]
    fn test_eval_faithfulness_with_hallucinations() {
        let engine = LLMJudgeEngine::with_default();
        let output = "这是正确的信息。这是不存在的信息。";
        let sources = vec!["这是正确的信息。"];

        let result = engine.eval_faithfulness(output, &sources);

        assert!(result.faithfulness_score < 100.0);
        assert!(!result.hallucinations.is_empty());
    }

    #[test]
    fn test_is_fact_claim() {
        let engine = LLMJudgeEngine::with_default();

        assert!(engine.is_fact_claim("这是一个事实陈述"));
        assert!(engine.is_fact_claim("数量为123"));
        assert!(engine.is_fact_claim("北京是首都"));
        assert!(!engine.is_fact_claim("可能的情况"));
    }
}
