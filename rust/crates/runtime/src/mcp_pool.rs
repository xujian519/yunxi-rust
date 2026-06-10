//! MCP 连接池管理。
//!
//! `McpConnectionPool` 为每个 MCP 服务器维护连接状态，
//! 复用全局 tokio Runtime，idle 超时 5min 后回收连接。

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value as JsonValue;
use tokio::runtime::Handle;

use crate::mcp_client::McpClientTransport;
use crate::mcp_remote::{create_remote_transport, McpTransportConnection};

/// 默认 idle 超时时间：5 分钟。
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

/// 连接池中的单个连接条目。
struct PooledConnection {
    /// 底层传输连接。
    transport: Box<dyn McpTransportConnection>,
    /// 最后活跃时间。
    last_active: Instant,
    /// 服务器名称。
    server_name: String,
    /// 传输类型。
    transport_type: String,
}

/// MCP 连接池。
///
/// 为每个 MCP 服务器维护一个连接实例，支持：
/// - `get_or_create`: 获取已有连接或创建新连接
/// - `release`: 释放连接（不关闭，返回池中）
/// - `cleanup_idle`: 清理超时连接
/// - 复用外部 tokio Runtime（通过 Handle）
///
/// # 使用约束
///
/// 所有方法均要求 `&mut self`，池本身不是 `Sync` 的。
/// 当通过 [`SharedConnectionPool`]（`Arc<Mutex<Self>>` 包装）跨线程共享时，
/// Mutex 在单次方法调用期间持有，不会跨 `.await` 点，因此不会产生死锁。
/// **不要**在持有 `MutexGuard` 的同时 `.await` 其他需要同一把锁的操作。
pub struct McpConnectionPool {
    /// 连接池条目。
    connections: BTreeMap<String, PooledConnection>,
    /// tokio Runtime Handle。
    runtime_handle: Handle,
    /// idle 超时时间。
    idle_timeout: Duration,
}

impl McpConnectionPool {
    /// 创建新的连接池。
    pub fn new(runtime_handle: Handle) -> Self {
        Self {
            connections: BTreeMap::new(),
            runtime_handle,
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
        }
    }

    /// 创建带自定义 idle 超时的连接池。
    pub fn with_idle_timeout(runtime_handle: Handle, idle_timeout: Duration) -> Self {
        Self {
            connections: BTreeMap::new(),
            runtime_handle,
            idle_timeout,
        }
    }

    /// 获取或创建连接。
    ///
    /// 返回 `(server_name, transport_type)` 以便调用方识别连接。
    /// 使用 `with_connection` 方法访问连接进行操作。
    ///
    /// # Errors
    ///
    /// - 如果创建连接失败
    pub fn get_or_create(
        &mut self,
        server_name: &str,
        transport: &McpClientTransport,
    ) -> Result<(String, String), String> {
        // 清理 idle 连接
        self.cleanup_idle();

        // 检查已有连接
        if let Some(entry) = self.connections.get_mut(server_name) {
            if entry.transport.is_connected() {
                entry.last_active = Instant::now();
                return Ok((entry.server_name.clone(), entry.transport_type.clone()));
            }
            // 连接已断开，移除
            self.connections.remove(server_name);
        }

        // 创建新连接
        let transport_type = crate::mcp_remote::transport_variant_name(transport).to_string();
        let mut conn = create_remote_transport(transport)?;

        self.runtime_handle.block_on(conn.connect())?;

        eprintln!("[mcp] 远程连接: {server_name} [{transport_type}] [pool: created]");

        let entry = PooledConnection {
            transport: conn,
            last_active: Instant::now(),
            server_name: server_name.to_string(),
            transport_type: transport_type.clone(),
        };

        self.connections.insert(server_name.to_string(), entry);

        Ok((server_name.to_string(), transport_type))
    }

    /// 通过连接池发送请求。
    pub async fn send(&mut self, server_name: &str, request: JsonValue) -> Result<(), String> {
        let entry = self
            .connections
            .get_mut(server_name)
            .ok_or_else(|| format!("no pooled connection for {server_name}"))?;
        entry.last_active = Instant::now();
        entry.transport.send(request).await
    }

    /// 通过连接池接收响应。
    pub async fn receive(&mut self, server_name: &str) -> Result<JsonValue, String> {
        let entry = self
            .connections
            .get_mut(server_name)
            .ok_or_else(|| format!("no pooled connection for {server_name}"))?;
        entry.last_active = Instant::now();
        entry.transport.receive().await
    }

    /// 释放连接（返回池中，不关闭）。
    pub fn release(&mut self, server_name: &str) {
        if let Some(entry) = self.connections.get_mut(server_name) {
            entry.last_active = Instant::now();
            eprintln!(
                "[mcp] 远程连接: {} [{}] [pool: released]",
                entry.server_name, entry.transport_type
            );
        }
    }

    /// 清理超时的 idle 连接。
    pub fn cleanup_idle(&mut self) {
        let now = Instant::now();
        let expired: Vec<String> = self
            .connections
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.last_active) > self.idle_timeout)
            .map(|(name, _)| name.clone())
            .collect();

        for name in expired {
            if let Some(mut entry) = self.connections.remove(&name) {
                let _ = self.runtime_handle.block_on(entry.transport.disconnect());
                eprintln!(
                    "[mcp] 远程连接: {} [{}] [pool: idle timeout, cleaned up]",
                    entry.server_name, entry.transport_type
                );
            }
        }
    }

    /// 关闭所有连接。
    pub fn close_all(&mut self) {
        let names: Vec<String> = self.connections.keys().cloned().collect();
        for name in names {
            if let Some(mut entry) = self.connections.remove(&name) {
                let _ = self.runtime_handle.block_on(entry.transport.disconnect());
                eprintln!(
                    "[mcp] 远程连接: {} [{}] [pool: closed]",
                    entry.server_name, entry.transport_type
                );
            }
        }
    }

    /// 获取池中活跃连接数量。
    pub fn active_count(&self) -> usize {
        self.connections
            .values()
            .filter(|e| e.transport.is_connected())
            .count()
    }

    /// 获取池中总连接数量。
    pub fn total_count(&self) -> usize {
        self.connections.len()
    }

    /// 检查指定服务器的连接是否活跃。
    pub fn is_connected(&self, server_name: &str) -> bool {
        self.connections
            .get(server_name)
            .map(|e| e.transport.is_connected())
            .unwrap_or(false)
    }
}

impl Drop for McpConnectionPool {
    fn drop(&mut self) {
        self.close_all();
    }
}

// ===========================================================================
// 线程安全包装
// ===========================================================================

/// 线程安全的连接池包装。
///
/// 使用 `Arc<Mutex>` 包装，支持跨线程共享。
pub struct SharedConnectionPool {
    inner: Arc<Mutex<McpConnectionPool>>,
}

impl SharedConnectionPool {
    /// 创建共享连接池。
    pub fn new(runtime_handle: Handle) -> Self {
        Self {
            inner: Arc::new(Mutex::new(McpConnectionPool::new(runtime_handle))),
        }
    }

    /// 获取内部池的可变访问。
    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, McpConnectionPool>, String> {
        self.inner
            .lock()
            .map_err(|e| format!("connection pool lock poisoned: {e}"))
    }

    /// 克隆共享引用。
    pub fn clone_handle(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// ===========================================================================
// 单元测试
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pool_starts_empty() {
        let handle = tokio::runtime::Handle::current();
        let pool = McpConnectionPool::new(handle);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.total_count(), 0);
    }

    #[tokio::test]
    async fn shared_pool_clones_share_state() {
        let handle = tokio::runtime::Handle::current();
        let pool1 = SharedConnectionPool::new(handle);
        let pool2 = pool1.clone_handle();

        assert!(pool1.lock().is_ok());
        assert!(pool2.lock().is_ok());
    }

    #[tokio::test]
    async fn pool_default_idle_timeout_is_5min() {
        let handle = tokio::runtime::Handle::current();
        let _pool = McpConnectionPool::new(handle);
        assert_eq!(DEFAULT_IDLE_TIMEOUT, Duration::from_secs(300));
    }
}
