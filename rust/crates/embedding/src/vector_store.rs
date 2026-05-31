//! 轻量向量存储
//!
//! 替代 Athena 的 Qdrant，使用 SQLite 持久化向量数据。
//! 支持集合管理、upsert、余弦相似度搜索。

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::service::Embedding;

/// 向量搜索结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

/// 向量存储错误类型
#[derive(Debug, thiserror::Error)]
pub enum VectorStoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
}

/// 轻量向量存储（基于 SQLite）
pub struct VectorStore {
    conn: Connection,
    dim: usize,
}

impl VectorStore {
    /// 打开或创建向量存储
    ///
    /// 数据库文件路径：`path`
    /// `dim`: 向量维度（通常 1024）
    pub fn open(path: &Path, dim: usize) -> Result<Self, VectorStoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        let store = Self { conn, dim };
        store.ensure_schema()?;
        Ok(store)
    }

    /// 使用默认路径打开
    pub fn open_default(dim: usize) -> Result<Self, VectorStoreError> {
        let path = dirs::home_dir()
            .unwrap_or_default()
            .join(".yunxi/vectors/vectors.db");
        Self::open(&path, dim)
    }

    /// 确保/获取指定集合（自动建表）
    fn ensure_collection(&self, collection: &str) -> Result<(), VectorStoreError> {
        let table_name = sanitize_collection_name(collection);
        self.conn.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS \"{table_name}\" (
                id TEXT PRIMARY KEY,
                vector BLOB NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{{}}',
                created_at REAL NOT NULL DEFAULT (strftime('%s','now'))
            );
            CREATE INDEX IF NOT EXISTS \"{table_name}_id_idx\" ON \"{table_name}\"(id);"
        ))?;
        Ok(())
    }

    fn ensure_schema(&self) -> Result<(), VectorStoreError> {
        // 元数据表
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    /// 插入或更新向量
    pub fn upsert(
        &self,
        collection: &str,
        id: &str,
        vector: &[f32],
        metadata: serde_json::Value,
    ) -> Result<(), VectorStoreError> {
        self.ensure_collection(collection)?;
        let table_name = sanitize_collection_name(collection);

        if vector.len() != self.dim {
            return Err(VectorStoreError::DimensionMismatch {
                expected: self.dim,
                actual: vector.len(),
            });
        }

        let vector_blob = encode_vector(vector);
        let metadata_str = serde_json::to_string(&metadata).unwrap_or_default();

        self.conn.execute(
            &format!(
                "INSERT OR REPLACE INTO \"{table_name}\" (id, vector, metadata) VALUES (?1, ?2, ?3)"
            ),
            params![id, vector_blob, metadata_str],
        )?;
        Ok(())
    }

    /// 批量插入或更新
    ///
    /// Returns the number of successfully inserted items.
    /// Items with dimension mismatch are skipped and counted in the returned skipped count.
    pub fn upsert_batch(
        &self,
        collection: &str,
        items: &[(String, Embedding, serde_json::Value)],
    ) -> Result<(usize, usize), VectorStoreError> {
        self.ensure_collection(collection)?;
        let table_name = sanitize_collection_name(collection);

        let mut inserted = 0;
        let mut skipped = 0;
        let tx = self.conn.unchecked_transaction()?;
        for (id, vector, metadata) in items {
            if vector.len() != self.dim {
                skipped += 1;
                continue;
            }
            let vector_blob = encode_vector(vector);
            let metadata_str = serde_json::to_string(metadata).unwrap_or_default();
            tx.execute(
                &format!(
                    "INSERT OR REPLACE INTO \"{table_name}\" (id, vector, metadata) VALUES (?1, ?2, ?3)"
                ),
                params![id, vector_blob, metadata_str],
            )?;
            inserted += 1;
        }
        tx.commit()?;
        Ok((inserted, skipped))
    }

    /// 删除向量
    pub fn delete(&self, collection: &str, id: &str) -> Result<bool, VectorStoreError> {
        let table_name = sanitize_collection_name(collection);
        let rows = self.conn.execute(
            &format!("DELETE FROM \"{table_name}\" WHERE id = ?1"),
            params![id],
        )?;
        Ok(rows > 0)
    }

    /// 余弦相似度搜索（暴力扫描，适合中小规模数据集）
    ///
    /// 返回按相似度降序排列的 top-k 结果
    pub fn search(
        &self,
        collection: &str,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>, VectorStoreError> {
        self.ensure_collection(collection)?;
        let table_name = sanitize_collection_name(collection);

        let query_norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
        if query_norm == 0.0 {
            return Ok(vec![]);
        }

        let mut stmt = self.conn.prepare(&format!(
            "SELECT id, vector, metadata FROM \"{table_name}\""
        ))?;

        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let vector_blob: Vec<u8> = row.get(1)?;
            let metadata_str: String = row.get(2)?;
            Ok((id, vector_blob, metadata_str))
        })?;

        let mut results: Vec<VectorSearchResult> = Vec::new();

        for row in rows {
            let (id, vector_blob, metadata_str) = row?;
            let vector = decode_vector(&vector_blob);

            let dot: f32 = query.iter().zip(vector.iter()).map(|(a, b)| a * b).sum();
            let vec_norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            let score = if vec_norm > 0.0 {
                dot / (query_norm * vec_norm)
            } else {
                0.0
            };

            let metadata: serde_json::Value =
                serde_json::from_str(&metadata_str).unwrap_or(serde_json::Value::Null);

            results.push(VectorSearchResult {
                id,
                score,
                metadata,
            });
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        Ok(results)
    }

    /// 获取指定向量
    pub fn get(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<(Embedding, serde_json::Value)>, VectorStoreError> {
        let table_name = sanitize_collection_name(collection);
        let result: Option<(Vec<u8>, String)> = self
            .conn
            .query_row(
                &format!("SELECT vector, metadata FROM \"{table_name}\" WHERE id = ?1"),
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;

        match result {
            Some((blob, meta_str)) => {
                let vector = decode_vector(&blob);
                let metadata = serde_json::from_str(&meta_str).unwrap_or(serde_json::Value::Null);
                Ok(Some((vector, metadata)))
            }
            None => Ok(None),
        }
    }

    /// 获取集合中的向量数量
    pub fn count(&self, collection: &str) -> Result<usize, VectorStoreError> {
        let table_name = sanitize_collection_name(collection);
        let count: usize = self
            .conn
            .query_row(
                &format!("SELECT COUNT(*) FROM \"{table_name}\""),
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(count)
    }

    /// 列出所有集合
    pub fn list_collections(&self) -> Result<Vec<String>, VectorStoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '_%' AND name NOT LIKE 'sqlite_%'",
        )?;
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(names)
    }
}

/// 编码向量为 BLOB（little-endian f32）
fn encode_vector(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for &f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

/// 解码 BLOB 为向量
fn decode_vector(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// 清理集合名（防止 SQL 注入）
fn sanitize_collection_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn make_store() -> (VectorStore, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.db");
        let store = VectorStore::open(&path, 4).unwrap();
        (store, path)
    }

    #[test]
    fn test_upsert_and_search() {
        let (store, _) = make_store();

        store
            .upsert(
                "test",
                "doc1",
                &[1.0, 0.0, 0.0, 0.0],
                serde_json::json!({"title": "patent 1"}),
            )
            .unwrap();
        store
            .upsert(
                "test",
                "doc2",
                &[0.0, 1.0, 0.0, 0.0],
                serde_json::json!({"title": "patent 2"}),
            )
            .unwrap();

        let results = store.search("test", &[1.0, 0.0, 0.0, 0.0], 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "doc1");
        assert!(results[0].score > 0.99);
    }

    #[test]
    fn test_get() {
        let (store, _) = make_store();

        store
            .upsert(
                "test",
                "doc1",
                &[1.0, 2.0, 3.0, 4.0],
                serde_json::json!({"x": 1}),
            )
            .unwrap();

        let (vector, meta) = store.get("test", "doc1").unwrap().unwrap();
        assert_eq!(vector, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(meta["x"], 1);
    }

    #[test]
    fn test_delete() {
        let (store, _) = make_store();

        store
            .upsert("test", "doc1", &[1.0, 0.0, 0.0, 0.0], serde_json::json!({}))
            .unwrap();
        assert!(store.delete("test", "doc1").unwrap());
        assert!(store.get("test", "doc1").unwrap().is_none());
    }

    #[test]
    fn test_count() {
        let (store, _) = make_store();

        store
            .upsert("test", "doc1", &[0.0; 4], serde_json::json!({}))
            .unwrap();
        store
            .upsert("test", "doc2", &[0.0; 4], serde_json::json!({}))
            .unwrap();
        assert_eq!(store.count("test").unwrap(), 2);
    }

    #[test]
    fn test_batch_upsert() {
        let (store, _) = make_store();

        let items = vec![
            (
                "doc1".into(),
                vec![1.0, 0.0, 0.0, 0.0],
                serde_json::json!({}),
            ),
            (
                "doc2".into(),
                vec![0.0, 1.0, 0.0, 0.0],
                serde_json::json!({}),
            ),
        ];
        let (inserted, _) = store.upsert_batch("test", &items).unwrap();
        assert_eq!(inserted, 2);
        assert_eq!(store.count("test").unwrap(), 2);
    }

    #[test]
    fn test_dimension_mismatch() {
        let (store, _) = make_store();
        let result = store.upsert("test", "doc1", &[1.0, 0.0], serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_collection_name() {
        assert_eq!(
            sanitize_collection_name("normal-name_123"),
            "normal-name_123"
        );
        assert_eq!(
            sanitize_collection_name("bad'; DROP TABLE--"),
            "badDROPTABLE--"
        );
    }
}
