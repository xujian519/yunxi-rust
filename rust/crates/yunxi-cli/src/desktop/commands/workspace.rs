//! 工作区文件夹选择、扫描（YUNXI.md / patentCase）。

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::Emitter;

/// 工作区内扫描到的项目目录（含 YUNXI.md 的文件夹）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceProjectEntry {
    pub folder_path: String,
    pub label: String,
    pub is_patent_project: bool,
    pub case_id: Option<String>,
    pub case_name: Option<String>,
    pub workspace_root: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanWorkspaceResult {
    pub projects: Vec<WorkspaceProjectEntry>,
}

/// 原生选择文件夹（macOS / Windows / Linux）
#[tauri::command]
pub fn pick_workspace_folder() -> Result<Option<String>, String> {
    let picked = rfd::FileDialog::new()
        .set_title("选择工作区文件夹")
        .pick_folder();
    Ok(picked.map(|p| p.to_string_lossy().into_owned()))
}

/// 扫描多个工作区根路径下的专利项目（含 YUNXI.md 的目录）
///
/// `max_depth`：从各工作区根向下递归的最大深度（默认 2，上限 5）。
#[tauri::command]
pub fn scan_workspace_roots(
    paths: Vec<String>,
    max_depth: Option<u8>,
) -> Result<ScanWorkspaceResult, String> {
    let max = max_depth.unwrap_or(2).clamp(1, 5);
    let mut projects = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for raw in paths {
        let root = PathBuf::from(raw.trim());
        if !root.is_dir() {
            continue;
        }
        collect_projects(&root, &root, 0, max, &mut projects, &mut seen);
    }

    projects.sort_by(|a, b| a.label.cmp(&b.label));
    Ok(ScanWorkspaceResult { projects })
}

/// 启动工作区目录监视（变更时 emit `yunxi://workspace/changed`）
#[tauri::command]
pub fn workspace_watch_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, std::sync::Arc<crate::state::DesktopState>>,
    paths: Vec<String>,
) -> Result<(), String> {
    use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::Mutex as StdMutex;
    use std::time::Instant;

    let mut guard = state.workspace_watcher.lock().map_err(|e| e.to_string())?;
    *guard = None;

    let last_emit = std::sync::Arc::new(StdMutex::new(
        Instant::now() - std::time::Duration::from_secs(60),
    ));
    let app_emit = app.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            let Ok(event) = res else {
                return;
            };
            let relevant = matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            );
            if !relevant {
                return;
            }
            if event.paths.iter().any(|p| should_ignore_watch_path(p)) {
                return;
            }
            let mut last = last_emit.lock().expect("debounce lock");
            let now = Instant::now();
            if now.duration_since(*last) < std::time::Duration::from_millis(2000) {
                return;
            }
            *last = now;
            let _ = app_emit.emit("yunxi://workspace/changed", ());
        },
        notify::Config::default(),
    )
    .map_err(|e| format!("监视器创建失败: {e}"))?;

    for raw in paths {
        let root = PathBuf::from(raw.trim());
        if root.is_dir() {
            let _ = watcher.watch(&root, RecursiveMode::Recursive);
        }
    }

    *guard = Some(watcher);
    Ok(())
}

#[tauri::command]
pub fn workspace_watch_stop(
    state: tauri::State<'_, std::sync::Arc<crate::state::DesktopState>>,
) -> Result<(), String> {
    let mut guard = state.workspace_watcher.lock().map_err(|e| e.to_string())?;
    *guard = None;
    Ok(())
}

fn collect_projects(
    workspace_root: &Path,
    dir: &Path,
    depth: u8,
    max_depth: u8,
    out: &mut Vec<WorkspaceProjectEntry>,
    seen: &mut std::collections::HashSet<String>,
) {
    if let Some(entry) = entry_from_dir(workspace_root, dir) {
        let key = entry.folder_path.clone();
        if seen.insert(key) {
            out.push(entry);
        }
    }

    if depth >= max_depth {
        return;
    }

    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };

    let mut children: Vec<PathBuf> = read_dir
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    children.sort();

    for child in children {
        let name = child.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }
        collect_projects(
            workspace_root,
            &child,
            depth.saturating_add(1),
            max_depth,
            out,
            seen,
        );
    }
}

fn entry_from_dir(workspace_root: &Path, dir: &Path) -> Option<WorkspaceProjectEntry> {
    let yunxi = dir.join("YUNXI.md");
    if !yunxi.is_file() {
        return None;
    }
    let body = fs::read_to_string(&yunxi).ok()?;
    let is_patent = body.contains("patentCase:");
    let (case_id, case_name) = if is_patent {
        parse_patent_case_meta(&body)
    } else {
        (None, None)
    };
    let label = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("项目")
        .to_string();

    Some(WorkspaceProjectEntry {
        folder_path: dir.to_string_lossy().into_owned(),
        label,
        is_patent_project: is_patent,
        case_id,
        case_name,
        workspace_root: workspace_root.to_string_lossy().into_owned(),
    })
}

/// 忽略构建产物、依赖与高频会话写入，避免工作区监视触发 UI 刷屏。
fn should_ignore_watch_path(path: &Path) -> bool {
    let s = path.to_string_lossy();
    let markers = [
        "/target/",
        "/node_modules/",
        "/.git/",
        "/dist/",
        "/.yunxi/sessions/",
        "/.yunxi/sessions\\",
    ];
    markers.iter().any(|m| s.contains(m)) || s.ends_with("/target") || s.ends_with("\\target")
}

fn parse_patent_case_meta(body: &str) -> (Option<String>, Option<String>) {
    let mut in_block = false;
    let mut id = None;
    let mut name = None;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("patentCase:") {
            in_block = true;
            continue;
        }
        if in_block {
            if !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
                if !trimmed.starts_with("patentCase") {
                    break;
                }
            }
            if let Some(v) = trimmed.strip_prefix("id:") {
                id = Some(v.trim().trim_matches('"').to_string());
            } else if let Some(v) = trimmed.strip_prefix("name:") {
                name = Some(v.trim().trim_matches('"').to_string());
            }
        }
    }

    (id, name)
}
