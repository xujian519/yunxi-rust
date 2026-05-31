use crate::AppState;
use axum::{extract::State, response::Json};
use serde_json::{json, Value};

pub async fn health_check(State(state): State<AppState>) -> Json<Value> {
    // 健康检查操作极快，直接锁定即可；若需保护长时间搜索，才用 spawn_blocking
    let search_status = state
        .search_engine
        .lock()
        .map(|e| e.status())
        .unwrap_or_else(|_| serde_json::json!({"error": "locked"}));
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "knowledge": search_status,
    }))
}
