use std::fmt::Write;

use crate::{analyzer::TaskAnalyzer, scorer::ComplexityScorer};
use crate::{ComplexityScore, ModelSelection, RouterError, TaskContext};

pub struct ModelSelector {
    analyzer: TaskAnalyzer,
    scorer: ComplexityScorer,
    fallback_model: String,
}

impl ModelSelector {
    pub fn new() -> Self {
        Self {
            analyzer: TaskAnalyzer::new(),
            scorer: ComplexityScorer::new(),
            fallback_model: "deepseek-v4-pro".to_string(),
        }
    }

    pub fn select_model(&self, ctx: &TaskContext) -> Result<ModelSelection, RouterError> {
        let mut features = self.analyzer.analyze(&ctx.user_input);
        features.history_rounds = ctx.history_rounds;
        features.files_involved = ctx.files_involved;

        let score = self.scorer.score(&features);
        let model = self.decide_model(&score);
        let reason = self.generate_reason(&score, model);

        Ok(ModelSelection {
            model: model.to_string(),
            score,
            reason,
            forced: false,
        })
    }

    pub fn select_model_forced(&self, model: &str) -> ModelSelection {
        ModelSelection {
            model: model.to_string(),
            score: ComplexityScore::zero(),
            reason: "用户强制指定".to_string(),
            forced: true,
        }
    }

    pub fn select_model_safe(&self, ctx: &TaskContext) -> ModelSelection {
        match self.select_model(ctx) {
            Ok(selection) => selection,
            Err(e) => {
                log::warn!("模型选择失败，回退到回退模型: {}", e);
                ModelSelection {
                    model: self.fallback_model.clone(),
                    score: ComplexityScore::zero(),
                    reason: format!("回退模式: {}", e),
                    forced: false,
                }
            }
        }
    }

    fn decide_model(&self, score: &ComplexityScore) -> &str {
        if score.total >= self.scorer.threshold() {
            "deepseek-v4-pro"
        } else {
            "deepseek-v4-flash"
        }
    }

    fn generate_reason(&self, score: &ComplexityScore, model: &str) -> String {
        let mut reason = String::new();
        let _ = writeln!(
            reason,
            "综合评分: {}/{} (阈值: {})",
            score.total,
            self.scorer.threshold(),
            self.scorer.threshold()
        );
        let _ = writeln!(reason, " - 任务类型: {}分", score.task_type_score);
        let _ = writeln!(reason, " - 输入复杂度: {}分", score.input_score);
        let _ = writeln!(reason, " - 上下文: {}分", score.context_score);
        let _ = writeln!(reason, " - 工具调用: {}分", score.tools_score);
        let _ = writeln!(reason, "选择模型: {}", model);
        reason
    }
}

impl Default for ModelSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UserInput;

    #[test]
    fn test_forced_selection() {
        let selector = ModelSelector::new();
        let selection = selector.select_model_forced("deepseek-v4-pro");
        assert_eq!(selection.model, "deepseek-v4-pro");
        assert!(selection.forced);
    }

    #[test]
    fn test_simple_task_uses_flash() {
        let selector = ModelSelector::new();
        let ctx = TaskContext::new(UserInput::new("你好"));
        let selection = selector.select_model_safe(&ctx);
        assert_eq!(selection.model, "deepseek-v4-flash");
    }

    #[test]
    fn test_complex_task_uses_pro() {
        let selector = ModelSelector::new();
        let ctx = TaskContext::new(UserInput::new(
            "帮我规划这个大型项目的架构设计并评估风险、制定实施策略。\n\
             重构以下代码: function init() { let config = {\"retry\": 3, \"timeout\": 5000}; return config; }",
        ))
        .with_history(10)
        .with_files(5);
        let selection = selector.select_model_safe(&ctx);
        assert_eq!(selection.model, "deepseek-v4-pro");
    }

    #[test]
    fn test_fallback_on_safe() {
        let selector = ModelSelector::new();
        let ctx = TaskContext::new(UserInput::new("测试"));
        let selection = selector.select_model_safe(&ctx);
        assert!(!selection.forced);
        assert!(!selection.model.is_empty());
    }
}
