//! 案件磁盘存储（与桌面端 `~/.yunxi/cases` 对齐）。

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const SEED_CLAIMS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../yunxi-cli/assets/desktop-seed-claims.md"
));
const SEED_DRAFTS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../yunxi-cli/assets/desktop-seed-drafts.md"
));
const SEED_REVIEW: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../yunxi-cli/assets/desktop-seed-review.json"
));
const SEED_DESCRIPTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../yunxi-cli/assets/desktop-seed-description.md"
));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseDocument {
    pub id: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    pub title: String,
    pub content_md: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentCase {
    pub id: String,
    pub name: String,
    pub application_number: String,
    pub status: String,
    pub documents: Vec<CaseDocument>,
    pub active_session_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct CaseStore;

impl CaseStore {
    pub fn cases_dir() -> PathBuf {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".yunxi").join("cases"))
            .unwrap_or_else(|_| {
                crate::agent_bridge::workspace_root()
                    .join(".yunxi")
                    .join("cases")
            })
    }

    fn case_path(id: &str) -> Result<PathBuf, String> {
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("invalid case id".into());
        }
        Ok(Self::cases_dir().join(format!("{id}.json")))
    }

    fn now_secs() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    }

    pub fn seed_if_empty() -> Result<(), String> {
        let dir = Self::cases_dir();
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let marker = dir.join(".demo-seeded");
        if marker.exists() {
            return Ok(());
        }
        let has_any = fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .any(|e| e.path().extension().is_some_and(|ext| ext == "json"));
        if has_any {
            let _ = fs::write(&marker, "1");
            return Ok(());
        }

        let now = Self::now_secs();
        let default = PatentCase {
            id: "case-1".to_string(),
            name: "智能电池管理系统".to_string(),
            application_number: "CN202410123456.X".to_string(),
            status: "examination".to_string(),
            documents: vec![
                CaseDocument {
                    id: "doc-claims-1".to_string(),
                    doc_type: "claims".to_string(),
                    title: "权利要求书（原始）".to_string(),
                    content_md: SEED_CLAIMS.to_string(),
                    updated_at: now.clone(),
                },
                CaseDocument {
                    id: "doc-drafts-1".to_string(),
                    doc_type: "drafts".to_string(),
                    title: "权利要求书（修改稿）".to_string(),
                    content_md: SEED_DRAFTS.to_string(),
                    updated_at: now.clone(),
                },
                CaseDocument {
                    id: "doc-review-1".to_string(),
                    doc_type: "review".to_string(),
                    title: "审查意见".to_string(),
                    content_md: SEED_REVIEW.to_string(),
                    updated_at: now.clone(),
                },
                CaseDocument {
                    id: "doc-desc-1".to_string(),
                    doc_type: "description".to_string(),
                    title: "说明书".to_string(),
                    content_md: SEED_DESCRIPTION.to_string(),
                    updated_at: now.clone(),
                },
            ],
            active_session_id: None,
            created_at: now.clone(),
            updated_at: now,
        };
        Self::save(&default)?;
        let _ = fs::write(marker, "1");
        Ok(())
    }

    pub fn list() -> Result<Vec<PatentCase>, String> {
        Self::seed_if_empty()?;
        let dir = Self::cases_dir();
        let mut cases = Vec::new();
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            if let Ok(case) = serde_json::from_str::<PatentCase>(&text) {
                cases.push(case);
            }
        }
        cases.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(cases)
    }

    pub fn load(id: &str) -> Result<PatentCase, String> {
        let path = Self::case_path(id)?;
        if !path.exists() {
            return Err(format!("案件不存在: {id}"));
        }
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&text).map_err(|e| e.to_string())
    }

    pub fn save(case: &PatentCase) -> Result<(), String> {
        let dir = Self::cases_dir();
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = Self::case_path(&case.id)?;
        let json = serde_json::to_string_pretty(case).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())
    }

    pub fn delete(id: &str) -> Result<(), String> {
        let path = Self::case_path(id)?;
        if path.exists() {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn create(name: String, application_number: Option<String>) -> Result<PatentCase, String> {
        let id = format!("case-{}", Self::now_secs());
        let now = Self::now_secs();
        let case = PatentCase {
            id,
            name,
            application_number: application_number.unwrap_or_default(),
            status: "draft".to_string(),
            documents: Vec::new(),
            active_session_id: None,
            created_at: now.clone(),
            updated_at: now,
        };
        Self::save(&case)?;
        Ok(case)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_path_rejects_traversal() {
        assert!(CaseStore::case_path("../evil").is_err());
    }
}
