//! 推理执行器 — 将 6 阶段指令模板交给外部 LLM 实际执行。
//!
//! `ReasoningPipeline` 的 `plan()` 方法生成各阶段的指令模板，
//! `execute()` 方法通过注入的 `ReasoningExecutor` 实现 LLM 调用，
//! 完成从"空壳"到"真正推理"的关键桥接。

use crate::pipeline::ReasoningPhase;
use serde::Serialize;

/// 执行器对单个推理阶段的调用结果
#[derive(Debug, Clone, Serialize)]
pub struct PhaseExecutionOutput {
    pub phase: ReasoningPhase,
    pub phase_label: String,
    pub instructions: serde_json::Value,
    pub llm_output: String,
    pub elapsed_ms: u64,
}

/// 完整的推理执行结果
#[derive(Debug, Clone, Serialize)]
pub struct ReasoningExecutionResult {
    pub phases: Vec<PhaseExecutionOutput>,
    pub total_elapsed_ms: u64,
    pub budget_exhausted: bool,
    pub loop_detected: bool,
}

/// 推理执行器 trait — 由调用方（如 tools crate）注入 LLM 实现。
pub trait ReasoningExecutor: Send {
    /// 对单个推理阶段执行 LLM 调用。
    ///
    /// # Arguments
    /// * `phase` - 当前推理阶段
    /// * `problem` - 用户输入的原始问题描述
    /// * `instructions` - 由 `plan()` 生成的指令 JSON
    /// * `previous_phases` - 之前阶段积累的结果（可为空）
    ///
    /// # Returns
    /// LLM 的输出文本
    fn execute_phase(
        &mut self,
        phase: ReasoningPhase,
        problem: &str,
        instructions: &serde_json::Value,
        previous_phases: &[PhaseExecutionOutput],
    ) -> Result<String, String>;

    /// 获取执行器的模型名称（用于日志）
    fn model_name(&self) -> &str {
        "unknown"
    }
}

/// 无操作执行器（用于测试，直接返回阶段名）
pub struct NoopReasoningExecutor {
    pub model: String,
}

impl ReasoningExecutor for NoopReasoningExecutor {
    fn execute_phase(
        &mut self,
        phase: ReasoningPhase,
        _problem: &str,
        instructions: &serde_json::Value,
        _previous_phases: &[PhaseExecutionOutput],
    ) -> Result<String, String> {
        let task = instructions
            .get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(format!(
            "[NOOP] 阶段「{}」的「{}」已执行（模拟LLM输出）",
            phase.label(),
            task
        ))
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_executor_returns_labeled_output() {
        let mut executor = NoopReasoningExecutor {
            model: "noop-test".into(),
        };
        let instructions = serde_json::json!({
            "task": "理解问题",
            "questions": ["问题的核心是什么？"]
        });
        let result = executor
            .execute_phase(ReasoningPhase::Engagement, "测试问题", &instructions, &[])
            .unwrap();
        assert!(result.contains("初始参与"));
        assert!(result.contains("NOOP"));
    }
}
