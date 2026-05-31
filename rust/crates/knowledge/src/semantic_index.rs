//! 预构建语义索引（`.yunpat-semantic-index.sqlite`）
//!
//! 与 yunpat-agent 知识库索引格式兼容：21179+ chunks，BGE-M3 1024 维向量已写入 SQLite。
//! 检索时仅需对查询文本调用嵌入服务（如 HTTP :8766），无需对文档重新向量化。

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

/// 单条 chunk 检索命中
#[derive(Debug, Clone)]
pub struct ChunkHit {
    pub chunk_id: String,
    pub file_path: String,
    pub title: String,
    pub content: String,
    pub score: f32,
}

#[derive(Debug, Error)]
pub enum SemanticIndexError {
    #[error("open index: {0}")]
    Open(#[from] rusqlite::Error),
    #[error("invalid embedding blob: dim={dim}, bytes={bytes}")]
    InvalidBlob { dim: usize, bytes: usize },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

struct VectorDocument {
    doc_id: String,
    title: String,
    content: String,
    embedding: Vec<f32>,
    file_path: String,
}

struct InMemoryChunkIndex {
    documents: Vec<VectorDocument>,
    embedding_model: Option<String>,
    db_path: PathBuf,
}

/// 只读预构建语义索引
pub struct PrebuiltSemanticIndex {
    inner: InMemoryChunkIndex,
}

static GLOBAL_INDEX: OnceLock<Option<PrebuiltSemanticIndex>> = OnceLock::new();

fn load_global() -> Option<&'static PrebuiltSemanticIndex> {
    GLOBAL_INDEX
        .get_or_init(|| {
            let path = crate::paths::KnowledgePaths::discover().semantic_index_db?;
            PrebuiltSemanticIndex::open(&path).ok()
        })
        .as_ref()
}

impl PrebuiltSemanticIndex {
    /// 进程级单例（默认路径）
    #[must_use]
    pub fn open_default() -> Option<&'static Self> {
        load_global()
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, SemanticIndexError> {
        let path = path.as_ref();
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let embedding_model = conn
            .query_row(
                "SELECT value FROM index_meta WHERE key = 'embedding_model'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?;

        let mut stmt = conn.prepare(
            "SELECT chunk_id, file_path, title, content, embedding, embedding_dim FROM chunks",
        )?;
        let rows = stmt.query_map([], |row| {
            let chunk_id: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            let title: String = row.get(2)?;
            let content: String = row.get(3)?;
            let blob: Vec<u8> = row.get(4)?;
            let dim: i32 = row.get(5)?;
            Ok((chunk_id, file_path, title, content, blob, dim))
        })?;

        let mut documents = Vec::new();
        for row in rows {
            let (chunk_id, file_path, title, content, blob, dim) = row?;
            let embedding = blob_to_embedding(&blob, dim as usize)?;
            documents.push(VectorDocument {
                doc_id: chunk_id,
                title,
                content,
                embedding,
                file_path,
            });
        }

        Ok(Self {
            inner: InMemoryChunkIndex {
                documents,
                embedding_model,
                db_path: path.to_path_buf(),
            },
        })
    }

    #[must_use]
    pub fn chunk_count(&self) -> usize {
        self.inner.documents.len()
    }

    #[must_use]
    pub fn embedding_model(&self) -> Option<&str> {
        self.inner.embedding_model.as_deref()
    }

    #[must_use]
    pub fn db_path(&self) -> &Path {
        &self.inner.db_path
    }

    /// 纯向量检索
    pub fn search_vector(&self, query_embedding: &[f32], top_k: usize) -> Vec<ChunkHit> {
        if query_embedding.is_empty() || self.inner.documents.is_empty() {
            return vec![];
        }

        let mut hits: Vec<ChunkHit> = self
            .inner
            .documents
            .iter()
            .map(|doc| ChunkHit {
                chunk_id: doc.doc_id.clone(),
                file_path: doc.file_path.clone(),
                title: doc.title.clone(),
                content: doc.content.clone(),
                score: cosine_similarity(query_embedding, &doc.embedding),
            })
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
    }

    /// 混合检索（向量 + 关键词），与 yunpat-agent 默认 alpha=0.7 一致
    pub fn hybrid_search(
        &self,
        query_embedding: &[f32],
        query: &str,
        top_k: usize,
        alpha: f32,
    ) -> Vec<ChunkHit> {
        if self.inner.documents.is_empty() {
            return vec![];
        }

        let keywords = extract_keywords(query);
        let mut hits: Vec<ChunkHit> = self
            .inner
            .documents
            .iter()
            .map(|doc| {
                let semantic_score = if query_embedding.is_empty() {
                    0.0
                } else {
                    cosine_similarity(query_embedding, &doc.embedding)
                };
                let keyword_score = score_keywords(&doc.content, &keywords);
                let hybrid = alpha * semantic_score + (1.0 - alpha) * keyword_score;
                ChunkHit {
                    chunk_id: doc.doc_id.clone(),
                    file_path: doc.file_path.clone(),
                    title: doc.title.clone(),
                    content: doc.content.clone(),
                    score: hybrid,
                }
            })
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
    }
}

fn blob_to_embedding(blob: &[u8], dim: usize) -> Result<Vec<f32>, SemanticIndexError> {
    if dim == 0 || blob.len() != dim * 4 {
        return Err(SemanticIndexError::InvalidBlob {
            dim,
            bytes: blob.len(),
        });
    }
    Ok(blob
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn is_separator(c: char) -> bool {
    c.is_whitespace() || "，。、；：？！\"\"''（）【】".contains(c)
}

fn extract_keywords(query: &str) -> Vec<String> {
    query
        .split(is_separator)
        .filter(|s| s.chars().count() >= 2)
        .map(str::to_string)
        .collect()
}

fn score_keywords(content: &str, keywords: &[String]) -> f32 {
    if keywords.is_empty() {
        return 0.0;
    }
    let mut matched = 0usize;
    for kw in keywords {
        if content.contains(kw.as_str()) {
            matched += 1;
        }
    }
    matched as f32 / keywords.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_repo_semantic_index_when_present() {
        let path = crate::paths::KnowledgePaths::discover().semantic_index_db;
        let Some(p) = path else {
            return;
        };
        let index = PrebuiltSemanticIndex::open(&p).expect("open index");
        assert!(index.chunk_count() > 1000, "expected large prebuilt index");
    }
}
