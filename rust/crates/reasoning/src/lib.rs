//! 结构化推理引擎
//!
//! 基于 Athena `core/reasoning/athena_super_reasoning_v2.py` 重写。
//! 提供 6 阶段推理管道、假设管理和元认知监控。

mod executor;
mod hypothesis;
mod monitor;
mod pipeline;

pub use executor::{
    NoopReasoningExecutor, PhaseExecutionOutput, ReasoningExecutionResult, ReasoningExecutor,
};
pub use hypothesis::HypothesisManager;
pub use monitor::{MetaCognitiveMonitor, ReasoningBudget};
pub use pipeline::{
    PhaseReflectionResult, PhaseReflector, PipelineConfig, ReasoningPhase, ReasoningPipeline,
    ReasoningStepOutput, ReflectionMemory,
};
