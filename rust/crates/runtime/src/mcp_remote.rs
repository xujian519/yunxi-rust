//! MCP 远程传输层：SSE / HTTP / WebSocket 三种传输实现。
//!
//! 定义统一的 `McpTransportConnection` trait，以及 SSE、HTTP、WS 三种具体实现。

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde_json::Value as JsonValue;
use tokio::sync::mpsc;
use tokio::time;
use tokio_tungstenite::tungstenite::{self, client::IntoClientRequest, Message};

use crate::mcp_client::{McpClientTransport, McpRemoteTransport};

// ---------------------------------------------------------------------------
// McpTransportConnection trait
// ---------------------------------------------------------------------------

/// 统一的 MCP 传输连接 trait。
///
/// 所有 MCP 传输方式（Stdio/SSE/HTTP/WS）均实现此 trait，
/// 以便上层代码（重连装饰器、连接池）可以透明地使用任意传输。
#[async_trait]
pub trait McpTransportConnection: Send + Sync {
    /// 建立连接。
    async fn connect(&mut self) -> Result<(), String>;
    /// 发送 JSON-RPC 请求。
    async fn send(&mut self, request: JsonValue) -> Result<(), String>;
    /// 接收 JSON-RPC 响应。
    async fn receive(&mut self) -> Result<JsonValue, String>;
    /// 断开连接。
    async fn disconnect(&mut self) -> Result<(), String>;
    /// 是否已连接。
    fn is_connected(&self) -> bool;
}

// ---------------------------------------------------------------------------
// Request ID 生成器 & JSON-RPC builder
// ---------------------------------------------------------------------------

static GLOBAL_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// 生成全局唯一的 JSON-RPC request id。
pub fn next_request_id() -> u64 {
    GLOBAL_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

/// 构建 JSON-RPC 2.0 请求体。
pub fn build_jsonrpc_request(method: &str, params: Option<JsonValue>) -> JsonValue {
    let id = next_request_id();
    let mut request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
    });
    if let Some(p) = params {
        request["params"] = p;
    }
    request
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// 判断 `McpClientTransport` 是否为远程传输（SSE/HTTP/WS）。
pub fn is_remote_transport(transport: &McpClientTransport) -> bool {
    matches!(
        transport,
        McpClientTransport::Sse(_) | McpClientTransport::Http(_) | McpClientTransport::WebSocket(_)
    )
}

/// 从 `McpRemoteTransport` 中提取 HTTP headers。
fn build_headers(remote: &McpRemoteTransport) -> BTreeMap<String, String> {
    remote.headers.clone()
}

/// 为 reqwest 请求设置 headers。
fn apply_reqwest_headers(
    builder: reqwest::RequestBuilder,
    headers: &BTreeMap<String, String>,
) -> reqwest::RequestBuilder {
    let mut builder = builder;
    for (key, value) in headers {
        builder = builder.header(key.as_str(), value.as_str());
    }
    builder
}

/// 获取 McpClientTransport 的 variant 名称。
pub fn transport_variant_name(transport: &McpClientTransport) -> &'static str {
    match transport {
        McpClientTransport::Stdio(_) => "Stdio",
        McpClientTransport::Sse(_) => "SSE",
        McpClientTransport::Http(_) => "HTTP",
        McpClientTransport::WebSocket(_) => "WebSocket",
        McpClientTransport::Sdk(_) => "SDK",
        McpClientTransport::ClaudeAiProxy(_) => "ClaudeAiProxy",
    }
}

// ===========================================================================
// SseTransport
// ===========================================================================

/// SSE 传输：GET /sse 建立事件流，POST /message 发送请求。
pub struct SseTransport {
    base_url: String,
    headers: BTreeMap<String, String>,
    client: Client,
    post_endpoint: Option<String>,
    incoming_rx: Option<mpsc::Receiver<Result<JsonValue, String>>>,
    connected: bool,
    io_task: Option<tokio::task::JoinHandle<()>>,
}

impl SseTransport {
    /// 创建 SSE 传输实例。
    pub fn new(remote: &McpRemoteTransport) -> Self {
        Self {
            base_url: remote.url.clone(),
            headers: build_headers(remote),
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            post_endpoint: None,
            incoming_rx: None,
            connected: false,
            io_task: None,
        }
    }
}

impl Drop for SseTransport {
    fn drop(&mut self) {
        if let Some(handle) = self.io_task.take() {
            handle.abort();
        }
    }
}

#[async_trait]
impl McpTransportConnection for SseTransport {
    async fn connect(&mut self) -> Result<(), String> {
        let sse_url = if self.base_url.ends_with("/sse") {
            self.base_url.clone()
        } else {
            format!("{}/sse", self.base_url.trim_end_matches('/'))
        };

        eprintln!("[mcp] 远程连接: SSE {sse_url} [connecting]");

        let (tx, rx) = mpsc::channel(64);
        let client = self.client.clone();
        let headers = self.headers.clone();

        let sse_url_clone = sse_url.clone();
        let handle = tokio::spawn(async move {
            let request = client.get(&sse_url_clone);
            let request = apply_reqwest_headers(request, &headers);

            match request.send().await {
                Ok(response) => {
                    let event_stream = response.bytes_stream().eventsource();
                    let mut stream = std::pin::pin!(event_stream);

                    while let Some(event_result) = stream.next().await {
                        match event_result {
                            Ok(event) => {
                                let data = event.data.clone();
                                if let Ok(value) = serde_json::from_str::<JsonValue>(&data) {
                                    if tx.send(Ok(value)).await.is_err() {
                                        break;
                                    }
                                } else {
                                    let endpoint_msg = serde_json::json!({
                                        "_sse_event": event.event.clone(),
                                        "_sse_data": data,
                                    });
                                    if tx.send(Ok(endpoint_msg)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(Err(format!("SSE stream error: {e}"))).await;
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("SSE connection failed: {e}"))).await;
                }
            }
        });

        self.io_task = Some(handle);
        self.incoming_rx = Some(rx);

        // 等待 endpoint 事件
        let post_endpoint = {
            let rx = self.incoming_rx.as_mut().ok_or("no incoming channel")?;
            let deadline = time::sleep(Duration::from_secs(10));
            tokio::pin!(deadline);

            let endpoint;
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(Ok(value)) => {
                                if let Some(data) = value.get("_sse_data").and_then(|d| d.as_str()) {
                                    let event_type = value.get("_sse_event").and_then(|e| e.as_str()).unwrap_or("");
                                    if event_type == "endpoint" || data.starts_with('/') {
                                        endpoint = Some(data.to_string());
                                        break;
                                    }
                                }
                            }
                            Some(Err(e)) => return Err(e),
                            None => return Err("SSE stream closed before endpoint received".to_string()),
                        }
                    }
                    _ = &mut deadline => {
                        endpoint = Some("/message".to_string());
                        break;
                    }
                }
            }
            endpoint
        };

        let base = self.base_url.trim_end_matches("/sse").trim_end_matches('/');
        self.post_endpoint = Some(format!(
            "{}{}",
            base,
            post_endpoint.as_deref().unwrap_or("/message")
        ));
        self.connected = true;

        eprintln!(
            "[mcp] 远程连接: SSE {sse_url} [connected] post_endpoint={}",
            self.post_endpoint.as_deref().unwrap_or("?")
        );
        Ok(())
    }

    async fn send(&mut self, request: JsonValue) -> Result<(), String> {
        let endpoint = self
            .post_endpoint
            .clone()
            .ok_or("SSE not connected: no post endpoint")?;

        let builder = self.client.post(&endpoint);
        let builder = apply_reqwest_headers(builder, &self.headers);
        let builder = builder.json(&request);

        let response = builder
            .send()
            .await
            .map_err(|e| format!("SSE POST failed: {e}"))?;

        if !response.status().is_success() && response.status().as_u16() != 202 {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("SSE POST returned {status}: {body}"));
        }

        Ok(())
    }

    async fn receive(&mut self) -> Result<JsonValue, String> {
        let rx = self
            .incoming_rx
            .as_mut()
            .ok_or("SSE not connected: no incoming channel")?;

        match rx.recv().await {
            Some(result) => result,
            None => {
                self.connected = false;
                Err("SSE stream closed".to_string())
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        self.incoming_rx = None;
        self.post_endpoint = None;
        eprintln!("[mcp] 远程连接: SSE [disconnected]");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

// ===========================================================================
// HttpTransport
// ===========================================================================

/// HTTP 传输：JSON-RPC over HTTP POST。
pub struct HttpTransport {
    url: String,
    headers: BTreeMap<String, String>,
    client: Client,
    connected: bool,
    pending_request: Option<JsonValue>,
}

impl HttpTransport {
    /// 创建 HTTP 传输实例。
    pub fn new(remote: &McpRemoteTransport) -> Self {
        Self {
            url: remote.url.clone(),
            headers: build_headers(remote),
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            connected: false,
            pending_request: None,
        }
    }
}

#[async_trait]
impl McpTransportConnection for HttpTransport {
    async fn connect(&mut self) -> Result<(), String> {
        eprintln!("[mcp] 远程连接: HTTP {} [connecting]", self.url);
        self.connected = true;
        eprintln!("[mcp] 远程连接: HTTP {} [connected]", self.url);
        Ok(())
    }

    async fn send(&mut self, request: JsonValue) -> Result<(), String> {
        self.pending_request = Some(request);
        Ok(())
    }

    async fn receive(&mut self) -> Result<JsonValue, String> {
        let request = self
            .pending_request
            .take()
            .ok_or("HTTP transport: no pending request to send")?;

        let builder = self.client.post(&self.url);
        let builder = apply_reqwest_headers(builder, &self.headers);
        let builder = builder.json(&request);

        let response = builder
            .send()
            .await
            .map_err(|e| format!("HTTP POST failed: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("HTTP POST returned {status}: {body}"));
        }

        let body = response
            .text()
            .await
            .map_err(|e| format!("HTTP read body failed: {e}"))?;

        serde_json::from_str::<JsonValue>(&body)
            .map_err(|e| format!("HTTP response JSON parse error: {e}"))
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        self.pending_request = None;
        eprintln!("[mcp] 远程连接: HTTP [disconnected]");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

// ===========================================================================
// WsTransport
// ===========================================================================

/// WebSocket 传输：双向 JSON-RPC over WebSocket，含心跳保活。
pub struct WsTransport {
    url: String,
    headers: BTreeMap<String, String>,
    send_tx: Option<mpsc::Sender<String>>,
    incoming_rx: Option<mpsc::Receiver<Result<JsonValue, String>>>,
    ping_stop_tx: Option<mpsc::Sender<()>>,
    connected: bool,
    io_task: Option<tokio::task::JoinHandle<()>>,
}

/// WebSocket 流类型别名。
type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

impl WsTransport {
    /// 创建 WebSocket 传输实例。
    pub fn new(remote: &McpRemoteTransport) -> Self {
        Self {
            url: remote.url.clone(),
            headers: build_headers(remote),
            send_tx: None,
            incoming_rx: None,
            ping_stop_tx: None,
            connected: false,
            io_task: None,
        }
    }
}

impl Drop for WsTransport {
    fn drop(&mut self) {
        if let Some(handle) = self.io_task.take() {
            handle.abort();
        }
    }
}

#[async_trait]
impl McpTransportConnection for WsTransport {
    async fn connect(&mut self) -> Result<(), String> {
        eprintln!("[mcp] 远程连接: WS {} [connecting]", self.url);

        // 构建 WebSocket 请求（含自定义 headers）
        let mut request = self
            .url
            .as_str()
            .into_client_request()
            .map_err(|e| format!("WS URL parse error: {e}"))?;

        for (key, value) in &self.headers {
            if let (Ok(name), Ok(val)) = (
                tungstenite::http::HeaderName::from_bytes(key.as_bytes()),
                tungstenite::http::HeaderValue::from_str(value),
            ) {
                request.headers_mut().insert(name, val);
            }
        }

        let (ws_stream, _response): (WsStream, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| format!("WebSocket connect failed: {e}"))?;

        let (msg_tx, msg_rx) = mpsc::channel(64);
        let (io_send_tx, mut io_send_rx) = mpsc::channel::<String>(64);
        let (io_ping_stop_tx, mut io_ping_stop_rx) = mpsc::channel::<()>(1);

        let io_url = self.url.clone();

        // 统一 I/O 任务
        let handle = tokio::spawn(async move {
            let (mut ws_write, mut ws_read) = ws_stream.split();
            let mut ping_interval = time::interval(Duration::from_secs(30));

            loop {
                tokio::select! {
                    msg = io_send_rx.recv() => {
                        match msg {
                            Some(text) => {
                                if ws_write.send(Message::Text(text)).await.is_err() {
                                    break;
                                }
                            }
                            None => {
                                let _ = ws_write.send(Message::Close(None)).await;
                                break;
                            }
                        }
                    }
                    msg = ws_read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                match serde_json::from_str::<JsonValue>(&text) {
                                    Ok(value) => {
                                        if msg_tx.send(Ok(value)).await.is_err() {
                                            break;
                                        }
                                    }
                                    Err(e) => {
                                        let _ = msg_tx.send(Err(format!("WS JSON parse: {e}"))).await;
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) => {
                                let _ = msg_tx.send(Err("WebSocket closed by server".into())).await;
                                break;
                            }
                            Some(Ok(Message::Ping(_))) => {}
                            Some(Ok(_)) => {}
                            Some(Err(e)) => {
                                let _ = msg_tx.send(Err(format!("WS read error: {e}"))).await;
                                break;
                            }
                            None => {
                                let _ = msg_tx.send(Err("WS stream ended".into())).await;
                                break;
                            }
                        }
                    }
                    _ = ping_interval.tick() => {
                        if ws_write.send(Message::Ping(vec![])).await.is_err() {
                            eprintln!("[mcp] 远程连接: WS {io_url} [ping failed]");
                            break;
                        }
                    }
                    _ = io_ping_stop_rx.recv() => {
                        let _ = ws_write.send(Message::Close(None)).await;
                        break;
                    }
                }
            }
            eprintln!("[mcp] 远程连接: WS {io_url} [io task ended]");
        });

        self.send_tx = Some(io_send_tx);
        self.incoming_rx = Some(msg_rx);
        self.ping_stop_tx = Some(io_ping_stop_tx);
        self.io_task = Some(handle);
        self.connected = true;

        eprintln!("[mcp] 远程连接: WS {} [connected]", self.url);
        Ok(())
    }

    async fn send(&mut self, request: JsonValue) -> Result<(), String> {
        let tx = self
            .send_tx
            .as_ref()
            .ok_or("WS not connected: no send channel")?;

        let text =
            serde_json::to_string(&request).map_err(|e| format!("WS serialize error: {e}"))?;

        tx.send(text)
            .await
            .map_err(|e| format!("WS send error: {e}"))
    }

    async fn receive(&mut self) -> Result<JsonValue, String> {
        let rx = self
            .incoming_rx
            .as_mut()
            .ok_or("WS not connected: no incoming channel")?;

        match rx.recv().await {
            Some(result) => result,
            None => {
                self.connected = false;
                Err("WebSocket stream closed".to_string())
            }
        }
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.connected = false;
        self.incoming_rx = None;
        self.send_tx = None;
        if let Some(stop_tx) = self.ping_stop_tx.take() {
            let _ = stop_tx.send(()).await;
        }
        eprintln!("[mcp] 远程连接: WS [disconnected]");
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

// ===========================================================================
// 工厂方法
// ===========================================================================

/// 根据 `McpClientTransport` 创建对应的远程传输实例。
pub fn create_remote_transport(
    transport: &McpClientTransport,
) -> Result<Box<dyn McpTransportConnection>, String> {
    match transport {
        McpClientTransport::Sse(remote) => Ok(Box::new(SseTransport::new(remote))),
        McpClientTransport::Http(remote) => Ok(Box::new(HttpTransport::new(remote))),
        McpClientTransport::WebSocket(remote) => Ok(Box::new(WsTransport::new(remote))),
        other => Err(format!(
            "Not a remote transport: {}",
            transport_variant_name(other)
        )),
    }
}

// ===========================================================================
// 单元测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp_client::McpClientAuth;

    #[test]
    fn build_jsonrpc_request_creates_valid_structure() {
        let req =
            build_jsonrpc_request("initialize", Some(serde_json::json!({"capabilities": {}})));
        assert_eq!(req["jsonrpc"], "2.0");
        assert_eq!(req["method"], "initialize");
        assert!(req.get("id").is_some());
        assert!(req.get("params").is_some());
    }

    #[test]
    fn build_jsonrpc_request_without_params() {
        let req = build_jsonrpc_request("notifications/initialized", None);
        assert_eq!(req["jsonrpc"], "2.0");
        assert_eq!(req["method"], "notifications/initialized");
        assert!(req.get("params").is_none());
    }

    #[test]
    fn is_remote_transport_returns_true_for_remote_types() {
        let remote = McpRemoteTransport {
            url: "https://example.com".to_string(),
            headers: BTreeMap::new(),
            headers_helper: None,
            auth: McpClientAuth::None,
        };
        assert!(is_remote_transport(&McpClientTransport::Sse(
            remote.clone()
        )));
        assert!(is_remote_transport(&McpClientTransport::Http(
            remote.clone()
        )));
        assert!(is_remote_transport(&McpClientTransport::WebSocket(remote)));

        let stdio = McpClientTransport::Stdio(crate::mcp_client::McpStdioTransport {
            command: "echo".to_string(),
            args: vec![],
            env: BTreeMap::new(),
        });
        assert!(!is_remote_transport(&stdio));
    }

    #[test]
    fn create_remote_transport_fails_for_non_remote() {
        let stdio = McpClientTransport::Stdio(crate::mcp_client::McpStdioTransport {
            command: "echo".to_string(),
            args: vec![],
            env: BTreeMap::new(),
        });
        let result = create_remote_transport(&stdio);
        assert!(result.is_err());
    }

    #[test]
    fn next_request_id_increments() {
        let id1 = next_request_id();
        let id2 = next_request_id();
        assert!(id2 > id1);
    }

    #[test]
    fn transport_variant_name_works() {
        let remote = McpRemoteTransport {
            url: "https://example.com".to_string(),
            headers: BTreeMap::new(),
            headers_helper: None,
            auth: McpClientAuth::None,
        };
        assert_eq!(
            transport_variant_name(&McpClientTransport::Sse(remote.clone())),
            "SSE"
        );
        assert_eq!(
            transport_variant_name(&McpClientTransport::Http(remote.clone())),
            "HTTP"
        );
        assert_eq!(
            transport_variant_name(&McpClientTransport::WebSocket(remote)),
            "WebSocket"
        );
    }
}
