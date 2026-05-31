//! 从专利项目目录导入材料到案件文档。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use super::case::{case_load, case_save, CaseDocument, PatentCase};

const MATERIAL_EXTENSIONS: &[&str] = &["md", "txt", "pdf", "doc", "docx", "pptx", "ppt", "rtf"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialFileEntry {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMaterialsResult {
    pub imported: Vec<String>,
    pub skipped: Vec<String>,
    pub errors: Vec<String>,
    pub case: PatentCase,
}

/// 列举项目目录下的可导入材料（可递归子目录）
///
/// `max_depth`：0 = 仅当前目录，默认 2，上限 5。
#[tauri::command]
pub fn list_project_materials(
    project_folder: String,
    max_depth: Option<u8>,
) -> Result<Vec<MaterialFileEntry>, String> {
    let dir = PathBuf::from(project_folder.trim());
    if !dir.is_dir() {
        return Err(format!("目录不存在: {}", dir.display()));
    }
    let max = max_depth.unwrap_or(2).min(5);
    let mut files = Vec::new();
    collect_material_files(&dir, &dir, 0, max, &mut files)?;
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

fn collect_material_files(
    root: &Path,
    dir: &Path,
    depth: u8,
    max_depth: u8,
    out: &mut Vec<MaterialFileEntry>,
) -> Result<(), String> {
    if depth > max_depth {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            if depth >= max_depth {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            collect_material_files(root, &path, depth + 1, max_depth, out)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if !MATERIAL_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let size_bytes = entry.metadata().map_err(|e| e.to_string())?.len();
        out.push(MaterialFileEntry {
            path: path.to_string_lossy().into_owned(),
            name: rel,
            extension: ext,
            size_bytes,
        });
    }
    Ok(())
}

/// 批量导入项目目录材料到案件（覆盖同类型文档正文）
#[tauri::command]
pub fn import_project_materials(
    case_id: String,
    project_folder: String,
    max_files: Option<usize>,
    max_depth: Option<u8>,
) -> Result<ImportMaterialsResult, String> {
    let limit = max_files.unwrap_or(20).min(50);
    let files = list_project_materials(project_folder, max_depth)?;
    let mut case = case_load(case_id)?;
    let now = chrono_lite_now();
    let mut imported = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();

    for file in files.into_iter().take(limit) {
        let path = PathBuf::from(&file.path);
        let doc_type = guess_doc_type(&file.name);
        match extract_markdown(&path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    skipped.push(format!("{}（空内容）", file.name));
                    continue;
                }
                upsert_document(&mut case, &now, &doc_type, &file.name, &content);
                imported.push(format!("{} → {}", file.name, doc_type));
            }
            Err(e) => errors.push(format!("{}: {e}", file.name)),
        }
    }

    case.updated_at = now;
    let saved = case_save(case)?;
    Ok(ImportMaterialsResult {
        imported,
        skipped,
        errors,
        case: saved,
    })
}

fn upsert_document(case: &mut PatentCase, now: &str, doc_type: &str, title: &str, content: &str) {
    if let Some(doc) = case.documents.iter_mut().find(|d| d.doc_type == doc_type) {
        doc.title = title.to_string();
        doc.content_md = content.to_string();
        doc.updated_at = now.to_string();
        return;
    }
    let id = format!("doc-{doc_type}-{}", now);
    case.documents.push(CaseDocument {
        id,
        doc_type: doc_type.to_string(),
        title: title.to_string(),
        content_md: content.to_string(),
        updated_at: now.to_string(),
    });
}

fn guess_doc_type(filename: &str) -> &'static str {
    let lower = filename.to_ascii_lowercase();
    if lower.contains("权利要求") || lower.contains("claim") {
        return "claims";
    }
    if lower.contains("审查") || lower.contains("oa") || lower.contains("意见") {
        return "review";
    }
    if lower.contains("检索") || lower.contains("search") {
        return "search";
    }
    if lower.contains("草稿") || lower.contains("draft") {
        return "drafts";
    }
    if lower.contains("说明") || lower.contains("description") || lower.contains("spec") {
        return "description";
    }
    "description"
}

fn extract_markdown(path: &Path) -> Result<String, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "md" | "txt" | "rtf" => fs::read_to_string(path).map_err(|e| e.to_string()),
        "pdf" | "doc" | "docx" | "ppt" | "pptx" => convert_with_markitdown(path),
        _ => Err(format!("不支持的扩展名: {ext}")),
    }
}

fn convert_with_markitdown(path: &Path) -> Result<String, String> {
    let script = find_markitdown_script().ok_or_else(|| {
        "未找到 scripts/patent/markitdown_convert.py，请设置 YUNXI_WORKSPACE 指向仓库根目录，或仅导入 .md/.txt".to_string()
    })?;
    let output = Command::new("python3")
        .arg(&script)
        .arg(path)
        .output()
        .map_err(|e| format!("调用 python3 失败: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("markitdown 失败: {err}"));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value =
        serde_json::from_str(text.trim()).map_err(|e| format!("解析 markitdown 输出失败: {e}"))?;
    if let Some(md) = value.get("markdown").and_then(|v| v.as_str()) {
        return Ok(md.to_string());
    }
    if let Some(err) = value.get("error").and_then(|v| v.as_str()) {
        return Err(err.to_string());
    }
    Err("markitdown 无 markdown 字段".to_string())
}

fn find_markitdown_script() -> Option<PathBuf> {
    if let Ok(ws) = std::env::var("YUNXI_WORKSPACE") {
        let p = PathBuf::from(ws).join("scripts/patent/markitdown_convert.py");
        if p.is_file() {
            return Some(p);
        }
    }
    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts/patent/markitdown_convert.py");
    if dev.is_file() {
        return Some(dev);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn collect_materials_recursive_finds_nested_file() {
        let tmp = std::env::temp_dir().join(format!("yunxi-mat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("docs")).unwrap();
        fs::write(tmp.join("docs/claims.md"), "# claims").unwrap();
        let mut out = Vec::new();
        collect_material_files(&tmp, &tmp, 0, 2, &mut out).unwrap();
        assert_eq!(out.len(), 1);
        assert!(out[0].name.contains("claims.md"));
        let _ = fs::remove_dir_all(&tmp);
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}
