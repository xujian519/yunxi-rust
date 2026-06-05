//! 会话 REST API（HTTP 模式持久化）

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::session_store::SessionMeta;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SessionCreateBody {
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionCreateResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionSaveBody {
    pub session_json: String,
}

#[derive(Debug, Serialize)]
pub struct SessionSaveResponse {
    pub id: String,
}

/// GET /api/sessions
pub async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<Vec<SessionMeta>>, (StatusCode, String)> {
    state
        .session_store
        .list()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// GET /api/sessions/:id
pub async fn load_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<String, (StatusCode, String)> {
    let session = state
        .session_store
        .load(&id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(session.to_json().render())
}

/// POST /api/sessions
pub async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<SessionCreateBody>,
) -> Result<Json<SessionCreateResponse>, (StatusCode, String)> {
    let title = body.title.unwrap_or_else(|| "云熙对话".to_string());
    let id = state
        .session_store
        .create(&title)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if let Ok(mut cache) = state.chat_sessions.lock() {
        cache.insert(id.clone(), runtime::Session::new());
    }
    Ok(Json(SessionCreateResponse { id }))
}

/// PUT /api/sessions/:id
pub async fn save_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<SessionSaveBody>,
) -> Result<Json<SessionSaveResponse>, (StatusCode, String)> {
    let path = std::env::temp_dir().join(format!("yunxi-validate-{id}.json"));
    std::fs::write(&path, &body.session_json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let session = runtime::Session::load_from_path(&path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    let _ = std::fs::remove_file(&path);

    state
        .session_store
        .save(&id, &session)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if let Ok(mut cache) = state.chat_sessions.lock() {
        cache.insert(id.clone(), session);
    }
    Ok(Json(SessionSaveResponse { id }))
}

/// DELETE /api/sessions/:id
pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .session_store
        .delete(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    if let Ok(mut cache) = state.chat_sessions.lock() {
        cache.remove(&id);
    }
    Ok(StatusCode::NO_CONTENT)
}
