//! 记忆类型定义

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 记忆类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    User,
    Feedback,
    Project,
    Reference,
}

/// 记忆元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMeta {
    pub memory_type: MemoryType,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub path: PathBuf,
    pub meta: MemoryMeta,
    pub content: String,
}
