//! 权限 REST 回退（WebSocket 不可用时的备用）

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::permission::resolve_permission;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PermissionRespondBody {
    pub request_id: String,
    pub outcome: String,
}

/// POST /api/chat/permission
pub async fn permission_respond(
    State(state): State<AppState>,
    Json(body): Json<PermissionRespondBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    let allow = matches!(body.outcome.as_str(), "allow" | "always");
    if !resolve_permission(&state.permission_waiters, &body.request_id, allow) {
        return Err((
            StatusCode::NOT_FOUND,
            format!("permission request not found: {}", body.request_id),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}
