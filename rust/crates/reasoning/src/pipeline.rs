//! 6 阶段推理管道
//!
//! 基于 Athena 的超级推理引擎重写。
//! Engagement → Analysis → Hypothesis → Discovery → Testing → Correction

use std::sync::Arc;
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
pub struct ReasoningResult {
    pub steps: Vec<ReasoningStepOutput>,
    pub final_conclusion: String,
    pub hypotheses_generated: usize,
    pub hypotheses_validated: usize,
    pub total_elapsed_ms: u64,
}

/// 推理管道配置
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// 反思结果（推理管道内部使用）
///
/// 由 `PhaseReflector` 返回，用于指导 Correction 阶段的重试逻辑。
#[derive(Debug, Clone)]
pub struct PhaseReflectionResult {
    /// 阶段是否通过
    pub passed: bool,
    /// 质量评分 [0, 100]
    pub score: f64,
    /// 是否建议重试
    pub should_retry: bool,
    /// 改进建议
    pub suggestions: Vec<String>,
}

/// 推理阶段反思接口
///
/// 在 Correction 阶段调用，评估阶段输出质量并决定是否需要重试。
/// 定义在 reasoning crate 内以避免对 tools crate 的循环依赖。
pub trait PhaseReflector: Send + Sync {
    /// 反思评估阶段输出。
    ///
    /// # Arguments
    /// * `phase` - 当前推理阶段
    /// * `phase_output` - 阶段执行输出
    /// * `all_phases` - 已完成的所有阶段输出
    ///
    /// # Returns
    /// 反思评估结果
    fn reflect_phase(
        &self,
        phase: ReasoningPhase,
        phase_output: &PhaseExecutionOutput,
        all_phases: &[PhaseExecutionOutput],
    ) -> PhaseReflectionResult;
}

/// 记忆写入接口（推理管道内部使用）
///
/// 用于在反思完成后将评估结果写入记忆系统。
/// 定义在 reasoning crate 内以避免对 memory crate 的直接依赖。
pub trait ReflectionMemory: Send + Sync {
    /// 写入反思评估结果到记忆系统。
    fn store_reflection(
        &self,
        session_id: &str,
        phase: ReasoningPhase,
        score: f64,
        passed: bool,
        suggestions: &[String],
    ) -> Result<(), String>;

    /// 写入完整推理结果到记忆系统。
    ///
    /// # Arguments
    /// * `session_id` - 会话 ID
    /// * `problem` - 原始问题
    /// * `phases_summary` - 各阶段摘要（phase_label → llm_output 前 200 字符）
    /// * `total_elapsed_ms` - 总耗时
    /// * `budget_exhausted` - 是否预算耗尽
    fn store_reasoning_result(
        &self,
        _session_id: &str,
        _problem: &str,
        _phases_summary: &[(String, String)],
        _total_elapsed_ms: u64,
        _budget_exhausted: bool,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// 推理管道
///
/// 编排 6 阶段推理流程。这是一个框架层，具体的推理逻辑
/// 由调用方通过 LLM + KG + 知识搜索工具实现。
pub struct ReasoningPipeline {
    config: PipelineConfig,
    reflector: Option<Arc<dyn PhaseReflector>>,
    memory: Option<Arc<dyn ReflectionMemory>>,
}

impl ReasoningPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            reflector: None,
            memory: None,
        }
    }

    /// Builder: 注入阶段反思器。
    pub fn with_reflector(mut self, reflector: Arc<dyn PhaseReflector>) -> Self {
        self.reflector = Some(reflector);
        self
    }

    /// Builder: 注入记忆系统。
    pub fn with_memory(mut self, memory: Arc<dyn ReflectionMemory>) -> Self {
        self.memory = Some(memory);
        self
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

        let total_elapsed_ms = start.elapsed().as_millis() as u64;
        let budget_exhausted = !monitor.within_budget();

        // P0-2: 推理结果写入记忆
        if let Some(ref memory) = self.memory {
            let session_id = format!(
                "reasoning-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            );
            let phases_summary: Vec<(String, String)> = phases
                .iter()
                .map(|p| {
                    let excerpt = if p.llm_output.len() > 200 {
                        format!("{}...", &p.llm_output[..200])
                    } else {
                        p.llm_output.clone()
                    };
                    (p.phase_label.clone(), excerpt)
                })
                .collect();
            if let Err(e) = memory.store_reasoning_result(
                &session_id,
                problem,
                &phases_summary,
                total_elapsed_ms,
                budget_exhausted,
            ) {
                eprintln!("[reasoning] 写入推理结果到记忆失败: {}", e);
            }
        }

        ReasoningExecutionResult {
            phases,
            total_elapsed_ms,
            budget_exhausted,
            loop_detected,
        }
    }

    /// 带反思的推理执行 — 在 Correction 阶段调用反思器评估输出质量。
    ///
    /// 当 `should_retry=true` 且重试次数 < `max_retries` 时，
    /// 重新执行低分阶段，直至质量达标或重试次数耗尽。
    pub fn execute_with_reflection(
        &self,
        problem: &str,
        executor: &mut dyn ReasoningExecutor,
        budget: Option<ReasoningBudget>,
    ) -> ReasoningExecutionResult {
        let start = Instant::now();

        // 先执行常规推理流程
        let mut result = self.execute(problem, executor, budget.clone());

        // 如果没有注入反思器，直接返回
        let reflector = match self.reflector {
            Some(ref r) => r,
            None => {
                eprintln!("[reasoning] 无反思器，跳过反思阶段");
                return result;
            }
        };

        // 对已完成的阶段进行反思评估
        let max_retries = self.config.max_iterations;
        let mut retry_count = 0;

        loop {
            // 找到评分最低的阶段
            let mut lowest_score = 100.0;
            let mut lowest_idx = None;

            for (idx, phase_output) in result.phases.iter().enumerate() {
                let reflection =
                    reflector.reflect_phase(phase_output.phase, phase_output, &result.phases);

                // 写入记忆
                if let Some(ref memory) = self.memory {
                    let session_id = format!("reflection-{}", start.elapsed().as_millis());
                    if let Err(e) = memory.store_reflection(
                        &session_id,
                        phase_output.phase,
                        reflection.score,
                        reflection.passed,
                        &reflection.suggestions,
                    ) {
                        eprintln!("[reasoning] 写入反思记忆失败: {}", e);
                    }
                }

                if reflection.score < lowest_score {
                    lowest_score = reflection.score;
                    lowest_idx = Some((idx, reflection));
                }
            }

            // 检查是否需要重试
            if let Some((idx, reflection)) = lowest_idx {
                if !reflection.should_retry || retry_count >= max_retries {
                    eprintln!(
                        "[reasoning] 反思评估完成: 最低评分={:.1}, 重试次数={}/{}",
                        lowest_score, retry_count, max_retries
                    );
                    break;
                }

                retry_count += 1;
                eprintln!(
                    "[reasoning] 反思重试 #{}/{}: 阶段「{}」评分 {:.1}，低于阈值",
                    retry_count, max_retries, result.phases[idx].phase_label, lowest_score
                );

                // 重新执行低分阶段
                let phase = result.phases[idx].phase;
                let plan_steps = self.plan(problem);
                let step = plan_steps
                    .iter()
                    .find(|s| s.phase == phase)
                    .unwrap_or_else(|| &plan_steps[0]);

                let phase_start = Instant::now();
                match executor.execute_phase(phase, problem, &step.output, &result.phases) {
                    Ok(llm_output) => {
                        result.phases[idx] = PhaseExecutionOutput {
                            phase,
                            phase_label: phase.label().to_string(),
                            instructions: step.output.clone(),
                            llm_output,
                            elapsed_ms: phase_start.elapsed().as_millis() as u64,
                        };
                    }
                    Err(e) => {
                        eprintln!("[reasoning] 重试阶段执行失败: {}", e);
                        break;
                    }
                }
            } else {
                break;
            }
        }

        result.total_elapsed_ms = start.elapsed().as_millis() as u64;

        // P0-2: 反思后的最终推理结果写入记忆
        if let Some(ref memory) = self.memory {
            let session_id = format!(
                "reasoning-reflection-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            );
            let phases_summary: Vec<(String, String)> = result
                .phases
                .iter()
                .map(|p| {
                    let excerpt = if p.llm_output.len() > 200 {
                        format!("{}...", &p.llm_output[..200])
                    } else {
                        p.llm_output.clone()
                    };
                    (p.phase_label.clone(), excerpt)
                })
                .collect();
            if let Err(e) = memory.store_reasoning_result(
                &session_id,
                problem,
                &phases_summary,
                result.total_elapsed_ms,
                result.budget_exhausted,
            ) {
                eprintln!("[reasoning] 写入反思推理结果到记忆失败: {}", e);
            }
        }

        result
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

    /// 测试用的 PhaseReflector 实现：始终建议重试
    struct AlwaysRetryReflector;

    impl PhaseReflector for AlwaysRetryReflector {
        fn reflect_phase(
            &self,
            phase: ReasoningPhase,
            _phase_output: &PhaseExecutionOutput,
            _all_phases: &[PhaseExecutionOutput],
        ) -> PhaseReflectionResult {
            PhaseReflectionResult {
                passed: false,
                score: 50.0,
                should_retry: true,
                suggestions: vec![format!("阶段 {:?} 需要改进", phase)],
            }
        }
    }

    #[test]
    fn test_execute_with_reflection_no_reflector() {
        let pipeline = ReasoningPipeline::new(PipelineConfig {
            max_hypotheses: 5,
            max_iterations: 10, // 足够执行所有阶段
            min_confidence: 0.6,
        });
        let mut executor = NoopReasoningExecutor {
            model: "test".into(),
        };
        let result = pipeline.execute_with_reflection("测试", &mut executor, None);
        // 无反思器时行为与 execute 相同
        assert_eq!(result.phases.len(), 6);
    }

    #[test]
    fn test_execute_with_reflection_and_reflector() {
        let pipeline = ReasoningPipeline::new(PipelineConfig {
            max_hypotheses: 5,
            max_iterations: 1, // 只允许重试 1 次
            min_confidence: 0.6,
        })
        .with_reflector(Arc::new(AlwaysRetryReflector));

        let mut executor = NoopReasoningExecutor {
            model: "test".into(),
        };

        // 使用足够大的 budget 使 execute 能完成所有阶段
        let budget = ReasoningBudget {
            max_iterations: 10,
            current_iteration: 0,
            max_tokens: 100_000,
            tokens_used: 0,
        };
        let result = pipeline.execute_with_reflection("测试", &mut executor, Some(budget));
        // 应有阶段被重新执行，且结果包含所有 6 个阶段
        assert_eq!(result.phases.len(), 6);
    }

    /// 测试用的 ReflectionMemory 实现
    struct TestMemory {
        entries: std::sync::Mutex<Vec<String>>,
    }

    impl ReflectionMemory for TestMemory {
        fn store_reflection(
            &self,
            session_id: &str,
            phase: ReasoningPhase,
            score: f64,
            passed: bool,
            suggestions: &[String],
        ) -> Result<(), String> {
            let entry = format!(
                "{}:{:?}:{:.1}:{}:{:?}",
                session_id, phase, score, passed, suggestions
            );
            self.entries.lock().unwrap().push(entry);
            Ok(())
        }

        fn store_reasoning_result(
            &self,
            session_id: &str,
            problem: &str,
            phases_summary: &[(String, String)],
            total_elapsed_ms: u64,
            budget_exhausted: bool,
        ) -> Result<(), String> {
            let entry = format!(
                "result:{}:{}:{}phases:{}:exhausted:{}",
                session_id,
                problem,
                phases_summary.len(),
                total_elapsed_ms,
                budget_exhausted
            );
            self.entries.lock().unwrap().push(entry);
            Ok(())
        }
    }

    #[test]
    fn test_execute_with_reflection_and_memory() {
        let test_memory = Arc::new(TestMemory {
            entries: std::sync::Mutex::new(Vec::new()),
        });

        let pipeline = ReasoningPipeline::new(PipelineConfig {
            max_hypotheses: 5,
            max_iterations: 1,
            min_confidence: 0.6,
        })
        .with_reflector(Arc::new(AlwaysRetryReflector))
        .with_memory(test_memory.clone());

        let mut executor = NoopReasoningExecutor {
            model: "test".into(),
        };

        let budget = ReasoningBudget {
            max_iterations: 10,
            current_iteration: 0,
            max_tokens: 100_000,
            tokens_used: 0,
        };
        let result = pipeline.execute_with_reflection("测试", &mut executor, Some(budget));
        assert_eq!(result.phases.len(), 6);

        // 应有反思记忆写入
        let entries = test_memory.entries.lock().unwrap();
        assert!(!entries.is_empty());
    }

    #[test]
    fn test_execute_writes_reasoning_result_to_memory() {
        let test_memory = Arc::new(TestMemory {
            entries: std::sync::Mutex::new(Vec::new()),
        });

        let pipeline =
            ReasoningPipeline::new(PipelineConfig::default()).with_memory(test_memory.clone());

        let mut executor = NoopReasoningExecutor {
            model: "test".into(),
        };
        let _ = pipeline.execute("测试推理结果写入", &mut executor, None);

        let entries = test_memory.entries.lock().unwrap();
        assert!(entries.iter().any(|e| e.contains("reasoning-")));
    }
}
