mod case;
mod chat;
mod constitutional;
mod materials;
mod reasoning;
mod session;
mod settings;
mod shell;
mod system;
mod tools;
mod workspace;

pub use case::{case_create, case_delete, case_list, case_load, case_save, get_workspace_info};
pub use chat::{chat_cancel, chat_send, permission_respond};
pub use constitutional::{check_compliance, list_rule_types};
pub use materials::{import_project_materials, list_project_materials};
pub use reasoning::{get_pipeline_config, list_reasoning_phases, run_reasoning};
pub use session::{session_create, session_delete, session_list, session_load, session_save};
pub use settings::{get_settings, get_usage, llm_auth_configured, save_llm_api_key, save_settings};
pub use shell::shell_exec;
pub use system::{
    execute_slash_command, get_mcp_config, get_mcp_status, init_claude_md, init_workspace,
    oauth_login, oauth_logout, oauth_status, run_doctor_check,
};
pub use tools::{
    abstract_drafter, claim_formality_check, claim_generator, claim_parse, examiner_simulate,
    execute_tool_raw, formal_check, hybrid_retrieval, infringement_analysis, innovation_evaluator,
    inventiveness_analysis, knowledge_card, knowledge_search, law_query, legal_reasoning,
    memory_search, novelty_analysis, oa_parse, oa_strategy, patent_compare, patent_search,
    quality_checker, quality_scorer, record_intent_preference, response_template, semantic_compare,
    spec_formality_check, specification_drafter, success_predictor, super_reasoning_plan,
};
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
