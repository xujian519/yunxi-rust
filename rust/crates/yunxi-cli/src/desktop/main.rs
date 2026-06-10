#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dependency_on_unit_never_type_fallback)]

mod commands;
mod paths;
mod shell_session;
mod state;
mod stream;

use std::sync::Arc;

use commands::{
    abstract_drafter, case_create, case_delete, case_list, case_load, case_save, chat_cancel,
    chat_send, check_compliance, claim_formality_check, claim_generator, claim_parse,
    examiner_simulate, execute_slash_command, execute_tool_raw, formal_check, get_mcp_config,
    get_mcp_status, get_pipeline_config, get_settings, get_usage, get_version, get_workspace_info,
    greet, hybrid_retrieval, import_project_materials, infringement_analysis, init_claude_md,
    init_workspace, innovation_evaluator, inventiveness_analysis, knowledge_card, knowledge_search,
    law_query, legal_reasoning, libreoffice_convert, list_directory, list_project_materials,
    list_reasoning_phases, list_rule_types, llm_auth_configured, memory_search, novelty_analysis,
    oa_parse, oa_strategy, oauth_login, oauth_logout, oauth_status, patent_compare, patent_search,
    permission_respond, pick_workspace_folder, quality_checker, quality_scorer,
    record_intent_preference, response_template, run_doctor_check, run_reasoning, save_llm_api_key,
    save_settings, scan_workspace_roots, semantic_compare, session_create, session_delete,
    session_list, session_load, session_save, shell_exec, spec_formality_check,
    specification_drafter, success_predictor, super_reasoning_plan, workspace_watch_start,
    workspace_watch_stop,
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
            patent_compare,
            knowledge_search,
            memory_search,
            novelty_analysis,
            inventiveness_analysis,
            claim_generator,
            abstract_drafter,
            specification_drafter,
            quality_scorer,
            quality_checker,
            formal_check,
            claim_formality_check,
            spec_formality_check,
            oa_strategy,
            response_template,
            success_predictor,
            infringement_analysis,
            legal_reasoning,
            examiner_simulate,
            hybrid_retrieval,
            oa_parse,
            claim_parse,
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
            oauth_status,
            oauth_login,
            oauth_logout,
            run_doctor_check,
            init_workspace,
            init_claude_md,
            execute_slash_command,
            get_mcp_status,
            get_mcp_config,
            execute_tool_raw,
            run_reasoning,
            list_reasoning_phases,
            get_pipeline_config,
            check_compliance,
            list_rule_types,
            record_intent_preference,
            law_query,
            knowledge_card,
            super_reasoning_plan,
            innovation_evaluator,
            semantic_compare,
            libreoffice_convert,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
