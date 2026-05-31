//! 记忆存储
//!
//! 管理记忆文件的读写、索引和检索。

use crate::frontmatter::parse_memory_file;
use crate::relevance::score_relevance;
use crate::types::{MemoryEntry, MemoryType};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

/// 记忆存储
pub struct MemoryStore {
    base_dir: PathBuf,
    index_path: PathBuf,
}

impl MemoryStore {
    /// 创建记忆存储，指定根目录
    pub fn new(base_dir: &str) -> Self {
        let base = PathBuf::from(base_dir);
        let index = base.join("MEMORY.md");
        Self {
            base_dir: base,
            index_path: index,
        }
    }

    /// 使用默认路径 ~/.yunxi/memory/
    pub fn default_path() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Self::new(&format!("{home}/.yunxi/memory"))
    }

    /// 确保目录结构存在
    pub fn ensure_dirs(&self) -> Result<(), String> {
        for mt in &[
            MemoryType::User,
            MemoryType::Feedback,
            MemoryType::Project,
            MemoryType::Reference,
        ] {
            let dir = self.type_dir(mt);
            std::fs::create_dir_all(&dir)
                .map_err(|e| format!("创建目录失败 {}: {e}", dir.display()))?;
        }
        Ok(())
    }

    /// 获取某类型目录
    fn type_dir(&self, mt: &MemoryType) -> PathBuf {
        let sub = match mt {
            MemoryType::User => "user",
            MemoryType::Feedback => "feedback",
            MemoryType::Project => "project",
            MemoryType::Reference => "reference",
        };
        self.base_dir.join(sub)
    }

    /// 列出所有记忆条目
    pub fn list_all(&self) -> Vec<MemoryEntry> {
        let mut entries = Vec::new();
        if !self.base_dir.exists() {
            return entries;
        }

        for entry in WalkDir::new(&self.base_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.into_path();
            if path.extension().is_some_and(|ext| ext == "md")
                && path.file_name().is_some_and(|n| n != "MEMORY.md")
            {
                if let Some(entry) = parse_memory_file(&path) {
                    entries.push(entry);
                }
            }
        }
        entries
    }

    /// 按类型列出记忆
    pub fn list_by_type(&self, mt: MemoryType) -> Vec<MemoryEntry> {
        let dir = self.type_dir(&mt);
        if !dir.exists() {
            return Vec::new();
        }
        let mut entries = Vec::new();
        for entry in WalkDir::new(&dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.into_path();
            if path.extension().is_some_and(|ext| ext == "md") {
                if let Some(entry) = parse_memory_file(&path) {
                    entries.push(entry);
                }
            }
        }
        entries
    }

    /// 按关键词检索记忆（按相关性排序）
    pub fn recall(&self, query: &str, limit: usize) -> Vec<MemoryEntry> {
        let all = self.list_all();
        let mut scored: Vec<(f64, MemoryEntry)> = all
            .into_iter()
            .map(|entry| {
                let score = score_relevance(query, &entry);
                (score, entry)
            })
            .filter(|(score, _)| *score > 0.0)
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);
        scored.into_iter().map(|(_, e)| e).collect()
    }

    /// 写入记忆条目
    pub fn store(
        &self,
        mt: MemoryType,
        name: &str,
        content: &str,
        tags: Vec<String>,
    ) -> Result<PathBuf, String> {
        self.ensure_dirs()?;
        let dir = self.type_dir(&mt);
        let filename = if name.ends_with(".md") {
            name.to_string()
        } else {
            format!("{name}.md")
        };
        let path = dir.join(&filename);

        let type_str = match mt {
            MemoryType::User => "user",
            MemoryType::Feedback => "feedback",
            MemoryType::Project => "project",
            MemoryType::Reference => "reference",
        };

        let tags_yaml = if tags.is_empty() {
            "[]".into()
        } else {
            format!(
                "[{}]",
                tags.iter()
                    .map(|t| format!("\"{t}\""))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let now = chrono_now();
        let file_content = format!(
            "---\nname: {name}\ndescription: {content}\nmetadata:\n  type: {type_str}\n  tags: {tags_yaml}\n  created_at: \"{now}\"\n  updated_at: \"{now}\"\n---\n\n{content}\n"
        );

        std::fs::write(&path, file_content).map_err(|e| format!("写入记忆文件失败: {e}"))?;

        let _ = self.rebuild_index();

        Ok(path)
    }

    /// 删除指定记忆文件
    pub fn delete(&self, mt: MemoryType, name: &str) -> Result<(), String> {
        let dir = self.type_dir(&mt);
        let filename = if name.ends_with(".md") {
            name.to_string()
        } else {
            format!("{name}.md")
        };
        let path = dir.join(&filename);
        if !path.exists() {
            return Err(format!("记忆文件不存在: {}", path.display()));
        }
        std::fs::remove_file(&path).map_err(|e| format!("删除记忆文件失败: {e}"))?;
        let _ = self.rebuild_index();
        Ok(())
    }

    /// 生成 MEMORY.md 索引
    pub fn rebuild_index(&self) -> Result<(), String> {
        let entries = self.list_all();
        let mut lines = vec!["# Memory Index\n".into()];

        // 按类型分组
        let mut groups: HashMap<String, Vec<&MemoryEntry>> = HashMap::new();
        for entry in &entries {
            let type_str = format!("{:?}", entry.meta.memory_type);
            groups.entry(type_str).or_default().push(entry);
        }

        for (type_name, group) in groups {
            lines.push(format!("\n## {type_name}\n"));
            for entry in group {
                let rel = entry
                    .path
                    .strip_prefix(&self.base_dir)
                    .unwrap_or(&entry.path);
                let desc = entry
                    .content
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(120)
                    .collect::<String>();
                lines.push(format!(
                    "- [{name}]({path}) — {desc}",
                    name = rel.display(),
                    path = rel.display(),
                    desc = desc
                ));
            }
        }

        let content = lines.join("\n");
        std::fs::write(&self.index_path, content).map_err(|e| format!("写入索引失败: {e}"))?;
        Ok(())
    }
}

fn chrono_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_store_and_recall() {
        let tmp = tempfile_path();
        let _ = fs::remove_dir_all(&tmp);
        let store = MemoryStore::new(&tmp);

        store
            .store(
                MemoryType::User,
                "test-user",
                "用户偏好测试内容",
                vec!["测试".into()],
            )
            .unwrap();
        store
            .store(
                MemoryType::Feedback,
                "test-fb",
                "不要做X",
                vec!["反馈".into()],
            )
            .unwrap();

        let results = store.recall("偏好", 5);
        assert!(!results.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_list_by_type() {
        let tmp = format!("/tmp/yunxi-test-memory-type-{}", std::process::id());
        let _ = fs::remove_dir_all(&tmp);
        let store = MemoryStore::new(&tmp);

        store
            .store(MemoryType::User, "u1", "用户信息", vec![])
            .unwrap();
        store
            .store(MemoryType::Project, "p1", "项目信息", vec![])
            .unwrap();

        let user_entries = store.list_by_type(MemoryType::User);
        assert_eq!(user_entries.len(), 1);

        let _ = fs::remove_dir_all(&tmp);
    }

    fn tempfile_path() -> String {
        format!("/tmp/yunxi-test-memory-store-{}", std::process::id())
    }
}
