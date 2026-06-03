//! 云熙智能体 - HTTP/WebSocket 服务器
//!
//! 提供 REST API 和 WebSocket 实时通信能力。

pub mod agent_bridge;
pub mod auth;
pub mod case_store;
pub mod permission;
pub mod routes;
pub mod server;
pub mod session_store;
pub mod settings_store;
pub mod ws_stream;

#[cfg(test)]
mod tests;

pub use server::{start, AppState, ServerConfig};
