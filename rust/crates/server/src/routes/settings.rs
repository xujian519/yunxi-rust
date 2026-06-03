//! 设置 REST API

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::settings_store::{get_settings, save_settings, YunxiSettings};
use crate::AppState;

/// GET /api/settings
pub async fn get_settings_handler(
    State(_state): State<AppState>,
) -> Result<Json<YunxiSettings>, (StatusCode, String)> {
    get_settings()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// PUT /api/settings
pub async fn put_settings_handler(
    State(_state): State<AppState>,
    Json(body): Json<YunxiSettings>,
) -> Result<StatusCode, (StatusCode, String)> {
    save_settings(&body).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}
