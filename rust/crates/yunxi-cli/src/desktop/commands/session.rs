//! 会话管理 IPC。

use runtime::Session;
use serde::Serialize;
use serde_json::Value;
use yunxi_cli::session_mgr::{
    create_managed_session_handle, list_managed_sessions, resolve_session_reference,
};

#[derive(Debug, Serialize)]
pub struct SessionMeta {
    pub id: String,
    pub message_count: usize,
    pub modified_at: u64,
}

#[derive(Debug, Serialize)]
pub struct SessionCreateResult {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct SessionSaveResult {
    pub id: String,
}

#[tauri::command]
pub fn session_list() -> Result<Vec<SessionMeta>, String> {
    list_managed_sessions()
        .map(|sessions| {
            sessions
                .into_iter()
                .map(|s| SessionMeta {
                    id: s.id,
                    message_count: s.message_count,
                    modified_at: s.modified_epoch_secs,
                })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn session_load(id: String) -> Result<String, String> {
    let handle = resolve_session_reference(&id).map_err(|e| e.to_string())?;
    let session = Session::load_from_path(&handle.path).map_err(|e| e.to_string())?;
    Ok(session.to_json().render())
}

#[tauri::command]
pub fn session_save(session_json: String) -> Result<SessionSaveResult, String> {
    let parsed: Value =
        serde_json::from_str(&session_json).map_err(|e| format!("invalid session json: {e}"))?;

    let handle = if let Some(session_id) = parsed.get("id").and_then(Value::as_str) {
        resolve_session_reference(session_id).map_err(|e| e.to_string())?
    } else {
        create_managed_session_handle().map_err(|e| e.to_string())?
    };

    let validate_path = std::env::temp_dir().join(format!("yunxi-validate-{}.json", handle.id));
    std::fs::write(&validate_path, &session_json).map_err(|e| e.to_string())?;
    Session::load_from_path(&validate_path).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&validate_path);

    std::fs::write(&handle.path, session_json).map_err(|e| e.to_string())?;

    Ok(SessionSaveResult { id: handle.id })
}

#[tauri::command]
pub fn session_create(title: String) -> Result<SessionCreateResult, String> {
    let handle = create_managed_session_handle().map_err(|e| e.to_string())?;
    let mut session = Session::new();
    if !title.is_empty() {
        session
            .messages
            .push(runtime::ConversationMessage::user_text(format!(
                "[session title: {title}]"
            )));
    }
    session
        .save_to_path(&handle.path)
        .map_err(|e| e.to_string())?;
    Ok(SessionCreateResult { id: handle.id })
}
