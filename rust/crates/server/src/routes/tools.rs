//! 工具列表 API

use axum::{extract::State, Json};
use serde::Serialize;
use tools::mvp_tool_specs;

use crate::AppState;

#[derive(Serialize)]
pub(crate) struct ToolListItem {
    name: &'static str,
    description: &'static str,
    required_permission: String,
}

#[derive(Serialize)]
pub struct ToolsResponse {
    count: usize,
    tools: Vec<ToolListItem>,
}

/// GET /api/tools — 返回当前运行时注册的全部工具规格
pub async fn list_tools(State(_state): State<AppState>) -> Json<ToolsResponse> {
    let specs = mvp_tool_specs();
    let tools: Vec<ToolListItem> = specs
        .into_iter()
        .map(|spec| ToolListItem {
            name: spec.name,
            description: spec.description,
            required_permission: format!("{:?}", spec.required_permission),
        })
        .collect();
    let count = tools.len();
    Json(ToolsResponse { count, tools })
}
