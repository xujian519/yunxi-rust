//! 路由模块

pub mod cases;
pub mod chat;
pub mod health;
pub mod knowledge;
pub mod mcp;
pub mod memory;
pub mod permission;
pub mod sessions;
pub mod settings;
pub mod tools;

use crate::AppState;
use axum::routing::{get, post, put, Router};

/// 构建所有 API 路由
pub fn build_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/knowledge/search", get(knowledge::search))
        .route("/api/chat", get(chat::ws_handler))
        .route("/api/router/detect", get(knowledge::detect_domain))
        .route("/api/tools", get(tools::list_tools))
        .route("/api/tools/execute", tools::execute_route())
        .route("/api/mcp/status", get(mcp::mcp_status))
        .route("/api/memory/search", get(memory::search))
        .route("/api/settings", get(settings::get_settings_handler).put(settings::put_settings_handler))
        .route("/api/cases", get(cases::list_cases).post(cases::create_case))
        .route(
            "/api/cases/{id}",
            get(cases::load_case)
                .put(cases::save_case)
                .delete(cases::delete_case),
        )
        .route("/api/chat/permission", post(permission::permission_respond))
        .route("/api/sessions", get(sessions::list_sessions).post(sessions::create_session))
        .route(
            "/api/sessions/{id}",
            get(sessions::load_session)
                .put(sessions::save_session)
                .delete(sessions::delete_session),
        )
        .with_state(state)
}
