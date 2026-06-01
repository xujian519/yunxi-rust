#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod paths;
mod shell_session;
mod state;
mod stream;

use std::sync::Arc;

use commands::{
    case_create, case_delete, case_list, case_load, case_save, chat_cancel, chat_send,
    get_settings, get_usage, get_version, get_workspace_info, greet, import_project_materials,
    knowledge_search, list_directory, list_project_materials, llm_auth_configured, patent_search,
    permission_respond, pick_workspace_folder, save_llm_api_key, save_settings,
    scan_workspace_roots, session_create, session_delete, session_list, session_load, session_save,
    shell_exec, workspace_watch_start, workspace_watch_stop,
};
use paths::init_desktop_environment;
use shell_session::{
    shell_session_close, shell_session_resize, shell_session_start, shell_session_write,
};
use state::DesktopState;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            init_desktop_environment().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

            // 窗口 / Dock 图标（需 icons/ 与 tauri bundle 配置；macOS Dock 需 .app 启动）
            if let Some(icon) = app.default_window_icon().cloned() {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon);
                }
            }

            Ok(())
        })
        .manage(Arc::new(DesktopState::new()))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_version,
            get_settings,
            save_settings,
            get_usage,
            get_workspace_info,
            session_list,
            session_load,
            session_save,
            session_create,
            session_delete,
            case_list,
            case_load,
            case_save,
            case_create,
            case_delete,
            patent_search,
            knowledge_search,
            chat_send,
            chat_cancel,
            permission_respond,
            pick_workspace_folder,
            scan_workspace_roots,
            list_directory,
            shell_exec,
            shell_session_start,
            shell_session_write,
            shell_session_close,
            shell_session_resize,
            llm_auth_configured,
            save_llm_api_key,
            workspace_watch_start,
            workspace_watch_stop,
            list_project_materials,
            import_project_materials,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
