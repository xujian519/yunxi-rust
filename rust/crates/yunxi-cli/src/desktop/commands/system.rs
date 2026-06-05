//! 系统级 IPC：OAuth、doctor、init、斜杠命令、MCP 状态。

use std::path::PathBuf;

use runtime::{ConfigLoader, PermissionMode};
use serde::Serialize;
use yunxi_cli::desktop_slash::{
    execute_desktop_slash, load_mcp_status, run_init_claude_md, run_workspace_init,
    DesktopSlashContext, SlashExecuteResult,
};
use yunxi_cli::doctor::{collect_doctor_report, DoctorReport};
use yunxi_cli::mcp_runtime::McpStatusReport;
use yunxi_cli::oauth_flow::{oauth_configured, run_login, run_logout};

#[derive(Debug, Serialize)]
pub struct OAuthStatus {
    pub configured: bool,
}

#[tauri::command]
pub fn oauth_status() -> OAuthStatus {
    OAuthStatus {
        configured: oauth_configured(),
    }
}

#[tauri::command]
pub async fn oauth_login() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| run_login().map_err(|e| e.to_string()))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn oauth_logout() -> Result<(), String> {
    run_logout().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn run_doctor_check() -> DoctorReport {
    collect_doctor_report()
}

#[tauri::command]
pub fn init_workspace() -> Result<String, String> {
    let root = workspace_root()?;
    run_workspace_init(&root)
}

#[tauri::command]
pub fn init_claude_md() -> Result<String, String> {
    run_init_claude_md()
}

#[tauri::command]
pub fn get_mcp_status() -> Result<McpStatusReport, String> {
    let root = workspace_root()?;
    load_mcp_status(&root)
}

#[tauri::command]
pub fn execute_slash_command(
    session_id: String,
    input: String,
    model: Option<String>,
    workspace_root_arg: Option<String>,
) -> Result<Option<SlashExecuteResult>, String> {
    let root = workspace_root_arg
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().unwrap_or_else(|_| PathBuf::from(".")));
    let handle = yunxi_cli::session_mgr::resolve_session_reference(&session_id)
        .map_err(|e| e.to_string())?;
    let configured_model =
        model.unwrap_or_else(|| yunxi_cli::model_routing::default_model_from_config());
    let ctx = DesktopSlashContext {
        session_id,
        session_path: handle.path,
        model: configured_model,
        permission_mode: PermissionMode::DangerFullAccess,
        workspace_root: root,
    };
    execute_desktop_slash(&input, &ctx)
}

fn workspace_root() -> Result<PathBuf, String> {
    yunxi_cli::session_mgr::workspace_root().map_err(|e| e.to_string())
}

/// 读取合并配置中的 MCP 服务器列表（供设置页展示）。
#[tauri::command]
pub fn get_mcp_config() -> Result<serde_json::Value, String> {
    let root = workspace_root()?;
    let config = ConfigLoader::default_for(root)
        .load()
        .map_err(|e| e.to_string())?;
    let servers: serde_json::Map<String, serde_json::Value> = config
        .mcp()
        .servers()
        .iter()
        .map(|(name, cfg)| {
            let detail = match &cfg.config {
                runtime::McpServerConfig::Stdio(s) => serde_json::json!({
                    "command": s.command,
                    "args": s.args,
                }),
                runtime::McpServerConfig::Http(h) | runtime::McpServerConfig::Sse(h) => {
                    serde_json::json!({ "url": h.url })
                }
                runtime::McpServerConfig::Ws(w) => serde_json::json!({ "url": w.url }),
                _ => serde_json::json!({}),
            };
            (
                name.clone(),
                serde_json::json!({
                    "transport": format!("{:?}", cfg.transport()),
                    "detail": detail,
                }),
            )
        })
        .collect();
    Ok(serde_json::Value::Object(servers))
}
