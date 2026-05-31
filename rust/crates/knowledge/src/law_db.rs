//! 法律法规数据库
//!
//! 基于 SQLite 的法律法规查询引擎，支持 FTS5 全文搜索。

use crate::types::{LawCategory, LawDocument};
use rusqlite::{params, Connection, OpenFlags};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LawDbError {
    #[error("数据库打开失败: {0}")]
    OpenFailed(String),
    #[error("查询失败: {0}")]
    QueryFailed(String),
}

/// 法律法规数据库
pub struct LawDatabase {
    conn: Connection,
}

impl LawDatabase {
    /// 打开法律法规数据库（只读模式）
    pub fn open(path: &str) -> Result<Self, LawDbError> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| LawDbError::OpenFailed(format!("{path}: {e}")))?;
        Ok(Self { conn })
    }

    /// 按名称模糊搜索法律法规
    pub fn search_by_name(
        &self,
        keyword: &str,
        limit: usize,
    ) -> Result<Vec<LawDocument>, LawDbError> {
        let pattern = format!("%{keyword}%");
        let sql =
            "SELECT id, level, name, subtitle, filename, publish, expired, category_id, content \
                   FROM law WHERE name LIKE ?1 LIMIT ?2";
        self.query_laws(sql, params![pattern, limit])
    }

    /// 按内容全文搜索（使用 LIKE，laws.db 无 FTS5）
    pub fn search_by_content(
        &self,
        keyword: &str,
        limit: usize,
    ) -> Result<Vec<LawDocument>, LawDbError> {
        let pattern = format!("%{keyword}%");
        let sql =
            "SELECT id, level, name, subtitle, filename, publish, expired, category_id, content \
                   FROM law WHERE name LIKE ?1 OR content LIKE ?1 LIMIT ?2";
        self.query_laws(sql, params![pattern, limit])
    }

    /// 按法律层级查询
    pub fn list_by_level(&self, level: &str, limit: usize) -> Result<Vec<LawDocument>, LawDbError> {
        let sql =
            "SELECT id, level, name, subtitle, filename, publish, expired, category_id, content \
                   FROM law WHERE level = ?1 ORDER BY publish DESC LIMIT ?2";
        self.query_laws(sql, params![level, limit])
    }

    /// 获取全部法律层级类型
    pub fn list_levels(&self) -> Result<Vec<String>, LawDbError> {
        let sql = "SELECT DISTINCT level FROM law ORDER BY level";
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| LawDbError::QueryFailed(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| LawDbError::QueryFailed(e.to_string()))?;
        let levels: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(levels)
    }

    /// 获取法律法规类别列表
    pub fn list_categories(&self) -> Result<Vec<LawCategory>, LawDbError> {
        let sql = "SELECT id, name, folder, isSubFolder, \"group\", \"order\" FROM category ORDER BY \"order\"";
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| LawDbError::QueryFailed(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(LawCategory {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    folder: row.get(2)?,
                    is_sub_folder: row.get::<_, i32>(3)? != 0,
                    group: row.get(4)?,
                    order: row.get(5)?,
                })
            })
            .map_err(|e| LawDbError::QueryFailed(e.to_string()))?;
        let cats: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(cats)
    }

    /// 获取法律法规总数
    pub fn count(&self) -> Result<usize, LawDbError> {
        self.conn
            .query_row("SELECT COUNT(*) FROM law", [], |row| row.get(0))
            .map_err(|e| LawDbError::QueryFailed(e.to_string()))
    }

    /// 列出所有法律法规（分页）
    pub fn list_all(&self, limit: usize, offset: usize) -> Result<Vec<LawDocument>, LawDbError> {
        self.query_laws(
            "SELECT id, level, name, subtitle, filename, publish, expired, category_id, content FROM law ORDER BY id LIMIT ?1 OFFSET ?2",
            rusqlite::params![limit, offset],
        )
    }

    fn query_laws<P: rusqlite::Params>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Vec<LawDocument>, LawDbError> {
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| LawDbError::QueryFailed(e.to_string()))?;
        let rows = stmt
            .query_map(params, |row| {
                Ok(LawDocument {
                    id: row.get(0)?,
                    level: row.get(1)?,
                    name: row.get(2)?,
                    subtitle: row.get(3)?,
                    filename: row.get(4)?,
                    publish: row.get(5)?,
                    expired: row.get::<_, i32>(6)? != 0,
                    category_id: row.get(7)?,
                    content: row.get(8)?,
                })
            })
            .map_err(|e| LawDbError::QueryFailed(e.to_string()))?;
        let laws: Vec<_> = rows.filter_map(|r| r.ok()).collect();
        Ok(laws)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_open_law_db() {
        let path = "../../../assets/knowledge/data/laws.db";
        if !Path::new(path).exists() {
            eprintln!("Skipping test: laws.db not found");
            return;
        }
        let db = LawDatabase::open(path).unwrap();
        let count = db.count().unwrap();
        assert!(count > 0, "laws.db should contain laws");
    }

    #[test]
    fn test_search_by_name() {
        let path = "../../../assets/knowledge/data/laws.db";
        if !Path::new(path).exists() {
            eprintln!("Skipping test: laws.db not found");
            return;
        }
        let db = LawDatabase::open(path).unwrap();
        let results = db.search_by_name("专利", 10).unwrap();
        assert!(!results.is_empty(), "should find patent-related laws");
    }

    #[test]
    fn test_list_levels() {
        let path = "../../../assets/knowledge/data/laws.db";
        if !Path::new(path).exists() {
            eprintln!("Skipping test: laws.db not found");
            return;
        }
        let db = LawDatabase::open(path).unwrap();
        let levels = db.list_levels().unwrap();
        assert!(!levels.is_empty());
    }
}
