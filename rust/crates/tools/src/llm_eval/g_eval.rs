use crate::llm_eval::{GEvalResult, LLMJudgeEngine};

impl LLMJudgeEngine {
    /// G-Eval 通用评估。
    ///
    /// 使用 LLM 对输入进行结构化评估，返回分数和推理。
    pub fn eval_g(
        &self,
        input_text: &str,
        evaluation_criteria: &[EvaluationCriteria],
        context: Option<&str>,
    ) -> GEvalResult {
        let _prompt = self.build_g_eval_prompt(input_text, evaluation_criteria, context);

        // STUB: 返回硬编码模拟结果，待集成 LLM 模块后替换。
        GEvalResult {
            score: 75.0,
            reasoning: "基于评估标准，输入文本在大部分维度上表现良好，但在某些方面需要改进"
                .to_string(),
            confidence: 0.8,
            criteria_met: vec!["完整性".to_string(), "准确性".to_string()],
            criteria_missing: vec!["深度分析".to_string()],
        }
    }

    fn build_g_eval_prompt(
        &self,
        input_text: &str,
        criteria: &[EvaluationCriteria],
        context: Option<&str>,
    ) -> String {
        let mut prompt = "你是一名资深评估专家。请按照以下标准对文本进行评估：\n\n".to_string();

        if let Some(ctx) = context {
            prompt.push_str(&format!("背景信息：\n{}\n\n", ctx));
        }

        prompt.push_str(&format!("待评估文本：\n{}\n\n", input_text));
        prompt.push_str("评估标准：\n");

        for (i, criterion) in criteria.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. {} (权重: {})\n   说明: {}\n",
                i + 1,
                criterion.name,
                criterion.weight,
                criterion.description
            ));
        }

        prompt.push_str(
            "\n请严格按照以下格式输出：\n\
            ===评分===\n\
            [0-100 之间的数字]\n\
            \n\
            ===推理===\n\
            [详细的评分理由]\n\
            \n\
            ===满足的标准===\n\
            [满足的标准列表，用逗号分隔]\n\
            \n\
            ===未满足的标准===\n\
            [未满足的标准列表，用逗号分隔]\n\
            \n\
            ===置信度===\n\
            [0-1 之间的数字]",
        );

        prompt
    }
}

#[derive(Debug, Clone)]
pub struct EvaluationCriteria {
    pub name: String,
    pub description: String,
    pub weight: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_g_eval_prompt() {
        let engine = LLMJudgeEngine::with_default();
        let criteria = vec![
            EvaluationCriteria {
                name: "完整性".to_string(),
                description: "内容是否完整".to_string(),
                weight: 0.3,
            },
            EvaluationCriteria {
                name: "准确性".to_string(),
                description: "内容是否准确".to_string(),
                weight: 0.4,
            },
        ];

        let prompt =
            engine.build_g_eval_prompt("这是待评估的文本", &criteria, Some("这是背景信息"));

        assert!(prompt.contains("待评估文本"));
        assert!(prompt.contains("完整性"));
        assert!(prompt.contains("权重: 0.3"));
    }

    #[test]
    fn test_eval_g() {
        let engine = LLMJudgeEngine::with_default();
        let criteria = vec![EvaluationCriteria {
            name: "完整性".to_string(),
            description: "内容是否完整".to_string(),
            weight: 0.3,
        }];

        let result = engine.eval_g("测试文本", &criteria, None);

        assert_eq!(result.score, 75.0);
        assert_eq!(result.confidence, 0.8);
        assert!(result.criteria_met.contains(&"完整性".to_string()));
    }
}
