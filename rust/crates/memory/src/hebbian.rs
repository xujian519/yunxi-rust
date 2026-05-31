//! 赫布路径优化器 — 工具间协同调用模式学习。
//!
//! 基于"一起激活的神经元连在一起"（Hebbian learning）原理，
//! 记录工具间协同调用模式并优化路由建议。

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

// ---- 常量 ----

const DEFAULT_LEARNING_RATE: f64 = 0.15;
const DEFAULT_DECAY_RATE: f64 = 0.01;
const STRONG_THRESHOLD: f64 = 0.7;
const MODERATE_THRESHOLD: f64 = 0.3;
const PATH_DIRECT_MIN: f64 = 0.5;
const PATH_HOP_MIN: f64 = 0.3;

// ---- 数据结构 ----

/// 连接状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Weak,
    Moderate,
    Strong,
    Permanent,
}

impl ConnectionState {
    pub fn from_strength(s: f64) -> Self {
        if s >= STRONG_THRESHOLD {
            Self::Strong
        } else if s >= MODERATE_THRESHOLD {
            Self::Moderate
        } else {
            Self::Weak
        }
    }
}

/// 神经连接（工具间协同权重）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NeuralConnection {
    /// 连接 ID：字典序拼接 `a<->b`。
    pub id: String,
    /// 源工具。
    pub source: String,
    /// 目标工具。
    pub target: String,
    /// 连接强度 [0, 1]。
    pub strength: f64,
    /// 连接状态。
    pub state: ConnectionState,
    /// 累计激活次数。
    pub activation_count: u32,
    /// 最近一次激活时间。
    pub last_activation: Option<DateTime<Utc>>,
}

/// 优化路径建议。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizationPath {
    /// 起始工具。
    pub from: String,
    /// 建议的后续工具列表（按权重降序）。
    pub suggestions: Vec<PathSuggestion>,
    /// 路径类型。
    pub path_type: PathType,
}

/// 单个路径建议。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathSuggestion {
    pub tool_id: String,
    pub strength: f64,
    pub state: ConnectionState,
}

/// 路径类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathType {
    /// 直接连接（源→目标 strength > 0.5）。
    Direct,
    /// 单跳连接（源→中间→目标，中间节点 strength > 0.3）。
    SingleHop,
    /// 无已知路径。
    None,
}

// ---- 优化器 ----

/// 赫布路径优化器。
pub struct HebbianOptimizer {
    conn: Connection,
    learning_rate: f64,
    decay_rate: f64,
}

impl HebbianOptimizer {
    /// 打开（或创建）赫布连接数据库。
    pub fn open(db_path: &Path) -> Result<Self, String> {
        Self::open_with_params(db_path, DEFAULT_LEARNING_RATE, DEFAULT_DECAY_RATE)
    }

    /// 打开并指定学习率和衰减率。
    pub fn open_with_params(
        db_path: &Path,
        learning_rate: f64,
        decay_rate: f64,
    ) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        let optimizer = Self {
            conn,
            learning_rate,
            decay_rate,
        };
        optimizer.init_schema()?;
        Ok(optimizer)
    }

    /// 加强源→目标连接（赫布学习核心操作）。
    pub fn strengthen(&self, source: &str, target: &str) -> Result<NeuralConnection, String> {
        let (id, src, tgt) = sorted_pair(source, target);
        let now = Utc::now();

        let existing = self.load_connection(&id)?;

        let (new_strength, new_count) = match existing {
            Some(mut c) => {
                c.strength = (c.strength + self.learning_rate).min(1.0);
                c.activation_count += 1;
                c.last_activation = Some(now);
                (c.strength, c.activation_count)
            }
            None => (self.learning_rate.min(1.0), 1),
        };

        let state = ConnectionState::from_strength(new_strength);

        self.conn.execute(
            "INSERT INTO hebbian_connections (id, source, target, strength, state, activation_count, last_activation)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                strength = excluded.strength,
                state = excluded.state,
                activation_count = excluded.activation_count,
                last_activation = excluded.last_activation",
            params![id, src, tgt, new_strength, state_as_str(state), new_count, now.to_rfc3339()],
        ).map_err(|e| e.to_string())?;

        Ok(NeuralConnection {
            id,
            source: src,
            target: tgt,
            strength: new_strength,
            state,
            activation_count: new_count,
            last_activation: Some(now),
        })
    }

    /// 对所有连接应用时间衰减。
    /// 返回衰减后仍存活的连接数。
    pub fn apply_decay(&self) -> Result<usize, String> {
        let connections = self.load_all_connections()?;
        let now = Utc::now();
        let mut alive = 0;

        for c in &connections {
            let last = c.last_activation.unwrap_or(now);
            let days = (now - last).num_days().max(0) as f64;
            let decayed = (c.strength - self.decay_rate * days).max(0.0);

            if decayed < 0.01 {
                // 衰减至近零，删除
                self.conn
                    .execute(
                        "DELETE FROM hebbian_connections WHERE id = ?1",
                        params![c.id],
                    )
                    .map_err(|e| e.to_string())?;
            } else {
                let state = ConnectionState::from_strength(decayed);
                self.conn
                    .execute(
                        "UPDATE hebbian_connections SET strength = ?1, state = ?2 WHERE id = ?3",
                        params![decayed, state_as_str(state), c.id],
                    )
                    .map_err(|e| e.to_string())?;
                alive += 1;
            }
        }

        Ok(alive)
    }

    /// 查找从指定工具出发的最优路径。
    pub fn find_optimal_path(&self, source: &str) -> Result<OptimizationPath, String> {
        // 直接连接
        let direct = self.connections_from(source)?;
        let direct_strong: Vec<PathSuggestion> = direct
            .iter()
            .filter(|c| c.strength >= PATH_DIRECT_MIN)
            .map(|c| PathSuggestion {
                tool_id: c.target.clone(),
                strength: c.strength,
                state: c.state,
            })
            .collect();

        if !direct_strong.is_empty() {
            return Ok(OptimizationPath {
                from: source.to_string(),
                suggestions: direct_strong,
                path_type: PathType::Direct,
            });
        }

        // 单跳路径：源→中间→目标
        let mut hop_suggestions: Vec<PathSuggestion> = Vec::new();
        for first_hop in &direct {
            if first_hop.strength < PATH_HOP_MIN {
                continue;
            }
            let second_hops = self.connections_from(&first_hop.target)?;
            for second in second_hops {
                if second.strength < PATH_HOP_MIN || second.target == source {
                    continue;
                }
                let combined = first_hop.strength * second.strength;
                hop_suggestions.push(PathSuggestion {
                    tool_id: second.target.clone(),
                    strength: combined,
                    state: ConnectionState::from_strength(combined),
                });
            }
        }

        // 去重并按权重排序
        let mut seen = HashMap::new();
        for s in hop_suggestions {
            seen.entry(s.tool_id.clone())
                .and_modify(|existing: &mut PathSuggestion| {
                    if s.strength > existing.strength {
                        *existing = s.clone();
                    }
                })
                .or_insert(s);
        }
        let mut suggestions: Vec<PathSuggestion> = seen.into_values().collect();
        suggestions.sort_by(|a, b| {
            b.strength
                .partial_cmp(&a.strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let path_type = if suggestions.is_empty() {
            PathType::None
        } else {
            PathType::SingleHop
        };

        Ok(OptimizationPath {
            from: source.to_string(),
            suggestions,
            path_type,
        })
    }

    /// 获取指定工具的所有连接（按 strength 降序）。
    /// 无向边：返回的连接中，source 始终为 tool_id，target 为对端工具。
    pub fn connections_from(&self, tool_id: &str) -> Result<Vec<NeuralConnection>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source, target, strength, state, activation_count, last_activation
             FROM hebbian_connections
             WHERE source = ?1 OR target = ?1
             ORDER BY strength DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![tool_id], |row| {
                let mut c = row_to_connection(row)?;
                if c.target == tool_id {
                    std::mem::swap(&mut c.source, &mut c.target);
                }
                Ok(c)
            })
            .map_err(|e| e.to_string())?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// 获取所有连接。
    fn load_all_connections(&self) -> Result<Vec<NeuralConnection>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source, target, strength, state, activation_count, last_activation
             FROM hebbian_connections ORDER BY strength DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], row_to_connection)
            .map_err(|e| e.to_string())?;
        Ok(rows.filter_map(std::result::Result::ok).collect())
    }

    /// 获取所有连接（公开）。
    pub fn all_connections(&self) -> Result<Vec<NeuralConnection>, String> {
        self.load_all_connections()
    }

    /// 统计连接数。
    pub fn connection_count(&self) -> Result<usize, String> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM hebbian_connections", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        Ok(count as usize)
    }

    /// 将连接设为永久（不受衰减影响）。
    pub fn make_permanent(&self, source: &str, target: &str) -> Result<(), String> {
        let (id, _, _) = sorted_pair(source, target);
        let changed = self
            .conn
            .execute(
                "UPDATE hebbian_connections SET state = 'permanent', strength = 1.0 WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err(format!("connection not found: {id}"));
        }
        Ok(())
    }

    /// 删除指定工具间的连接
    pub fn delete_connection(&self, source: &str, target: &str) -> Result<(), String> {
        let (id, _, _) = sorted_pair(source, target);
        let changed = self
            .conn
            .execute("DELETE FROM hebbian_connections WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            return Err(format!("连接未找到: {id}"));
        }
        Ok(())
    }

    fn load_connection(&self, id: &str) -> Result<Option<NeuralConnection>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source, target, strength, state, activation_count, last_activation
             FROM hebbian_connections WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
        match rows.next().map_err(|e| e.to_string())? {
            Some(row) => Ok(Some(row_to_connection(row).map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS hebbian_connections (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                strength REAL NOT NULL DEFAULT 0.0,
                state TEXT NOT NULL DEFAULT 'weak',
                activation_count INTEGER NOT NULL DEFAULT 0,
                last_activation TEXT,
                schema_version INTEGER NOT NULL DEFAULT 1
             );
             CREATE INDEX IF NOT EXISTS idx_hebbian_source ON hebbian_connections(source);
             CREATE INDEX IF NOT EXISTS idx_hebbian_target ON hebbian_connections(target);",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

// ---- 辅助函数 ----

/// 字典序排列工具对，生成确定性连接 ID。
fn sorted_pair(a: &str, b: &str) -> (String, String, String) {
    if a <= b {
        (format!("{a}<->{b}"), a.to_string(), b.to_string())
    } else {
        (format!("{b}<->{a}"), b.to_string(), a.to_string())
    }
}

fn state_as_str(s: ConnectionState) -> &'static str {
    match s {
        ConnectionState::Weak => "weak",
        ConnectionState::Moderate => "moderate",
        ConnectionState::Strong => "strong",
        ConnectionState::Permanent => "permanent",
    }
}

fn row_to_connection(
    row: &rusqlite::Row<'_>,
) -> std::result::Result<NeuralConnection, rusqlite::Error> {
    let id: String = row.get(0)?;
    let source: String = row.get(1)?;
    let target: String = row.get(2)?;
    let strength: f64 = row.get(3)?;
    let state_str: String = row.get(4)?;
    let activation_count: u32 = row.get(5)?;
    let last_activation_str: Option<String> = row.get(6)?;

    let state = match state_str.as_str() {
        "permanent" => ConnectionState::Permanent,
        "strong" => ConnectionState::Strong,
        "moderate" => ConnectionState::Moderate,
        _ => ConnectionState::Weak,
    };

    let last_activation = last_activation_str.and_then(|s| s.parse::<DateTime<Utc>>().ok());

    Ok(NeuralConnection {
        id,
        source,
        target,
        strength,
        state,
        activation_count,
        last_activation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_db() -> (PathBuf, HebbianOptimizer) {
        let uniq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "yunxi-hebbian-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            uniq
        ));
        let opt = HebbianOptimizer::open(&dir.join("hebbian.sqlite")).unwrap();
        (dir, opt)
    }

    #[test]
    fn strengthen_creates_connection() {
        let (_dir, opt) = temp_db();
        let c = opt.strengthen("search", "patent-analyzer").unwrap();
        assert_eq!(c.source, "patent-analyzer"); // 字典序：p < s
        assert_eq!(c.target, "search");
        assert!(c.strength > 0.0);
        assert_eq!(c.activation_count, 1);
        assert_eq!(c.state, ConnectionState::Weak);
    }

    #[test]
    fn strengthen_increments_strength() {
        let (_dir, opt) = temp_db();
        opt.strengthen("search", "patent-analyzer").unwrap();
        opt.strengthen("search", "patent-analyzer").unwrap();
        opt.strengthen("search", "patent-analyzer").unwrap();
        let c = opt
            .load_connection("patent-analyzer<->search")
            .unwrap()
            .unwrap();
        assert!(
            c.strength >= 0.4,
            "strength should be >= 0.4, got {}",
            c.strength
        );
        assert_eq!(c.activation_count, 3);
    }

    #[test]
    fn strength_capped_at_one() {
        let (_dir, opt) = temp_db();
        for _ in 0..20 {
            opt.strengthen("search", "patent-analyzer").unwrap();
        }
        let c = opt
            .load_connection("patent-analyzer<->search")
            .unwrap()
            .unwrap();
        assert!(c.strength <= 1.0);
    }

    #[test]
    fn find_optimal_path_direct() {
        let (_dir, opt) = temp_db();
        // 建立 strong 连接
        for _ in 0..5 {
            opt.strengthen("search", "patent-analyzer").unwrap();
        }
        let path = opt.find_optimal_path("search").unwrap();
        assert_eq!(path.path_type, PathType::Direct);
        assert!(!path.suggestions.is_empty());
    }

    #[test]
    fn find_optimal_path_none() {
        let (_dir, opt) = temp_db();
        // 使用唯一的工具名避免与其他并行测试冲突
        let path = opt
            .find_optimal_path("unique_nonexistent_tool_12345")
            .unwrap();
        assert_eq!(path.path_type, PathType::None);
        assert!(path.suggestions.is_empty());
    }

    #[test]
    fn connection_count() {
        let (_dir, opt) = temp_db();
        assert_eq!(opt.connection_count().unwrap(), 0);
        opt.strengthen("a", "b").unwrap();
        opt.strengthen("b", "c").unwrap();
        assert_eq!(opt.connection_count().unwrap(), 2);
    }

    #[test]
    fn connections_from_returns_related() {
        let (_dir, opt) = temp_db();
        opt.strengthen("search", "patent-analyzer").unwrap();
        opt.strengthen("search", "quality").unwrap();
        let conns = opt.connections_from("search").unwrap();
        assert_eq!(conns.len(), 2);
    }

    #[test]
    fn make_permanent() {
        let (_dir, opt) = temp_db();
        opt.strengthen("search", "patent-analyzer").unwrap();
        opt.make_permanent("search", "patent-analyzer").unwrap();
        let c = opt
            .load_connection("patent-analyzer<->search")
            .unwrap()
            .unwrap();
        assert_eq!(c.state, ConnectionState::Permanent);
        assert!((c.strength - 1.0).abs() < f64::EPSILON);
    }
}
