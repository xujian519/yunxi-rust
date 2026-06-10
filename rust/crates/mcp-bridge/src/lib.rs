//! MCP 工具发现与运行时调用（CLI / Server 共用）。

use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use api::ToolDefinition;
use runtime::{ManagedMcpTool, McpServerManager, McpToolCallResult, McpTransport, RuntimeConfig};
use serde::Serialize;
use serde_json::Value as JsonValue;
/// 全局共享 tokio Runtime。
///
/// 使用 LazyLock 确保整个进程只创建一个 Runtime 实例，
/// 消除 OnceLock 的 get_or_init 调用。
static SHARED_RUNTIME: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Runtime::new().expect("failed to create shared MCP runtime"));

/// 获取全局共享 Runtime 的 handle。
fn shared_runtime_handle() -> tokio::runtime::Handle {
    SHARED_RUNTIME.handle().clone()
}

/// 获取全局共享 Runtime 的引用。
fn shared_runtime() -> &'static tokio::runtime::Runtime {
    &SHARED_RUNTIME
}

/// 收集发现的工具，按服务器名称计数。
fn count_discovered_tools(discovered: &[ManagedMcpTool]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for tool in discovered {
        *counts.entry(tool.server_name.clone()).or_insert(0) += 1;
    }
    counts
}

/// 将发现的工具转换为 ToolDefinition 列表。
fn discovered_to_tool_defs(discovered: &[ManagedMcpTool]) -> Vec<ToolDefinition> {
    discovered
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
        .collect()
}

/// 从配置构建服务器状态列表（含 supported + unsupported）。
fn build_server_list(
    config: &RuntimeConfig,
    counts: &BTreeMap<String, usize>,
) -> Vec<McpServerStatus> {
    let mut servers: Vec<McpServerStatus> = Vec::new();
    for (name, cfg) in config.mcp().servers() {
        let transport = format!("{:?}", cfg.transport());
        let count = counts.get(name).copied().unwrap_or(0);
        let status = if count > 0 {
            "ready".to_string()
        } else if matches!(
            cfg.transport(),
            McpTransport::Stdio | McpTransport::Sse | McpTransport::Http | McpTransport::Ws
        ) {
            "configured".to_string()
        } else {
            "unsupported".to_string()
        };
        servers.push(McpServerStatus {
            name: name.clone(),
            transport,
            status,
            tool_count: count,
            detail: None,
        });
    }
    servers
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
    ///
    /// 异步版本，适合在已有 tokio Runtime 的上下文中调用。
    pub async fn try_from_config_async(config: &RuntimeConfig) -> Self {
        let mut manager = McpServerManager::from_runtime_config(config);
        let discovered = tokio::time::timeout(Duration::from_secs(30), manager.discover_tools())
            .await
            .unwrap_or_else(|_| {
                eprintln!("[mcp] 工具发现总超时（30s），跳过 MCP 工具加载");
                Ok(Vec::new())
            })
            .unwrap_or_default();
        let counts = count_discovered_tools(&discovered);
        let servers = build_server_list(config, &counts);
        Self::new(manager, discovered, servers)
    }

    /// 从运行时配置加载 MCP 服务器并发现工具；失败时返回空工具集（不阻断对话）。
    ///
    /// 使用全局共享 Runtime 执行异步发现。
    pub fn try_from_config(config: &RuntimeConfig) -> Self {
        let mut manager = McpServerManager::from_runtime_config(config);
        let discovered = shared_runtime()
            .block_on(async {
                tokio::time::timeout(Duration::from_secs(30), manager.discover_tools())
                    .await
                    .unwrap_or_else(|_| {
                        eprintln!("[mcp] 工具发现总超时（30s），跳过 MCP 工具加载");
                        Ok(Vec::new())
                    })
            })
            .unwrap_or_default();
        let counts = count_discovered_tools(&discovered);
        let servers = build_server_list(config, &counts);
        Self::new(manager, discovered, servers)
    }

    /// 使用已发现的工具构造 McpRuntime。
    fn new(
        manager: McpServerManager,
        discovered: Vec<ManagedMcpTool>,
        servers: Vec<McpServerStatus>,
    ) -> Self {
        let extra_tools = discovered_to_tool_defs(&discovered);
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

    /// 异步调用 MCP 工具并返回文本结果。
    ///
    /// 适合在已有 tokio Runtime 的上下文中调用。
    #[allow(clippy::await_holding_lock)]
    pub async fn call_tool_async(&self, name: &str, input: &str) -> Result<String, String> {
        let arguments = serde_json::from_str(input).map_err(|e| format!("invalid JSON: {e}"))?;
        let manager = Arc::clone(&self.manager);
        let name = name.to_string();
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
