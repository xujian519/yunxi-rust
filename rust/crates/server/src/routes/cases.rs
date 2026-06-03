//! 案件 REST API

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use crate::case_store::{CaseStore, PatentCase};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CaseCreateBody {
    pub name: String,
    pub application_number: Option<String>,
}

/// GET /api/cases
pub async fn list_cases(
    State(_state): State<AppState>,
) -> Result<Json<Vec<PatentCase>>, (StatusCode, String)> {
    CaseStore::list()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// GET /api/cases/{id}
pub async fn load_case(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PatentCase>, (StatusCode, String)> {
    CaseStore::load(&id)
        .map(Json)
        .map_err(|e| (StatusCode::NOT_FOUND, e))
}

/// POST /api/cases
pub async fn create_case(
    State(_state): State<AppState>,
    Json(body): Json<CaseCreateBody>,
) -> Result<Json<PatentCase>, (StatusCode, String)> {
    CaseStore::create(body.name, body.application_number)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}

/// PUT /api/cases/{id}
pub async fn save_case(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(case_data): Json<PatentCase>,
) -> Result<Json<PatentCase>, (StatusCode, String)> {
    if case_data.id != id {
        return Err((StatusCode::BAD_REQUEST, "case id mismatch".into()));
    }
    CaseStore::save(&case_data).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(case_data))
}

/// DELETE /api/cases/{id}
pub async fn delete_case(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    CaseStore::delete(&id).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(StatusCode::NO_CONTENT)
}
