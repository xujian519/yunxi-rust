//! 记忆检索 API

use axum::extract::{Query, State};
use axum::Json;
use memory::UnifiedMemory;
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct MemorySearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct MemorySearchEntry {
    pub id: String,
    pub tier: String,
    pub source: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct MemorySearchResponse {
    pub query: String,
    pub count: usize,
    pub entries: Vec<MemorySearchEntry>,
}

/// GET /api/memory/search?q=...&limit=10
pub async fn search(
    State(_state): State<AppState>,
    Query(query): Query<MemorySearchQuery>,
) -> Result<Json<MemorySearchResponse>, (axum::http::StatusCode, String)> {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let um = UnifiedMemory::default_paths()
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let entries = um.search(&query.q, limit);
    Ok(Json(MemorySearchResponse {
        query: query.q.clone(),
        count: entries.len(),
        entries: entries
            .into_iter()
            .map(|e| MemorySearchEntry {
                id: e.id,
                tier: e.tier,
                source: e.source,
                content: e.content,
            })
            .collect(),
    }))
}
