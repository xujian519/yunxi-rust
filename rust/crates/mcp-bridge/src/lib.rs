//! MCP 工具发现与运行时调用（CLI / Server 共用）。

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use api::ToolDefinition;
use runtime::{McpServerManager, McpToolCallResult, McpTransport, RuntimeConfig};
use serde::Serialize;
use serde_json::Value as JsonValue;

/// 全局共享 tokio Runtime。
///
/// 使用 OnceLock 确保整个进程只创建一个 Runtime 实例，
/// 避免 call_tool() 每次调用都创建新 Runtime。
static SHARED_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// 获取全局共享 Runtime 的 handle。
fn shared_runtime_handle() -> tokio::runtime::Handle {
    shared_runtime().handle().clone()
}

/// 获取全局共享 Runtime。
fn shared_runtime() -> &'static tokio::runtime::Runtime {
    SHARED_RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("failed to create shared MCP runtime")
    })
}

/// MCP 运行时：发现工具并代理调用。
pub struct McpRuntime {
    manager: Arc<Mutex<McpServerManager>>,
    extra_tools: Vec<ToolDefinition>,
    servers: Vec<McpServerStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpServerStatus {
    pub name: String,
    pub transport: String,
    pub status: String,
    pub tool_count: usize,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpStatusReport {
    pub servers: Vec<McpServerStatus>,
    pub total_tools: usize,
}

impl McpRuntime {
    /// 从运行时配置加载 MCP 服务器并发现工具；失败时返回空工具集（不阻断对话）。
    pub fn try_from_config(config: &RuntimeConfig) -> Self {
        let mut manager = McpServerManager::from_runtime_config(config);
        let mut servers: Vec<McpServerStatus> = manager
            .unsupported_servers()
            .iter()
            .map(|s| McpServerStatus {
                name: s.server_name.clone(),
                transport: format!("{:?}", s.transport),
                status: "unsupported".to_string(),
                tool_count: 0,
                detail: Some(s.reason.clone()),
            })
            .collect();

        let rt = shared_runtime();
        let discovered = rt
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(30), manager.discover_tools())
                    .await
                    .unwrap_or_else(|_| {
                        eprintln!("[mcp] 工具发现总超时（30s），跳过 MCP 工具加载");
                        Ok(Vec::new())
                    })
            })
            .unwrap_or_default();

        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for tool in &discovered {
            *counts.entry(tool.server_name.clone()).or_insert(0) += 1;
        }

        for (name, cfg) in config.mcp().servers() {
            if servers.iter().any(|s| s.name == *name) {
                continue;
            }
            let count = counts.get(name).copied().unwrap_or(0);
            let transport = cfg.transport();
            let status = if count > 0 {
                "ready".to_string()
            } else if transport == McpTransport::Stdio
                || transport == McpTransport::Sse
                || transport == McpTransport::Http
                || transport == McpTransport::Ws
            {
                "configured".to_string()
            } else {
                "unsupported".to_string()
            };
            servers.push(McpServerStatus {
                name: name.clone(),
                transport: format!("{transport:?}"),
                status,
                tool_count: count,
                detail: None,
            });
        }

        let extra_tools = discovered
            .iter()
            .map(|t| ToolDefinition {
                name: t.qualified_name.clone(),
                description: t.tool.description.clone(),
                input_schema: t
                    .tool
                    .input_schema
                    .clone()
                    .unwrap_or_else(|| JsonValue::Object(Default::default())),
            })
            .collect();

        Self {
            manager: Arc::new(Mutex::new(manager)),
            extra_tools,
            servers,
        }
    }

    #[must_use]
    pub fn extra_tool_definitions(&self) -> &[ToolDefinition] {
        &self.extra_tools
    }

    #[must_use]
    pub fn is_mcp_tool(name: &str) -> bool {
        name.starts_with("mcp__")
    }

    /// 调用 MCP 工具并返回文本结果。
    ///
    /// 使用全局共享 Runtime，避免每次调用创建新 Runtime。
    #[allow(clippy::await_holding_lock)]
    pub fn call_tool(&self, name: &str, input: &str) -> Result<String, String> {
        let arguments = serde_json::from_str(input).map_err(|e| format!("invalid JSON: {e}"))?;
        let manager = Arc::clone(&self.manager);
        let name = name.to_string();
        let handle = shared_runtime_handle();
        handle.block_on(async move {
            let mut guard = manager
                .lock()
                .map_err(|_| "MCP manager lock poisoned".to_string())?;
            let response = guard
                .call_tool(&name, Some(arguments))
                .await
                .map_err(|e| e.to_string())?;
            if let Some(error) = response.error {
                return Err(format!("MCP error: {}", error.message));
            }
            let result = response
                .result
                .ok_or_else(|| "MCP empty result".to_string())?;
            format_mcp_result(&result)
        })
    }

    #[must_use]
    pub fn status_report(&self) -> McpStatusReport {
        McpStatusReport {
            total_tools: self.extra_tools.len(),
            servers: self.servers.clone(),
        }
    }
}

/// 从配置生成 MCP 状态（不启动进程）。
#[must_use]
pub fn mcp_config_status(config: &RuntimeConfig) -> McpStatusReport {
    let servers = config
        .mcp()
        .servers()
        .iter()
        .map(|(name, cfg)| {
            let transport = cfg.transport();
            let status = if transport == McpTransport::Stdio
                || transport == McpTransport::Sse
                || transport == McpTransport::Http
                || transport == McpTransport::Ws
            {
                "configured".to_string()
            } else {
                "unsupported".to_string()
            };
            McpServerStatus {
                name: name.clone(),
                transport: format!("{transport:?}"),
                status,
                tool_count: 0,
                detail: None,
            }
        })
        .collect();
    McpStatusReport {
        servers,
        total_tools: 0,
    }
}

fn format_mcp_result(result: &McpToolCallResult) -> Result<String, String> {
    if result.is_error == Some(true) {
        let text = result
            .content
            .iter()
            .filter_map(|block| block.data.get("text").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(if text.is_empty() {
            "MCP tool returned error".to_string()
        } else {
            text
        });
    }
    if let Some(structured) = &result.structured_content {
        return serde_json::to_string_pretty(structured).map_err(|e| e.to_string());
    }
    let text = result
        .content
        .iter()
        .filter_map(|block| {
            block
                .data
                .get("text")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.is_empty() {
        Ok("(MCP tool returned no text content)".to_string())
    } else {
        Ok(text)
    }
}
