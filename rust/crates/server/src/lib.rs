//! 云熙智能体 - HTTP/WebSocket 服务器
//!
//! 提供 REST API 和 WebSocket 实时通信能力。

pub mod auth;
pub mod routes;
pub mod server;

#[cfg(test)]
mod tests;

pub use server::{start, AppState, ServerConfig};
