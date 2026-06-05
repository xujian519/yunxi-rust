//! 云熙智能体 - 持久化记忆系统
//!
//! 基于文件的持久化记忆，支持 YAML frontmatter 格式，
//! 按类型分类存储，支持关键词检索。
//!
//! 核心能力：
//! - Hebbian 关联学习（hebbian.rs）
//! - 四层记忆分级管理（tier.rs：HOT/WARM/COLD/ETERNAL）
//! - 统一记忆门面（unified.rs：桥接文件+分层）
//! - 语义检索增强（semantic.rs：需启用 `semantic` feature）

pub mod context;
pub mod frontmatter;
pub mod hebbian;
pub mod relevance;
pub mod semantic;
pub mod store;
pub mod tier;
pub mod types;
pub mod unified;

pub use context::{build_context_section, search_report, DEFAULT_CONTEXT_LIMIT};
pub use hebbian::{
    ConnectionState, HebbianOptimizer, NeuralConnection, OptimizationPath, PathSuggestion, PathType,
};
pub use semantic::{SemanticSearch, SemanticSearchResult};
pub use store::MemoryStore;
pub use tier::{
    MemoryTier, TierMigrationReport, TieredMemoryEntry, TieredMemoryFilter, TieredMemoryStore,
};
pub use types::{MemoryEntry, MemoryType};
pub use unified::UnifiedMemory;
