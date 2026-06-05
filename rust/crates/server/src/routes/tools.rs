//! 工具列表与执行 API

use axum::extract::State;
use axum::http::StatusCode;
use axum::{routing::post, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tools::{execute_tool, mvp_tool_specs};

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

#[derive(Debug, Deserialize)]
pub struct ToolExecuteRequest {
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Serialize)]
pub struct ToolExecuteResponse {
    pub result: String,
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

/// POST /api/tools/execute — 执行指定工具
pub async fn execute_tool_handler(
    State(_state): State<AppState>,
    Json(body): Json<ToolExecuteRequest>,
) -> Result<Json<ToolExecuteResponse>, (StatusCode, String)> {
    let result = execute_tool(&body.name, &body.input).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    Ok(Json(ToolExecuteResponse { result }))
}

/// 工具执行路由（供 mod 注册）
pub fn execute_route() -> axum::routing::MethodRouter<AppState> {
    post(execute_tool_handler)
}
