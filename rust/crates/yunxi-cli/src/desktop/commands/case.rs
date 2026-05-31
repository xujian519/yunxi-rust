//! 案件本地存储 IPC。

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::paths::{cases_dir, yunxi_home_dir};

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

fn case_path(id: &str) -> Result<PathBuf, String> {
    Ok(cases_dir()?.join(format!("{id}.json")))
}

fn seed_if_empty() -> Result<(), String> {
    let dir = cases_dir()?;
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

    let now = chrono_lite_now();
    let claims = include_str!("../../../assets/desktop-seed-claims.md");
    let review = include_str!("../../../assets/desktop-seed-review.json");
    let drafts = include_str!("../../../assets/desktop-seed-drafts.md");
    let description = include_str!("../../../assets/desktop-seed-description.md");
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
                content_md: claims.to_string(),
                updated_at: now.clone(),
            },
            CaseDocument {
                id: "doc-drafts-1".to_string(),
                doc_type: "drafts".to_string(),
                title: "权利要求书（修改稿）".to_string(),
                content_md: drafts.to_string(),
                updated_at: now.clone(),
            },
            CaseDocument {
                id: "doc-review-1".to_string(),
                doc_type: "review".to_string(),
                title: "审查意见".to_string(),
                content_md: review.to_string(),
                updated_at: now.clone(),
            },
            CaseDocument {
                id: "doc-desc-1".to_string(),
                doc_type: "description".to_string(),
                title: "说明书".to_string(),
                content_md: description.to_string(),
                updated_at: now.clone(),
            },
        ],
        active_session_id: None,
        created_at: now.clone(),
        updated_at: now,
    };
    save_case(&default)?;
    let _ = fs::write(&marker, "1");
    Ok(())
}

/// 为既有 case-1 补全桌面演示文档（不覆盖已有正文）。
fn enrich_demo_case(case: &mut PatentCase) -> Result<(), String> {
    if case.id != "case-1" {
        return Ok(());
    }
    let now = chrono_lite_now();
    let mut changed = false;

    changed |= push_doc_if_missing(
        case,
        &now,
        "drafts",
        "doc-drafts-1",
        "权利要求书（修改稿）",
        include_str!("../../../assets/desktop-seed-drafts.md"),
    );
    changed |= push_doc_if_missing(
        case,
        &now,
        "review",
        "doc-review-1",
        "审查意见",
        include_str!("../../../assets/desktop-seed-review.json"),
    );
    changed |= push_doc_if_missing(
        case,
        &now,
        "description",
        "doc-desc-1",
        "说明书",
        include_str!("../../../assets/desktop-seed-description.md"),
    );

    if changed {
        case.updated_at = now;
        save_case(case)?;
    }
    Ok(())
}

fn push_doc_if_missing(
    case: &mut PatentCase,
    now: &str,
    doc_type: &str,
    id: &str,
    title: &str,
    content: &str,
) -> bool {
    if case.documents.iter().any(|d| d.doc_type == doc_type) {
        return false;
    }
    case.documents.push(CaseDocument {
        id: id.to_string(),
        doc_type: doc_type.to_string(),
        title: title.to_string(),
        content_md: content.to_string(),
        updated_at: now.to_string(),
    });
    true
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn save_case(case: &PatentCase) -> Result<(), String> {
    let path = case_path(&case.id)?;
    let json = serde_json::to_string_pretty(case).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn case_list() -> Result<Vec<PatentCase>, String> {
    seed_if_empty()?;
    let dir = cases_dir()?;
    let mut cases = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        if let Ok(mut case) = serde_json::from_str::<PatentCase>(&text) {
            let _ = enrich_demo_case(&mut case);
            cases.push(case);
        }
    }
    cases.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(cases)
}

#[tauri::command]
pub fn case_load(id: String) -> Result<PatentCase, String> {
    let path = case_path(&id)?;
    if !path.exists() {
        return Err(format!("案件不存在: {id}"));
    }
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut case: PatentCase = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    enrich_demo_case(&mut case)?;
    Ok(case)
}

#[tauri::command]
pub fn case_save(case: PatentCase) -> Result<PatentCase, String> {
    save_case(&case)?;
    Ok(case)
}

#[tauri::command]
pub fn case_delete(id: String) -> Result<(), String> {
    let path = case_path(&id)?;
    if !path.exists() {
        return Err(format!("案件不存在: {id}"));
    }
    fs::remove_file(path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn case_create(name: String, application_number: Option<String>) -> Result<PatentCase, String> {
    let id = format!(
        "case-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let now = chrono_lite_now();
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
    save_case(&case)?;
    Ok(case)
}

#[tauri::command]
pub fn get_workspace_info() -> Result<serde_json::Value, String> {
    let workspace = yunxi_cli::session_mgr::workspace_root().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "workspaceRoot": workspace.to_string_lossy(),
        "yunxiHome": yunxi_home_dir()?.to_string_lossy(),
    }))
}
