//! 统一记忆接口
//!
//! 桥接两套记忆系统：
//! - `MemoryStore`（文件记忆，`~/.yunxi/memory/`）
//! - `TieredMemoryStore`（SQLite 分层记忆，HOT/WARM/COLD/ETERNAL）
//!
//! 新增能力（Phase 2）：
//! - 语义检索（`SemanticSearch`，需启用 `semantic` feature）
//! - 混合检索（关键词 + 语义）
//! - 丰富元数据写入（标签、重要度、来源类型）
//!
//! 提供统一的 `UnifiedMemory` 门面，使调用方只需一个接口即可访问全部记忆。

use std::collections::HashSet;
use std::path::Path;

use chrono::Utc;
use serde::Serialize;

use crate::semantic::{SemanticSearch, SemanticSearchResult};
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
    semantic: SemanticSearch,
}

impl UnifiedMemory {
    /// 创建统一记忆实例
    ///
    /// # Arguments
    /// * `file_store` - 文件记忆存储（可选）
    /// * `tier_store` - 分层记忆存储（可选）
    /// * `semantic` - 语义搜索引擎
    pub fn new(
        file_store: Option<MemoryStore>,
        tier_store: Option<TieredMemoryStore>,
        semantic: SemanticSearch,
    ) -> Self {
        Self {
            file_store,
            tier_store,
            semantic,
        }
    }

    /// 使用默认路径的便捷构造。
    ///
    /// 语义搜索默认禁用（`SemanticSearch::disabled()`）。
    /// 可通过 `with_semantic()` builder 方法启用。
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
            semantic: SemanticSearch::disabled(),
        })
    }

    /// Builder: 启用语义搜索。
    pub fn with_semantic(mut self, semantic: SemanticSearch) -> Self {
        self.semantic = semantic;
        self
    }

    /// 写入一条记忆（存储到分层系统，并同步索引向量）。
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

            // 同步索引到语义搜索
            let metadata = serde_json::json!({
                "agent_id": agent_id,
                "session_id": session_id.unwrap_or(""),
                "tier": "Hot",
            });
            if let Err(e) = self.semantic.index(id, content, metadata) {
                eprintln!("[memory] 语义索引失败（非致命）: {e}");
            }
        }
        Ok(())
    }

    /// 丰富记忆写入，支持标签、重要度、来源等元数据。
    ///
    /// # Arguments
    /// * `id` - 记忆唯一标识
    /// * `agent_id` - 所属智能体
    /// * `content` - 记忆内容
    /// * `session_id` - 会话 ID（可选）
    /// * `tags` - 标签列表
    /// * `importance` - 重要度评分 (0.0 - 1.0)
    /// * `source_type` - 来源类型 (e.g. "reflection", "feedback", "user_input")
    pub fn remember_rich(
        &self,
        id: &str,
        agent_id: &str,
        content: &str,
        session_id: Option<&str>,
        tags: Vec<String>,
        importance: f64,
        source_type: &str,
    ) -> Result<(), String> {
        if let Some(ref store) = self.tier_store {
            // 根据 importance 决定初始层级
            let tier = if importance >= 0.9 {
                MemoryTier::Eternal
            } else {
                // 默认 Hot，后续通过迁移降级
                MemoryTier::Hot
            };

            let mut metadata = std::collections::HashMap::new();
            metadata.insert("tags".to_string(), tags.join(","));
            metadata.insert("importance".to_string(), format!("{:.2}", importance));
            metadata.insert("source_type".to_string(), source_type.to_string());

            let entry = TieredMemoryEntry {
                id: id.to_string(),
                tier,
                agent_id: agent_id.to_string(),
                session_id: session_id.map(str::to_string),
                content: content.to_string(),
                metadata,
                created_at: Utc::now(),
                accessed_at: Utc::now(),
                access_count: 0,
            };
            store.store(&entry)?;

            // 同步语义索引
            let index_meta = serde_json::json!({
                "agent_id": agent_id,
                "session_id": session_id.unwrap_or(""),
                "tier": tier.as_str(),
                "tags": tags,
                "importance": importance,
                "source_type": source_type,
            });
            if let Err(e) = self.semantic.index(id, content, index_meta) {
                eprintln!("[memory] 语义索引失败（非致命）: {e}");
            }
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

    /// 语义搜索。
    ///
    /// 委托给 `SemanticSearch`，语义未启用时返回空 Vec。
    pub fn semantic_search(&self, query: &str, limit: usize) -> Vec<SemanticSearchResult> {
        self.semantic.search(query, limit).unwrap_or_default()
    }

    /// 混合检索：关键词结果 + 语义结果，去重后合并。
    ///
    /// 当语义搜索不可用时，自动退化为纯关键词检索。
    pub fn hybrid_search(&self, query: &str, limit: usize) -> Vec<UnifiedMemoryEntry> {
        let mut seen_ids = HashSet::new();
        let mut results = Vec::new();

        // 1. 关键词检索
        let keyword_results = self.search(query, limit);
        for entry in &keyword_results {
            seen_ids.insert(entry.id.clone());
            results.push(entry.clone());
        }

        // 2. 语义检索补充
        let semantic_limit = limit.saturating_sub(results.len());
        if semantic_limit > 0 {
            if let Ok(semantic_results) = self.semantic.search(query, semantic_limit * 2) {
                for sr in semantic_results {
                    if seen_ids.insert(sr.id.clone()) {
                        // 通过 tier_store.retrieve 获取完整内容
                        if let Ok(Some(content)) = self.retrieve(&sr.id) {
                            results.push(UnifiedMemoryEntry {
                                id: sr.id,
                                content,
                                tier: "semantic".to_string(),
                                source: "semantic_search".to_string(),
                                created_at: String::new(),
                                access_count: 0,
                            });
                        }
                    }
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        results
    }

    /// 执行分层迁移（带频率加权）。
    ///
    /// HOT→WARM→COLD→evict，高频访问的记忆降级更慢。
    pub fn migrate(&self) -> Result<(), String> {
        if let Some(ref store) = self.tier_store {
            let report = store.run_tier_migration_with_frequency()?;
            eprintln!(
                "[memory] 记忆迁移（频率加权）: hot→warm={}, warm→cold={}, cold_evicted={}",
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

        let um = UnifiedMemory::new(Some(file_store), None, SemanticSearch::disabled());
        let results = um.search("用户偏好", 5);
        assert!(!results.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_remember_rich_with_metadata() {
        let db_path = format!(
            "/tmp/yunxi-test-remember-rich-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let tier_store =
            TieredMemoryStore::open(std::path::Path::new(&db_path)).expect("open tier store");

        let um = UnifiedMemory::new(None, Some(tier_store), SemanticSearch::disabled());

        um.remember_rich(
            "rich-001",
            "agent-a",
            "这是一条带丰富元数据的记忆",
            Some("session-1"),
            vec!["测试".to_string(), "元数据".to_string()],
            0.95,
            "reflection",
        )
        .unwrap();

        let content = um.retrieve("rich-001").unwrap().unwrap();
        assert_eq!(content, "这是一条带丰富元数据的记忆");
    }

    #[test]
    fn test_hybrid_search_returns_keyword_results() {
        let tmp = format!("/tmp/yunxi-test-hybrid-search-{}", std::process::id());
        let _ = std::fs::remove_dir_all(&tmp);

        let file_store = MemoryStore::new(&tmp);
        file_store.ensure_dirs().unwrap();
        file_store
            .store(
                crate::types::MemoryType::User,
                "hybrid-test",
                "混合检索测试内容",
                vec!["混合".into()],
            )
            .unwrap();

        let um = UnifiedMemory::new(Some(file_store), None, SemanticSearch::disabled());
        let results = um.hybrid_search("混合", 10);
        // 语义未启用时应仍然返回关键词结果
        assert!(!results.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
