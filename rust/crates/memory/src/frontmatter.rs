//! YAML frontmatter 解析

use crate::types::{MemoryEntry, MemoryMeta, MemoryType};
use std::path::Path;

/// 解析带 frontmatter 的记忆文件
pub fn parse_memory_file(path: &Path) -> Option<MemoryEntry> {
    let content = std::fs::read_to_string(path).ok()?;

    if !content.starts_with("---") {
        return Some(MemoryEntry {
            path: path.to_path_buf(),
            meta: MemoryMeta {
                memory_type: MemoryType::Project,
                tags: vec![],
                created_at: String::new(),
                updated_at: String::new(),
            },
            content,
        });
    }

    let rest = &content[3..];
    let end = rest.find("---")?;
    let frontmatter = &rest[..end];
    let body = rest[end + 3..].trim().to_string();

    let meta = parse_frontmatter(frontmatter);
    Some(MemoryEntry {
        path: path.to_path_buf(),
        meta,
        content: body,
    })
}

fn parse_frontmatter(yaml: &str) -> MemoryMeta {
    let parsed: serde_yaml::Value = match serde_yaml::from_str(yaml) {
        Ok(v) => v,
        Err(_) => {
            return MemoryMeta {
                memory_type: MemoryType::Project,
                tags: vec![],
                created_at: String::new(),
                updated_at: String::new(),
            }
        }
    };

    let empty = serde_yaml::Mapping::new();
    let top = parsed.as_mapping().unwrap_or(&empty);

    // 嵌套 metadata 节点（store() 写入的格式）
    let nested = parsed
        .get("metadata")
        .and_then(|v| v.as_mapping())
        .unwrap_or(&empty);

    let memory_type = extract_type(top, nested);
    let tags = extract_tags(top, nested);
    let created_at = extract_str(nested, "created_at")
        .or_else(|| extract_str(top, "created_at"))
        .unwrap_or_default();
    let updated_at = extract_str(nested, "updated_at")
        .or_else(|| extract_str(top, "updated_at"))
        .unwrap_or_default();

    MemoryMeta {
        memory_type,
        tags,
        created_at,
        updated_at,
    }
}

fn extract_type(top: &serde_yaml::Mapping, nested: &serde_yaml::Mapping) -> MemoryType {
    let type_str = extract_str(nested, "type")
        .or_else(|| extract_str(top, "type"))
        .unwrap_or_else(|| "project".into());
    match type_str.as_str() {
        "user" => MemoryType::User,
        "feedback" => MemoryType::Feedback,
        "reference" => MemoryType::Reference,
        _ => MemoryType::Project,
    }
}

fn extract_tags(top: &serde_yaml::Mapping, nested: &serde_yaml::Mapping) -> Vec<String> {
    let tags_key = serde_yaml::Value::String("tags".into());
    let tags_val = nested.get(&tags_key).or_else(|| top.get(&tags_key));

    tags_val
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_str(mapping: &serde_yaml::Mapping, key: &str) -> Option<String> {
    let key_val = serde_yaml::Value::String(key.into());
    mapping
        .get(&key_val)
        .and_then(|v| v.as_str())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_flat_frontmatter() {
        let yaml = "type: user\ntags:\n  - \"rust\"\n  - \"测试\"\ncreated_at: \"2025-01-01T00:00:00\"\nupdated_at: \"2025-01-02T00:00:00\"\n";
        let meta = parse_frontmatter(yaml);
        assert_eq!(meta.memory_type, MemoryType::User);
        assert_eq!(meta.tags, vec!["rust", "测试"]);
        assert_eq!(meta.created_at, "2025-01-01T00:00:00");
    }

    #[test]
    fn test_parse_nested_metadata() {
        let yaml = "name: test\ndescription: desc\nmetadata:\n  type: feedback\n  tags: [\"x\", \"y\"]\n  created_at: \"2025-06-01T12:00:00\"\n  updated_at: \"2025-06-01T12:00:00\"\n";
        let meta = parse_frontmatter(yaml);
        assert_eq!(meta.memory_type, MemoryType::Feedback);
        assert_eq!(meta.tags, vec!["x", "y"]);
        assert_eq!(meta.created_at, "2025-06-01T12:00:00");
    }

    #[test]
    fn test_parse_empty_frontmatter() {
        let meta = parse_frontmatter("");
        assert_eq!(meta.memory_type, MemoryType::Project);
        assert!(meta.tags.is_empty());
        assert!(meta.created_at.is_empty());
    }

    #[test]
    fn test_roundtrip_store_parse() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("roundtrip.md");
        let content = "---\nname: roundtrip\ndescription: roundtrip content\nmetadata:\n  type: project\n  tags: [\"tag1\", \"tag2\"]\n  created_at: \"2025-06-15T10:30:00\"\n  updated_at: \"2025-06-15T10:30:00\"\n---\n\nroundtrip content\n";
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();

        let entry = parse_memory_file(&path).unwrap();
        assert_eq!(entry.meta.memory_type, MemoryType::Project);
        assert_eq!(entry.meta.tags, vec!["tag1", "tag2"]);
        assert_eq!(entry.content, "roundtrip content");
    }
}
