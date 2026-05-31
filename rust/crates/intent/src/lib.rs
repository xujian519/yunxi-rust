//! 意图识别引擎
//!
//! 基于 Athena 意图识别系统重写。
//! 支持两层分类：
//! - 第一层：关键词领域检测（复用 router crate）
//! - 第二层：细粒度意图分类（50 个专利法律意图类别）
//!
//! 分类策略：关键词匹配 + BGE-M3 嵌入相似度

mod classifier;
mod complexity;
mod intent_types;

pub use classifier::{IntentClassifier, IntentResult};
pub use complexity::ComplexityAssessor;
pub use intent_types::IntentType;
