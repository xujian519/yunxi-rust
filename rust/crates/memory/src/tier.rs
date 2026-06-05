//! 四层记忆分级管理器 (HOT / WARM / COLD / ETERNAL)。
//!
//! 基于访问频率和时间自动在层级间迁移记忆条目。

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// 记忆层级。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryTier {
    /// 热层：当前对话上下文（本轮使用），RAM 持有。
    Hot,
    /// 暖层：近期会话摘要，SQLite 持久化。
    Warm,
    /// 冷层：长期归档知识，SQLite + 可选压缩。
    Cold,
    /// 永恒层：不可变核心知识（如用户偏好、法律规则）。
    Eternal,
}

impl MemoryTier {
    /// SQLite 存储用的字符串表示。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hot => "hot",
            Self::Warm => "warm",
            Self::Cold => "cold",
            Self::Eternal => "eternal",
        }
    }

    /// 从字符串解析层级。
    /// 未知值默认降级到 Warm，保持向前兼容。
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "hot" => Self::Hot,
            "warm" => Self::Warm,
            "cold" => Self::Cold,
            "eternal" => Self::Eternal,
            _ => Self::Warm,
        }
    }
}

/// 单条记忆。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredMemoryEntry {
    pub id: String,
    pub tier: MemoryTier,
    pub agent_id: String,
    pub session_id: Option<String>,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub access_count: u32,
}

/// 记忆查询过滤条件。
#[derive(Debug, Clone, Default)]
pub struct TieredMemoryFilter {
    pub tier: Option<MemoryTier>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub limit: Option<usize>,
}

/// 四层记忆存储管理器。
pub struct TieredMemoryStore {
    conn: Connection,
}

impl TieredMemoryStore {
    /// 打开（或创建）记忆数据库。
    pub fn open(db_path: &Path) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// 写入一条记忆（已存在则更新）。
    pub fn store(&self, entry: &TieredMemoryEntry) -> Result<(), String> {
        let metadata_json = serde_json::to_string(&entry.metadata).map_err(|e| e.to_string())?;
        self.conn.execute(
            "INSERT INTO tiered_memories (id, tier, agent_id, session_id, content, metadata, created_at, accessed_at, access_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                tier = excluded.tier,
                agent_id = excluded.agent_id,
                session_id = excluded.session_id,
                content = excluded.content,
                metadata = excluded.metadata,
                accessed_at = excluded.accessed_at,
                access_count = excluded.access_count",
            params![
                entry.id,
                entry.tier.as_str(),
                entry.agent_id,
                entry.session_id,
                entry.content,
                metadata_json,
                entry.created_at.to_rfc3339(),
                entry.accessed_at.to_rfc3339(),
                entry.access_count,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 按 ID 读取一条记忆。
    pub fn retrieve(&self, id: &str) -> Result<Option<TieredMemoryEntry>, String> {
        let mut stmt = self.conn.prepare(
            "SELECT id, tier, agent_id, session_id, content, metadata, created_at, accessed_at, access_count
             FROM tiered_memories WHERE id = ?1",
        ).map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(row_to_entry(row).map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    /// 按过滤条件查询记忆。
    pub fn query(&self, filter: TieredMemoryFilter) -> Result<Vec<TieredMemoryEntry>, String> {
        let mut sql = String::from(
            "SELECT id, tier, agent_id, session_id, content, metadata, created_at, accessed_at, access_count
             FROM tiered_memories WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref tier) = filter.tier {
            sql.push_str(" AND tier = ?");
            param_values.push(Box::new(tier.as_str().to_string()));
        }
        if let Some(ref agent_id) = filter.agent_id {
            sql.push_str(" AND agent_id = ?");
            param_values.push(Box::new(agent_id.clone()));
        }
        if let Some(ref session_id) = filter.session_id {
            sql.push_str(" AND session_id = ?");
            param_values.push(Box::new(session_id.clone()));
        }

        sql.push_str(" ORDER BY accessed_at DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(" LIMIT ?");
            param_values.push(Box::new(limit as i64));
        }

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_refs.as_slice(), row_to_entry)
            .map_err(|e| e.to_string())?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| e.to_string())?);
        }
        Ok(results)
    }

    /// 更新访问时间和计数。
    pub fn touch(&self, id: &str) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE tiered_memories SET accessed_at = ?1, access_count = access_count + 1 WHERE id = ?2",
            params![now, id],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 更新记忆层级。
    pub fn update_tier(&self, id: &str, new_tier: MemoryTier) -> Result<(), String> {
        let changed = self
            .conn
            .execute(
                "UPDATE tiered_memories SET tier = ?1 WHERE id = ?2",
                params![new_tier.as_str(), id],
            )
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err(format!("memory entry not found: {id}"));
        }
        Ok(())
    }

    /// 按 ID 删除一条记忆
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let changed = self
            .conn
            .execute("DELETE FROM tiered_memories WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err(format!("记忆条目未找到: {id}"));
        }
        Ok(())
    }

    /// 将指定层级中超过 max_days 天未访问的条目降级。
    pub fn downgrade_older_than(
        &self,
        from_tier: MemoryTier,
        max_days: u32,
        to_tier: MemoryTier,
    ) -> Result<usize, String> {
        let cutoff = Utc::now() - chrono::Duration::days(max_days as i64);
        let cutoff_str = cutoff.to_rfc3339();
        let changed = self
            .conn
            .execute(
                "UPDATE tiered_memories SET tier = ?1 WHERE tier = ?2 AND accessed_at < ?3",
                params![to_tier.as_str(), from_tier.as_str(), cutoff_str],
            )
            .map_err(|e| e.to_string())?;
        Ok(changed)
    }

    /// 删除指定层级中超过 max_days 天未访问的条目。
    pub fn evict_older_than(&self, tier: MemoryTier, max_days: u32) -> Result<usize, String> {
        let cutoff = Utc::now() - chrono::Duration::days(max_days as i64);
        let cutoff_str = cutoff.to_rfc3339();
        let deleted = self
            .conn
            .execute(
                "DELETE FROM tiered_memories WHERE tier = ?1 AND accessed_at < ?2",
                params![tier.as_str(), cutoff_str],
            )
            .map_err(|e| e.to_string())?;
        Ok(deleted)
    }

    /// 统计指定层级的条目数。
    pub fn count_by_tier(&self, tier: MemoryTier) -> Result<usize, String> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM tiered_memories WHERE tier = ?1",
                params![tier.as_str()],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(count as usize)
    }

    /// 执行层级迁移（按访问频率）。
    /// Hot → Warm: 超过 1 天未访问
    /// Warm → Cold: 超过 7 天未访问
    /// Cold → 删除: 超过 30 天未访问
    pub fn run_tier_migration(&self) -> Result<TierMigrationReport, String> {
        let hot_to_warm = self.downgrade_older_than(MemoryTier::Hot, 1, MemoryTier::Warm)?;
        let warm_to_cold = self.downgrade_older_than(MemoryTier::Warm, 7, MemoryTier::Cold)?;
        let cold_evicted = self.evict_older_than(MemoryTier::Cold, 30)?;

        Ok(TierMigrationReport {
            hot_to_warm,
            warm_to_cold,
            cold_evicted,
        })
    }

    /// 带频率加权的层级迁移。
    ///
    /// 高频访问的记忆降级更慢：
    ///   effective_age = actual_age / (1 + ln(access_count + 1))
    /// access_count >= 10 的条目不会被降级。
    pub fn run_tier_migration_with_frequency(&self) -> Result<TierMigrationReport, String> {
        // Hot → Warm: effective_age > 1 天 AND access_count < 10
        let hot_to_warm = self.downgrade_with_frequency(MemoryTier::Hot, 1, MemoryTier::Warm)?;
        // Warm → Cold: effective_age > 7 天 AND access_count < 10
        let warm_to_cold = self.downgrade_with_frequency(MemoryTier::Warm, 7, MemoryTier::Cold)?;
        // Cold → evict: effective_age > 30 天（无频率保护）
        let cold_evicted = self.evict_older_than(MemoryTier::Cold, 30)?;

        Ok(TierMigrationReport {
            hot_to_warm,
            warm_to_cold,
            cold_evicted,
        })
    }

    /// 按频率加权执行层级降级。
    ///
    /// 频率保护规则：
    /// - access_count >= 10 的条目跳过
    /// - effective_age = actual_age / (1 + ln(access_count + 1))
    fn downgrade_with_frequency(
        &self,
        from_tier: MemoryTier,
        max_days: u32,
        to_tier: MemoryTier,
    ) -> Result<usize, String> {
        let filter = TieredMemoryFilter {
            tier: Some(from_tier),
            ..Default::default()
        };
        let entries = self.query(filter)?;

        let cutoff = Utc::now() - chrono::Duration::days(max_days as i64);
        let mut to_downgrade = Vec::new();

        for entry in entries {
            // 高频保护：access_count >= 10 不降级
            if entry.access_count >= 10 {
                continue;
            }
            // 频率加权：有效年龄 = 实际年龄 / (1 + ln(access_count + 1))
            let age_days = (Utc::now() - entry.accessed_at).num_days().max(0) as f64;
            let freq_factor = 1.0 + (entry.access_count as f64 + 1.0).ln();
            let effective_age = age_days / freq_factor;

            if entry.accessed_at < cutoff || effective_age > max_days as f64 {
                to_downgrade.push(entry.id.clone());
            }
        }

        let count = to_downgrade.len();
        for id in to_downgrade {
            self.update_tier(&id, to_tier)?;
        }
        Ok(count)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS tiered_memories (
                id TEXT PRIMARY KEY,
                tier TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                session_id TEXT,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at TEXT NOT NULL,
                accessed_at TEXT NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 0,
                schema_version INTEGER NOT NULL DEFAULT 1
             );
             CREATE INDEX IF NOT EXISTS idx_tiered_tier ON tiered_memories(tier);
             CREATE INDEX IF NOT EXISTS idx_tiered_agent ON tiered_memories(agent_id);
             CREATE INDEX IF NOT EXISTS idx_tiered_session ON tiered_memories(session_id);",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// 层级迁移报告。
#[derive(Debug, Clone, Serialize)]
pub struct TierMigrationReport {
    pub hot_to_warm: usize,
    pub warm_to_cold: usize,
    pub cold_evicted: usize,
}

fn row_to_entry(
    row: &rusqlite::Row<'_>,
) -> std::result::Result<TieredMemoryEntry, rusqlite::Error> {
    let id: String = row.get(0)?;
    let tier_str: String = row.get(1)?;
    let agent_id: String = row.get(2)?;
    let session_id: Option<String> = row.get(3)?;
    let content: String = row.get(4)?;
    let metadata_json: Option<String> = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let accessed_at_str: String = row.get(7)?;
    let access_count: u32 = row.get(8)?;

    let metadata: HashMap<String, String> = metadata_json
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default();

    let created_at = created_at_str.parse().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(e))
    })?;
    let accessed_at = accessed_at_str.parse().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e))
    })?;

    Ok(TieredMemoryEntry {
        id,
        tier: MemoryTier::from_str_lossy(&tier_str),
        agent_id,
        session_id,
        content,
        metadata,
        created_at,
        accessed_at,
        access_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_db() -> (PathBuf, TieredMemoryStore) {
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "yunxi-tiered-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            uniq
        ));
        let store = TieredMemoryStore::open(&dir.join("tiered.sqlite")).unwrap();
        (dir, store)
    }

    fn make_entry(id: &str, tier: MemoryTier, agent: &str) -> TieredMemoryEntry {
        TieredMemoryEntry {
            id: id.to_string(),
            tier,
            agent_id: agent.to_string(),
            session_id: Some("sess-1".to_string()),
            content: format!("content of {id}"),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            accessed_at: Utc::now(),
            access_count: 0,
        }
    }

    #[test]
    fn round_trip() {
        let (_dir, store) = temp_db();
        let entry = make_entry("m1", MemoryTier::Hot, "agent-a");
        store.store(&entry).unwrap();

        let loaded = store.retrieve("m1").unwrap().unwrap();
        assert_eq!(loaded.id, "m1");
        assert_eq!(loaded.tier, MemoryTier::Hot);
        assert_eq!(loaded.agent_id, "agent-a");
    }

    #[test]
    fn query_by_tier() {
        let (_dir, store) = temp_db();
        store
            .store(&make_entry("h1", MemoryTier::Hot, "a"))
            .unwrap();
        store
            .store(&make_entry("h2", MemoryTier::Hot, "a"))
            .unwrap();
        store
            .store(&make_entry("w1", MemoryTier::Warm, "a"))
            .unwrap();

        let hot = store
            .query(TieredMemoryFilter {
                tier: Some(MemoryTier::Hot),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(hot.len(), 2);
    }

    #[test]
    fn touch_increments_access_count() {
        let (_dir, store) = temp_db();
        store
            .store(&make_entry("m1", MemoryTier::Hot, "a"))
            .unwrap();
        store.touch("m1").unwrap();
        store.touch("m1").unwrap();

        let loaded = store.retrieve("m1").unwrap().unwrap();
        assert_eq!(loaded.access_count, 2);
    }

    #[test]
    fn update_tier() {
        let (_dir, store) = temp_db();
        store
            .store(&make_entry("m1", MemoryTier::Hot, "a"))
            .unwrap();
        store.update_tier("m1", MemoryTier::Cold).unwrap();

        let loaded = store.retrieve("m1").unwrap().unwrap();
        assert_eq!(loaded.tier, MemoryTier::Cold);
    }

    #[test]
    fn tier_migration() {
        let (_dir, store) = temp_db();
        // 创建条目
        store
            .store(&make_entry("old_hot", MemoryTier::Hot, "a"))
            .unwrap();
        store
            .store(&make_entry("old_warm", MemoryTier::Warm, "a"))
            .unwrap();
        store
            .store(&make_entry("old_cold", MemoryTier::Cold, "a"))
            .unwrap();

        // 修改访问时间为很久以前
        let old = (Utc::now() - chrono::Duration::days(100)).to_rfc3339();
        store
            .conn
            .execute("UPDATE tiered_memories SET accessed_at = ?1", params![old])
            .unwrap();

        let report = store.run_tier_migration().unwrap();
        assert!(report.cold_evicted > 0);
    }

    #[test]
    fn test_frequency_migration_protects_hot_entries() {
        let (_dir, store) = temp_db();

        // 创建高频条目 (access_count = 15)
        let mut hot_entry = make_entry("hot-freq", MemoryTier::Hot, "a");
        hot_entry.access_count = 15;
        store.store(&hot_entry).unwrap();

        // 创建低频条目 (access_count = 2)
        let mut cold_entry = make_entry("cold-rare", MemoryTier::Hot, "a");
        cold_entry.access_count = 2;
        store.store(&cold_entry).unwrap();

        // 修改访问时间为很久以前
        let old = (Utc::now() - chrono::Duration::days(100)).to_rfc3339();
        store
            .conn
            .execute("UPDATE tiered_memories SET accessed_at = ?1", params![old])
            .unwrap();

        let report = store.run_tier_migration_with_frequency().unwrap();

        // 高频条目应保留在 Hot
        let loaded_freq = store.retrieve("hot-freq").unwrap().unwrap();
        assert_eq!(loaded_freq.tier, MemoryTier::Hot, "高频条目不应被降级");

        // 低频条目应已降级
        assert!(report.hot_to_warm >= 1, "应有低频条目被降级");
    }

    #[test]
    fn test_frequency_migration_migrates_cold_entries() {
        let (_dir, store) = temp_db();

        // 创建低频 Warm 条目
        let mut warm_entry = make_entry("warm-old", MemoryTier::Warm, "a");
        warm_entry.access_count = 1;
        store.store(&warm_entry).unwrap();

        // 修改访问时间为很久以前
        let old = (Utc::now() - chrono::Duration::days(100)).to_rfc3339();
        store
            .conn
            .execute("UPDATE tiered_memories SET accessed_at = ?1", params![old])
            .unwrap();

        let report = store.run_tier_migration_with_frequency().unwrap();
        assert!(report.warm_to_cold >= 1, "低频旧条目应被降级到 Cold");
    }
}
