//! 智能模型路由器 — 根据任务复杂度自动选择模型。

mod analyzer;
pub mod config;
mod scorer;
mod selector;
pub mod types;

pub use analyzer::TaskAnalyzer;
pub use config::RouterConfig;
pub use scorer::ComplexityScorer;
pub use selector::ModelSelector;
pub use types::{
    ComplexityScore, ModelSelection, RouterError, TaskContext, TaskFeatures, TaskType, UserInput,
};

#[cfg(test)]
mod e2e_tests;
