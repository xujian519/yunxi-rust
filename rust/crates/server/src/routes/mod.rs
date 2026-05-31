//! 路由模块

pub mod chat;
pub mod health;
pub mod knowledge;
pub mod tools;

use crate::AppState;
use axum::{routing::get, Router};

/// 构建所有 API 路由
pub fn build_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health::health_check))
        .route("/api/knowledge/search", get(knowledge::search))
        .route("/api/chat", get(chat::ws_handler))
        .route("/api/router/detect", get(knowledge::detect_domain))
        .route("/api/tools", get(tools::list_tools))
        .with_state(state)
}
