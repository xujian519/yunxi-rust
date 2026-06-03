//! MCP 状态 API

use axum::extract::{Query, State};
use axum::Json;
use mcp_bridge::{mcp_config_status, McpRuntime, McpStatusReport};
use runtime::ConfigLoader;
use serde::Deserialize;

use crate::agent_bridge;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct McpStatusQuery {
    /// 为 true 时启动 MCP 进程并发现工具（较慢）。
    pub discover: Option<bool>,
}

/// GET /api/mcp/status — 返回 MCP 服务器配置/发现状态
pub async fn mcp_status(
    State(_state): State<AppState>,
    Query(query): Query<McpStatusQuery>,
) -> Json<McpStatusReport> {
    let root = agent_bridge::workspace_root();
    let config = ConfigLoader::default_for(root)
        .load()
        .unwrap_or_else(|_| runtime::RuntimeConfig::empty());

    let report = if query.discover == Some(true) {
        McpRuntime::try_from_config(&config).status_report()
    } else {
        mcp_config_status(&config)
    };
    Json(report)
}
