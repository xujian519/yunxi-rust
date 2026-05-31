//! 认证中间件

use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub api_keys: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_keys: Vec::new(),
            enabled: false,
        }
    }
}

/// 验证请求认证
pub fn verify_auth(headers: &HeaderMap, config: &AuthConfig) -> Result<(), String> {
    if !config.enabled {
        return Ok(());
    }

    if config.api_keys.is_empty() {
        return Ok(());
    }

    let auth_header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("ApiKey "));

    let token = token.ok_or("缺少认证令牌")?;

    if config.api_keys.iter().any(|k| k == token) {
        Ok(())
    } else {
        Err("无效的认证令牌".into())
    }
}
