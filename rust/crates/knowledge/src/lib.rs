//! 云熙智能体 - 知识库引擎
//!
//! 提供法律法规数据库查询、知识卡片管理和统一搜索能力。

pub mod knowledge_cards;
pub mod law_db;
pub mod paths;
pub mod search;
pub mod semantic_index;
pub mod types;

pub use paths::ensure_user_knowledge_dirs;
pub use paths::KnowledgePaths;
pub use search::{SearchConfig, SearchMode, UnifiedSearch};
pub use semantic_index::{ChunkHit, PrebuiltSemanticIndex};
