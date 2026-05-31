//! 6 阶段推理管道
//!
//! 基于 Athena 的超级推理引擎重写。
//! Engagement → Analysis → Hypothesis → Discovery → Testing → Correction

use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::executor::{PhaseExecutionOutput, ReasoningExecutionResult, ReasoningExecutor};
use crate::monitor::{MetaCognitiveMonitor, ReasoningBudget};

/// 推理阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReasoningPhase {
    Engagement,
    Analysis,
    Hypothesis,
    Discovery,
    Testing,
    Correction,
}

impl ReasoningPhase {
    pub fn all() -> &'static [ReasoningPhase] {
        &[
            ReasoningPhase::Engagement,
            ReasoningPhase::Analysis,
            ReasoningPhase::Hypothesis,
            ReasoningPhase::Discovery,
            ReasoningPhase::Testing,
            ReasoningPhase::Correction,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Engagement => "初始参与",
            Self::Analysis => "问题分析",
            Self::Hypothesis => "假设生成",
            Self::Discovery => "证据发现",
            Self::Testing => "假设验证",
            Self::Correction => "修正综合",
        }
    }
}

/// 单步推理输出
#[derive(Debug, Clone, Serialize)]
pub struct ReasoningStepOutput {
    pub phase: ReasoningPhase,
    pub output: serde_json::Value,
    pub elapsed_ms: u64,
}

/// 推理管道结果
#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct ReasoningResult {
    pub steps: Vec<ReasoningStepOutput>,
    pub final_conclusion: String,
    pub hypotheses_generated: usize,
    pub hypotheses_validated: usize,
    pub total_elapsed_ms: u64,
}

/// 推理管道配置
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub max_hypotheses: usize,
    pub max_iterations: usize,
    pub min_confidence: f64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_hypotheses: 5,
            max_iterations: 3,
            min_confidence: 0.6,
        }
    }
}

/// 推理管道
///
/// 编排 6 阶段推理流程。这是一个框架层，具体的推理逻辑
/// 由调用方通过 LLM + KG + 知识搜索工具实现。
pub struct ReasoningPipeline {
    config: PipelineConfig,
}

impl ReasoningPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// 从问题描述生成结构化推理计划
    ///
    /// 返回各阶段应该执行的分析步骤（纯规划，不执行 LLM 调用）
    pub fn plan(&self, problem: &str) -> Vec<ReasoningStepOutput> {
        let mut steps = Vec::new();

        // Engagement: 理解问题
        steps.push(ReasoningStepOutput {
            phase: ReasoningPhase::Engagement,
            output: serde_json::json!({
                "task": "理解问题",
                "questions": [
                    format!("问题的核心是什么？"),
                    format!("涉及哪些法律/技术领域？"),
                    "有什么已知的约束条件？",
                ],
            }),
            elapsed_ms: 0,
        });

        // Analysis: 分解问题
        steps.push(ReasoningStepOutput {
            phase: ReasoningPhase::Analysis,
            output: serde_json::json!({
                "task": "分解问题",
                "sub_problems": [
                    "识别关键技术特征",
                    "确定适用法律条款",
                    "查找相关审查指南",
                ],
                "required_knowledge": [
                    "审查指南相关章节",
                    "类似案例/判决",
                    "技术领域常识",
                ],
            }),
            elapsed_ms: 0,
        });

        // Hypothesis: 生成假设
        steps.push(ReasoningStepOutput {
            phase: ReasoningPhase::Hypothesis,
            output: serde_json::json!({
                "task": "生成假设",
                "max_hypotheses": self.config.max_hypotheses,
                "instructions": format!(
                    "基于问题分析，生成{}个不同的推理假设，每个假设需包含：\
                     1) 核心主张 2) 支撑论据 3) 预期结论",
                    self.config.max_hypotheses
                ),
            }),
            elapsed_ms: 0,
        });

        // Discovery: 收集证据
        steps.push(ReasoningStepOutput {
            phase: ReasoningPhase::Discovery,
            output: serde_json::json!({
                "task": "收集证据",
                "sources": [
                    "知识图谱查询",
                    "法律法规检索",
                    "审查指南搜索",
                    "案例检索",
                ],
                "min_confidence": self.config.min_confidence,
            }),
            elapsed_ms: 0,
        });

        // Testing: 验证假设
        steps.push(ReasoningStepOutput {
            phase: ReasoningPhase::Testing,
            output: serde_json::json!({
                "task": "验证假设",
                "criteria": [
                    "证据支撑度",
                    "法律条文一致性",
                    "逻辑自洽性",
                    "与审查指南的符合度",
                ],
            }),
            elapsed_ms: 0,
        });

        // Correction: 修正综合
        steps.push(ReasoningStepOutput {
            phase: ReasoningPhase::Correction,
            output: serde_json::json!({
                "task": "修正综合",
                "instructions": [
                    "选择置信度最高的假设",
                    "整合各阶段发现的证据",
                    "形成最终结论和建议",
                ],
                "problem": problem,
            }),
            elapsed_ms: 0,
        });

        steps
    }

    /// 真正执行推理 — 逐阶段调用注入的 `ReasoningExecutor`（LLM）。
    ///
    /// 内部调用 `plan()` 生成各阶段指令，然后遍历每个阶段调用 executor
    /// 的 `execute_phase()` 收集输出。
    ///
    /// 使用 `MetaCognitiveMonitor` 跟踪预算和循环检测。
    pub fn execute(
        &self,
        problem: &str,
        executor: &mut dyn ReasoningExecutor,
        budget: Option<ReasoningBudget>,
    ) -> ReasoningExecutionResult {
        let start = Instant::now();
        let plan = self.plan(problem);
        let default_budget = ReasoningBudget {
            max_iterations: self.config.max_iterations,
            current_iteration: 0,
            max_tokens: 100_000,
            tokens_used: 0,
        };
        let mut monitor = MetaCognitiveMonitor::new(budget.unwrap_or(default_budget));
        let mut phases = Vec::new();
        let mut loop_detected = false;

        for step in &plan {
            if !monitor.within_budget() {
                break;
            }

            if monitor.record_hypothesis(&format!("{:?}", step.phase)) {
                loop_detected = true;
                break;
            }

            let phase_start = Instant::now();
            let llm_output =
                match executor.execute_phase(step.phase, problem, &step.output, &phases) {
                    Ok(output) => output,
                    Err(e) => {
                        phases.push(PhaseExecutionOutput {
                            phase: step.phase,
                            phase_label: step.phase.label().to_string(),
                            instructions: step.output.clone(),
                            llm_output: format!("[ERROR] {e}"),
                            elapsed_ms: phase_start.elapsed().as_millis() as u64,
                        });
                        continue;
                    }
                };

            phases.push(PhaseExecutionOutput {
                phase: step.phase,
                phase_label: step.phase.label().to_string(),
                instructions: step.output.clone(),
                llm_output,
                elapsed_ms: phase_start.elapsed().as_millis() as u64,
            });

            monitor.record_phase(step.phase);
            monitor.increment_iteration();
        }

        ReasoningExecutionResult {
            phases,
            total_elapsed_ms: start.elapsed().as_millis() as u64,
            budget_exhausted: !monitor.within_budget(),
            loop_detected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NoopReasoningExecutor;

    #[test]
    fn test_pipeline_plan() {
        let pipeline = ReasoningPipeline::new(PipelineConfig::default());
        let steps = pipeline.plan("分析专利的新颖性");
        assert_eq!(steps.len(), 6);

        let phases: Vec<_> = steps.iter().map(|s| s.phase).collect();
        assert_eq!(phases, ReasoningPhase::all());
    }

    #[test]
    fn test_phase_labels() {
        assert_eq!(ReasoningPhase::Engagement.label(), "初始参与");
        assert_eq!(ReasoningPhase::Testing.label(), "假设验证");
    }

    #[test]
    fn test_custom_config() {
        let config = PipelineConfig {
            max_hypotheses: 10,
            max_iterations: 5,
            min_confidence: 0.8,
        };
        let pipeline = ReasoningPipeline::new(config);
        let steps = pipeline.plan("测试");
        let hypo_step = &steps[2];
        assert_eq!(hypo_step.output["max_hypotheses"], 10);
    }

    #[test]
    fn test_execute_with_noop_executor() {
        let pipeline = ReasoningPipeline::new(PipelineConfig::default());
        let mut executor = NoopReasoningExecutor {
            model: "test".into(),
        };
        let budget = ReasoningBudget {
            max_iterations: 10,
            current_iteration: 0,
            max_tokens: 100_000,
            tokens_used: 0,
        };
        let result = pipeline.execute("分析专利的新颖性", &mut executor, Some(budget));
        assert_eq!(result.phases.len(), 6);
        assert!(result.total_elapsed_ms < 1000);
        assert!(!result.budget_exhausted);
        assert!(!result.loop_detected);

        for phase_output in &result.phases {
            assert!(phase_output.llm_output.contains("NOOP"));
            assert!(!phase_output.phase_label.is_empty());
        }
    }

    #[test]
    fn test_execute_respects_budget() {
        let pipeline = ReasoningPipeline::new(PipelineConfig {
            max_hypotheses: 5,
            max_iterations: 2,
            min_confidence: 0.6,
        });
        let mut executor = NoopReasoningExecutor {
            model: "test".into(),
        };
        let budget = ReasoningBudget {
            max_iterations: 2,
            current_iteration: 0,
            max_tokens: 10_000,
            tokens_used: 0,
        };
        let result = pipeline.execute("测试", &mut executor, Some(budget));
        assert!(result.phases.len() <= 2);
        assert!(result.budget_exhausted);
    }
}
