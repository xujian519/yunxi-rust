mod case;
mod chat;
mod materials;
mod session;
mod settings;
mod shell;
mod tools;
mod workspace;

pub use case::{case_create, case_delete, case_list, case_load, case_save, get_workspace_info};
pub use chat::{chat_cancel, chat_send, permission_respond};
pub use materials::{import_project_materials, list_project_materials};
pub use session::{session_create, session_delete, session_list, session_load, session_save};
pub use settings::{get_settings, get_usage, llm_auth_configured, save_llm_api_key, save_settings};
pub use shell::shell_exec;
pub use tools::{knowledge_search, patent_search};
pub use workspace::{
    list_directory, pick_workspace_folder, scan_workspace_roots, workspace_watch_start,
    workspace_watch_stop,
};

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("你好, {}! 云熙智能体已就绪。", name)
}

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
