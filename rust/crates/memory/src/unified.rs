//! 统一记忆接口
//!
//! 桥接两套记忆系统：
//! - `MemoryStore`（文件记忆，`~/.yunxi/memory/`）
//! - `TieredMemoryStore`（SQLite 分层记忆，HOT/WARM/COLD/ETERNAL）
//!
//! 提供统一的 `UnifiedMemory` 门面，使调用方只需一个接口即可访问全部记忆。

use std::path::Path;

use chrono::Utc;
use serde::Serialize;

use crate::store::MemoryStore;
use crate::tier::{MemoryTier, TieredMemoryEntry, TieredMemoryFilter, TieredMemoryStore};

/// 统一记忆操作结果
#[derive(Debug, Clone, Serialize)]
pub struct UnifiedMemoryEntry {
    pub id: String,
    pub content: String,
    pub tier: String,
    pub source: String,
    pub created_at: String,
    pub access_count: u64,
}

/// 统一记忆门面
pub struct UnifiedMemory {
    file_store: Option<MemoryStore>,
    tier_store: Option<TieredMemoryStore>,
}

impl UnifiedMemory {
    /// 创建统一记忆实例
    pub fn new(file_store: Option<MemoryStore>, tier_store: Option<TieredMemoryStore>) -> Self {
        Self {
            file_store,
            tier_store,
        }
    }

    /// 使用默认路径的便捷构造
    pub fn default_paths() -> Result<Self, String> {
        let file_store = Some(MemoryStore::default_path());
        if let Err(e) = file_store.as_ref().unwrap().ensure_dirs() {
            eprintln!("[memory] Failed to init file memory: {e}");
        }

        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let tier_db = Path::new(&home).join(".yunxi").join("tiered_memory.sqlite");
        let tier_store = TieredMemoryStore::open(&tier_db).ok();

        Ok(Self {
            file_store,
            tier_store,
        })
    }

    /// 写入一条记忆（存储到分层系统）
    pub fn remember(
        &self,
        id: &str,
        agent_id: &str,
        content: &str,
        session_id: Option<&str>,
    ) -> Result<(), String> {
        if let Some(ref store) = self.tier_store {
            let entry = TieredMemoryEntry {
                id: id.to_string(),
                tier: MemoryTier::Hot,
                agent_id: agent_id.to_string(),
                session_id: session_id.map(str::to_string),
                content: content.to_string(),
                metadata: std::collections::HashMap::new(),
                created_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 0,
            };
            store.store(&entry)?;
        }
        Ok(())
    }

    /// 按 ID 检索记忆
    pub fn retrieve(&self, id: &str) -> Result<Option<String>, String> {
        if let Some(ref store) = self.tier_store {
            if let Some(entry) = store.retrieve(id)? {
                return Ok(Some(entry.content));
            }
        }
        Ok(None)
    }

    /// 按关键词搜索记忆（文件 + 分层）
    pub fn search(&self, query: &str, limit: usize) -> Vec<UnifiedMemoryEntry> {
        let mut results = Vec::new();

        if let Some(ref store) = self.file_store {
            for entry in store.recall(query, limit) {
                results.push(UnifiedMemoryEntry {
                    id: entry
                        .path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    content: entry.content,
                    tier: "file".to_string(),
                    source: "file_store".to_string(),
                    created_at: entry.meta.created_at,
                    access_count: 0,
                });
            }
        }

        if let Some(ref store) = self.tier_store {
            let filter = TieredMemoryFilter {
                limit: Some(limit),
                ..Default::default()
            };
            if let Ok(entries) = store.query(filter) {
                for entry in entries {
                    results.push(UnifiedMemoryEntry {
                        id: entry.id,
                        content: entry.content,
                        tier: format!("{:?}", entry.tier),
                        source: "tier_store".to_string(),
                        created_at: entry.created_at.to_rfc3339(),
                        access_count: entry.access_count as u64,
                    });
                }
            }
        }

        results.truncate(limit);
        results
    }

    /// 执行分层迁移（HOT→WARM→COLD→evict）
    pub fn migrate(&self) -> Result<(), String> {
        if let Some(ref store) = self.tier_store {
            let report = store.run_tier_migration()?;
            eprintln!(
                "[memory] 记忆迁移: hot→warm={}, warm→cold={}, cold_evicted={}",
                report.hot_to_warm, report.warm_to_cold, report.cold_evicted
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_memory_search() {
        let tmp = format!("/tmp/yunxi-test-unified-memory-{}", std::process::id());
        let _ = std::fs::remove_dir_all(&tmp);

        let file_store = MemoryStore::new(&tmp);
        file_store.ensure_dirs().unwrap();
        file_store
            .store(
                crate::types::MemoryType::User,
                "test-user",
                "用户偏好测试内容",
                vec!["测试".into()],
            )
            .unwrap();

        let um = UnifiedMemory::new(Some(file_store), None);
        let results = um.search("用户偏好", 5);
        assert!(!results.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
