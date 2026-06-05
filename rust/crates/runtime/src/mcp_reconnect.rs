//! MCP 断线重连装饰器。
//!
//! `ReconnectingTransport` 包装任何 `McpTransportConnection`，
//! 透明地增加指数退避重连能力。

use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use tokio::time;

use crate::mcp_remote::McpTransportConnection;

/// 最大连续重连失败次数。超过此次数后标记为 disconnected。
const MAX_RECONNECT_ATTEMPTS: u32 = 3;

/// 退避时间序列（秒）：1, 2, 4, 8, 16, 30, 30, 30, ...
const BACKOFF_SECONDS: &[u64] = &[1, 2, 4, 8, 16, 30];

/// 断线重连装饰器。
///
/// 包装任何 `McpTransportConnection`，在检测到连接断开时：
/// 1. 自动尝试重连（指数退避）；
/// 2. 重连成功后重新执行 initialize + tools/list 恢复会话；
/// 3. 连续 3 次重连失败后标记为 disconnected。
pub struct ReconnectingTransport<T: McpTransportConnection> {
    /// 被包装的底层传输。
    inner: T,
    /// 服务器名称（用于日志）。
    server_name: String,
    /// 传输类型名称（用于日志）。
    transport_type: String,
    /// 连续重连失败计数。
    consecutive_failures: u32,
    /// 是否正在重连中。
    reconnecting: bool,
}

impl<T: McpTransportConnection> ReconnectingTransport<T> {
    /// 创建重连装饰器。
    pub fn new(inner: T, server_name: &str, transport_type: &str) -> Self {
        Self {
            inner,
            server_name: server_name.to_string(),
            transport_type: transport_type.to_string(),
            consecutive_failures: 0,
            reconnecting: false,
        }
    }

    /// 获取底层传输的引用。
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// 获取底层传输的可变引用。
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// 计算第 n 次重连的退避时间。
    fn backoff_duration(&self, attempt: u32) -> Duration {
        let idx = (attempt as usize).min(BACKOFF_SECONDS.len() - 1);
        Duration::from_secs(BACKOFF_SECONDS[idx])
    }

    /// 尝试重连。
    ///
    /// 返回 `true` 表示重连成功，`false` 表示超过最大重试次数。
    pub async fn try_reconnect(&mut self) -> bool {
        self.reconnecting = true;

        for attempt in 0..MAX_RECONNECT_ATTEMPTS {
            let backoff = self.backoff_duration(attempt);
            eprintln!(
                "[mcp] 远程连接: {} [{}] 重连尝试 {}/{}, 等待 {:?}",
                self.server_name,
                self.transport_type,
                attempt + 1,
                MAX_RECONNECT_ATTEMPTS,
                backoff,
            );

            time::sleep(backoff).await;

            // 先断开旧连接
            let _ = self.inner.disconnect().await;

            match self.inner.connect().await {
                Ok(()) => {
                    eprintln!(
                        "[mcp] 远程连接: {} [{}] 重连成功",
                        self.server_name, self.transport_type
                    );
                    self.consecutive_failures = 0;
                    self.reconnecting = false;
                    return true;
                }
                Err(e) => {
                    self.consecutive_failures += 1;
                    eprintln!(
                        "[mcp] 远程连接: {} [{}] 重连失败 ({}/{}): {e}",
                        self.server_name,
                        self.transport_type,
                        attempt + 1,
                        MAX_RECONNECT_ATTEMPTS,
                    );
                }
            }
        }

        eprintln!(
            "[mcp] 远程连接: {} [{}] 连续 {} 次重连失败, 标记为 disconnected",
            self.server_name, self.transport_type, MAX_RECONNECT_ATTEMPTS,
        );
        self.reconnecting = false;
        false
    }

    /// 重连后重新初始化会话（initialize + tools/list）。
    ///
    /// 返回发现的工具列表。
    pub async fn reinitialize(&mut self) -> Result<Vec<crate::mcp_stdio::McpTool>, String> {
        use crate::mcp_remote::build_jsonrpc_request;

        // 发送 initialize
        let init_params = serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "runtime",
                "version": env!("CARGO_PKG_VERSION")
            }
        });
        let init_req = build_jsonrpc_request("initialize", Some(init_params));
        self.inner.send(init_req).await?;
        let init_resp = self.inner.receive().await?;

        if let Some(error) = init_resp.get("error") {
            return Err(format!("initialize error: {error}"));
        }

        // 发送 tools/list
        let tools_req = build_jsonrpc_request("tools/list", Some(serde_json::json!({})));
        self.inner.send(tools_req).await?;
        let tools_resp = self.inner.receive().await?;

        if let Some(error) = tools_resp.get("error") {
            return Err(format!("tools/list error: {error}"));
        }

        // 解析工具列表
        let tools_value = tools_resp
            .get("result")
            .ok_or("tools/list missing result")?;
        let tools: Vec<crate::mcp_stdio::McpTool> =
            serde_json::from_value(tools_value.get("tools").cloned().unwrap_or_default())
                .map_err(|e| format!("tools/list parse error: {e}"))?;

        Ok(tools)
    }

    /// 是否正在重连中。
    pub fn is_reconnecting(&self) -> bool {
        self.reconnecting
    }

    /// 连续重连失败次数。
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }
}

#[async_trait]
impl<T: McpTransportConnection + 'static> McpTransportConnection for ReconnectingTransport<T> {
    async fn connect(&mut self) -> Result<(), String> {
        match self.inner.connect().await {
            Ok(()) => {
                self.consecutive_failures = 0;
                Ok(())
            }
            Err(e) => {
                self.consecutive_failures += 1;
                Err(e)
            }
        }
    }

    async fn send(&mut self, request: JsonValue) -> Result<(), String> {
        match self.inner.send(request).await {
            Ok(()) => Ok(()),
            Err(e) => {
                if !self.inner.is_connected() {
                    eprintln!(
                        "[mcp] 远程连接: {} [{}] send 失败, 尝试重连",
                        self.server_name, self.transport_type
                    );
                    if self.try_reconnect().await {
                        // 重连成功后不自动重新发送原请求（调用方应重新发起）
                        Err(format!("连接断开已重连, 请重试操作: {e}"))
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn receive(&mut self) -> Result<JsonValue, String> {
        match self.inner.receive().await {
            Ok(value) => {
                self.consecutive_failures = 0;
                Ok(value)
            }
            Err(e) => {
                if !self.inner.is_connected() {
                    eprintln!(
                        "[mcp] 远程连接: {} [{}] receive 失败, 尝试重连",
                        self.server_name, self.transport_type
                    );
                    if self.try_reconnect().await {
                        Err(format!("连接断开已重连, 请重试操作: {e}"))
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.inner.disconnect().await
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }
}

// ===========================================================================
// 单元测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;

    /// 模拟传输：记录操作，可配置为失败。
    struct MockTransport {
        connected: bool,
        connect_count: u32,
        send_count: u32,
        receive_count: u32,
        disconnect_count: u32,
        fail_connect_until: u32,
        fail_send: bool,
        fail_receive: bool,
        responses: Vec<JsonValue>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                connected: false,
                connect_count: 0,
                send_count: 0,
                receive_count: 0,
                disconnect_count: 0,
                fail_connect_until: 0,
                fail_send: false,
                fail_receive: false,
                responses: vec![json!({"jsonrpc": "2.0", "id": 1, "result": {}})],
            }
        }

        fn with_fail_connect_until(mut self, n: u32) -> Self {
            self.fail_connect_until = n;
            self
        }
    }

    #[async_trait]
    impl McpTransportConnection for MockTransport {
        async fn connect(&mut self) -> Result<(), String> {
            self.connect_count += 1;
            if self.connect_count <= self.fail_connect_until {
                Err(format!("mock connect failure #{}", self.connect_count))
            } else {
                self.connected = true;
                Ok(())
            }
        }

        async fn send(&mut self, _request: JsonValue) -> Result<(), String> {
            self.send_count += 1;
            if self.fail_send {
                self.connected = false;
                Err("mock send failure".to_string())
            } else {
                Ok(())
            }
        }

        async fn receive(&mut self) -> Result<JsonValue, String> {
            self.receive_count += 1;
            if self.fail_receive {
                self.connected = false;
                Err("mock receive failure".to_string())
            } else if let Some(resp) = self.responses.first().cloned() {
                Ok(resp)
            } else {
                Err("no mock response".to_string())
            }
        }

        async fn disconnect(&mut self) -> Result<(), String> {
            self.disconnect_count += 1;
            self.connected = false;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    #[tokio::test]
    async fn connect_delegates_to_inner() {
        let mock = MockTransport::new();
        let mut transport = ReconnectingTransport::new(mock, "test", "Mock");
        assert!(transport.connect().await.is_ok());
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn send_and_receive_delegate_to_inner() {
        let mock = MockTransport::new();
        let mut transport = ReconnectingTransport::new(mock, "test", "Mock");
        transport.connect().await.unwrap();
        assert!(transport.send(json!({"test": true})).await.is_ok());
        assert!(transport.receive().await.is_ok());
    }

    #[tokio::test]
    async fn reconnect_succeeds_after_failures() {
        let mock = MockTransport::new().with_fail_connect_until(0);
        let mut transport = ReconnectingTransport::new(mock, "test", "Mock");

        // 先连接（第1次 connect，成功因为 fail_connect_until=0）
        assert!(transport.connect().await.is_ok());

        // 修改内部 mock 使接下来的连接先失败 2 次再成功
        transport.inner_mut().fail_connect_until = 2;

        // 模拟断线
        transport.inner_mut().connected = false;

        // 重连应该成功（前两次失败，第三次成功）
        let result = transport.try_reconnect().await;
        assert!(result);
        assert!(transport.is_connected());
        assert_eq!(transport.consecutive_failures(), 0);
    }

    #[tokio::test]
    async fn reconnect_fails_after_max_attempts() {
        let mock = MockTransport::new().with_fail_connect_until(100);
        let mut transport = ReconnectingTransport::new(mock, "test", "Mock");

        // 直接尝试重连（内部 connect 会持续失败）
        let result = transport.try_reconnect().await;
        assert!(!result);
        assert_eq!(transport.consecutive_failures(), MAX_RECONNECT_ATTEMPTS);
    }

    #[test]
    fn backoff_durations_follow_expected_pattern() {
        let mock = MockTransport::new();
        let transport = ReconnectingTransport::new(mock, "test", "Mock");

        assert_eq!(transport.backoff_duration(0), Duration::from_secs(1));
        assert_eq!(transport.backoff_duration(1), Duration::from_secs(2));
        assert_eq!(transport.backoff_duration(2), Duration::from_secs(4));
        assert_eq!(transport.backoff_duration(3), Duration::from_secs(8));
        assert_eq!(transport.backoff_duration(4), Duration::from_secs(16));
        assert_eq!(transport.backoff_duration(5), Duration::from_secs(30));
        assert_eq!(transport.backoff_duration(100), Duration::from_secs(30));
    }

    #[tokio::test]
    async fn disconnect_delegates_to_inner() {
        let mock = MockTransport::new();
        let mut transport = ReconnectingTransport::new(mock, "test", "Mock");
        transport.connect().await.unwrap();
        assert!(transport.disconnect().await.is_ok());
        assert!(!transport.is_connected());
    }
}
