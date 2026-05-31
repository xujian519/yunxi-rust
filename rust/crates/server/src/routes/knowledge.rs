//! 知识库搜索端点

use crate::AppState;
use axum::{
    extract::{Query, State},
    response::Json,
};
use knowledge::search::SearchConfig;
use router::workflow_router::WorkflowRouter;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<Value> {
    let config = SearchConfig {
        query: params.q,
        limit: params.limit,
        ..Default::default()
    };
    let results = state
        .search_engine
        .lock()
        .map(|e| e.search(&config))
        .unwrap_or_default();
    Json(json!({
        "total": results.len(),
        "results": results,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DetectParams {
    pub input: String,
}

pub async fn detect_domain(
    State(_state): State<AppState>,
    Query(params): Query<DetectParams>,
) -> Json<Value> {
    let router = WorkflowRouter::default();
    let decision = router.route(&params.input);
    Json(json!({
        "domain": format!("{}", decision.domain),
        "complexity": format!("{}", decision.complexity),
        "confidence": decision.confidence,
        "reasoning": decision.reasoning,
        "suggested_tools": decision.suggested_tools,
        "suggested_agents": decision.suggested_agents,
    }))
}
