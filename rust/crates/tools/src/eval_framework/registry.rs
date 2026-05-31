use crate::eval_framework::{Evaluator, EvaluatorType};
use std::collections::HashMap;

/// 评估器注册表。
pub struct EvalRegistry {
    evaluators: HashMap<String, Box<dyn Evaluator>>,
}

impl EvalRegistry {
    pub fn new() -> Self {
        Self {
            evaluators: HashMap::new(),
        }
    }

    /// 注册评估器。
    pub fn register(&mut self, name: String, evaluator: Box<dyn Evaluator>) -> Result<(), String> {
        if self.evaluators.contains_key(&name) {
            return Err(format!("Evaluator {} already registered", name));
        }
        self.evaluators.insert(name, evaluator);
        Ok(())
    }

    /// 获取评估器（返回引用）。
    pub fn get(&self, name: &str) -> Option<&dyn Evaluator> {
        self.evaluators.get(name).map(|v| v.as_ref())
    }

    /// 列出所有注册的评估器。
    pub fn list_evaluators(&self) -> Vec<String> {
        self.evaluators.keys().cloned().collect()
    }

    /// 获取评估器类型。
    pub fn get_evaluator_type(&self, name: &str) -> Option<EvaluatorType> {
        self.evaluators.get(name).map(|evaluator| {
            let evaluator_name = evaluator.evaluator_name();
            if evaluator_name.contains("rule") {
                EvaluatorType::RuleBased(evaluator_name.to_string())
            } else if evaluator_name.contains("llm") {
                EvaluatorType::LLMBased(evaluator_name.to_string())
            } else {
                EvaluatorType::Hybrid(evaluator_name.to_string())
            }
        })
    }
}

impl Default for EvalRegistry {
    fn default() -> Self {
        Self::new()
    }
}
