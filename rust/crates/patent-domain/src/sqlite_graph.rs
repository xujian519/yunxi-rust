//! SQLite 知识图谱引擎
//!
//! 基于 `patent_kg.db` 提供专利知识图谱的节点/边查询和 FTS5 全文搜索。

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("数据库打开失败: {0}")]
    OpenFailed(String),
    #[error("查询失败: {0}")]
    QueryFailed(String),
}

/// 知识图谱节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub title: String,
    pub content: Option<String>,
    pub law_refs_count: Option<i64>,
    pub source: Option<String>,
    pub full_ref: Option<String>,
    pub chapter: Option<String>,
    pub article_number: Option<String>,
}

/// 知识图谱边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEdge {
    pub id: i64,
    pub source: String,
    pub target: String,
    pub relation: String,
}

/// 图谱统计信息
#[derive(Debug, Clone, Serialize)]
pub struct KgStats {
    pub node_count: usize,
    pub edge_count: usize,
}

/// 节点类型分布
#[derive(Debug, Clone, Serialize)]
pub struct NodeTypeCount {
    pub node_type: String,
    pub count: usize,
}

/// SQLite 知识图谱读取器
pub struct SqliteKnowledgeGraph {
    conn: Connection,
}

impl SqliteKnowledgeGraph {
    /// 打开指定路径的 SQLite 数据库
    pub fn open(path: &str) -> Result<Self, GraphError> {
        let conn =
            Connection::open(path).map_err(|e| GraphError::OpenFailed(format!("{path}: {e}")))?;

        // 启用 FTS5（如已编译则无需额外操作）
        Ok(Self { conn })
    }

    /// 以只读模式打开
    pub fn open_readonly(path: &str) -> Result<Self, GraphError> {
        let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| GraphError::OpenFailed(format!("{path}: {e}")))?;

        Ok(Self { conn })
    }

    /// 从已有 SQLite 连接构造（用于 in-memory 场景，如测试）
    pub fn from_connection(conn: Connection) -> Self {
        Self { conn }
    }

    /// 获取图谱统计信息
    pub fn stats(&self) -> Result<KgStats, GraphError> {
        let node_count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let edge_count: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        Ok(KgStats {
            node_count,
            edge_count,
        })
    }

    /// 按关键词搜索节点（使用 FTS5 全文索引）
    pub fn search_nodes(
        &self,
        query: &str,
        node_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<KgNode>, GraphError> {
        let fts_query = format!("\"{}\"", query.replace('"', "\"\""));

        if let Some(nt) = node_type {
            self.search_nodes_filtered(&fts_query, nt, limit)
        } else {
            self.search_nodes_all(&fts_query, limit)
        }
    }

    fn search_nodes_all(&self, fts_query: &str, limit: usize) -> Result<Vec<KgNode>, GraphError> {
        let sql = "SELECT n.id, n.node_type, n.name, n.title, n.content, n.law_refs_count, n.source, n.full_ref, n.chapter, n.article_number \
             FROM nodes_fts f \
             JOIN nodes n ON n.rowid = f.rowid \
             WHERE nodes_fts MATCH ?1 \
             ORDER BY f.rank \
             LIMIT ?2";
        self.query_nodes(sql, params![fts_query, limit])
    }

    fn search_nodes_filtered(
        &self,
        fts_query: &str,
        node_type: &str,
        limit: usize,
    ) -> Result<Vec<KgNode>, GraphError> {
        let sql = "SELECT n.id, n.node_type, n.name, n.title, n.content, n.law_refs_count, n.source, n.full_ref, n.chapter, n.article_number \
             FROM nodes_fts f \
             JOIN nodes n ON n.rowid = f.rowid \
             WHERE nodes_fts MATCH ?1 AND n.node_type = ?2 \
             ORDER BY f.rank \
             LIMIT ?3";
        self.query_nodes(sql, params![fts_query, node_type, limit])
    }

    fn query_nodes<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<KgNode>, GraphError> {
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params, |row| {
                Ok(KgNode {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    law_refs_count: row.get(5)?,
                    source: row.get(6)?,
                    full_ref: row.get(7)?,
                    chapter: row.get(8)?,
                    article_number: row.get(9)?,
                })
            })
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let mut nodes = Vec::new();
        for row in rows {
            match row {
                Ok(node) => nodes.push(node),
                Err(e) => eprintln!("Warning: skipping node row: {e}"),
            }
        }
        Ok(nodes)
    }

    /// 获取与指定节点关联的所有边
    pub fn get_edges(&self, node_id: &str) -> Result<Vec<KgEdge>, GraphError> {
        let sql = "SELECT id, source, target, relation FROM edges WHERE source = ?1 OR target = ?1";

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params![node_id], |row| {
                Ok(KgEdge {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    relation: row.get(3)?,
                })
            })
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let edges: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(edges)
    }

    /// 按节点类型获取节点
    pub fn get_nodes_by_type(
        &self,
        node_type: &str,
        limit: usize,
    ) -> Result<Vec<KgNode>, GraphError> {
        let sql = "SELECT id, node_type, name, title, content, law_refs_count, source, full_ref, chapter, article_number \
                   FROM nodes WHERE node_type = ?1 LIMIT ?2";

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map(params![node_type, limit], |row| {
                Ok(KgNode {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    name: row.get(2)?,
                    title: row.get(3)?,
                    content: row.get(4)?,
                    law_refs_count: row.get(5)?,
                    source: row.get(6)?,
                    full_ref: row.get(7)?,
                    chapter: row.get(8)?,
                    article_number: row.get(9)?,
                })
            })
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let nodes: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(nodes)
    }

    /// 节点类型分布统计
    pub fn node_type_distribution(&self) -> Result<Vec<NodeTypeCount>, GraphError> {
        let sql =
            "SELECT node_type, COUNT(*) as cnt FROM nodes GROUP BY node_type ORDER BY cnt DESC";

        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(NodeTypeCount {
                    node_type: row.get(0)?,
                    count: row.get(1)?,
                })
            })
            .map_err(|e| GraphError::QueryFailed(e.to_string()))?;

        let result: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_open_and_stats() {
        let db_path = "../../../assets/knowledge_graph/patent_kg.db";
        if !Path::new(db_path).exists() {
            eprintln!("Skipping test: patent_kg.db not found");
            return;
        }
        let kg = SqliteKnowledgeGraph::open_readonly(db_path).unwrap();
        let stats = kg.stats().unwrap();
        assert!(stats.node_count > 0);
        assert!(stats.edge_count > 0);
    }

    #[test]
    fn test_search_nodes() {
        let db_path = "../../../assets/knowledge_graph/patent_kg.db";
        if !Path::new(db_path).exists() {
            eprintln!("Skipping test: patent_kg.db not found");
            return;
        }
        let kg = SqliteKnowledgeGraph::open_readonly(db_path).unwrap();
        let nodes = kg.search_nodes("创造性", None, 5).unwrap();
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_node_type_distribution() {
        let db_path = "../../../assets/knowledge_graph/patent_kg.db";
        if !Path::new(db_path).exists() {
            eprintln!("Skipping test: patent_kg.db not found");
            return;
        }
        let kg = SqliteKnowledgeGraph::open_readonly(db_path).unwrap();
        let dist = kg.node_type_distribution().unwrap();
        assert!(!dist.is_empty());
    }
}
