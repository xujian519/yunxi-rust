//! 进程级嵌入服务单例（按配置懒加载）

use std::sync::{Mutex, OnceLock};

use crate::config::{load_semantic_config, semantic_enabled};
use crate::service::{EmbeddingError, EmbeddingService};

static GLOBAL: OnceLock<Mutex<Option<EmbeddingService>>> = OnceLock::new();

fn slot() -> &'static Mutex<Option<EmbeddingService>> {
    GLOBAL.get_or_init(|| Mutex::new(None))
}

/// 若用户已开启语义能力且服务可用，返回共享嵌入实例
pub fn shared_optional() -> Option<EmbeddingService> {
    if !semantic_enabled() {
        return None;
    }
    let mut guard = slot().lock().ok()?;
    if guard.is_none() {
        *guard = EmbeddingService::load_from_config().ok();
    }
    guard.clone()
}

/// 强制加载（用于 CLI 索引等）；未开启时返回配置错误
pub fn shared_required() -> Result<EmbeddingService, EmbeddingError> {
    if !semantic_enabled() {
        return Err(EmbeddingError::Disabled(
            "语义嵌入未启用：请在 .yunxi/settings.json 中设置 semantic.enabled=true".into(),
        ));
    }
    let mut guard = slot()
        .lock()
        .map_err(|_| EmbeddingError::Disabled("embedding global lock poisoned".into()))?;
    if guard.is_none() {
        *guard = Some(EmbeddingService::load_from_config()?);
    }
    guard
        .clone()
        .ok_or_else(|| EmbeddingError::Disabled("embedding service unavailable".into()))
}

/// 配置变更后重新加载（测试/热更新）
pub fn reload() -> Result<(), EmbeddingError> {
    let mut guard = slot()
        .lock()
        .map_err(|_| EmbeddingError::Disabled("embedding global lock poisoned".into()))?;
    *guard = if semantic_enabled() {
        Some(EmbeddingService::load_from_config()?)
    } else {
        None
    };
    Ok(())
}

/// 当前配置摘要（供状态输出）
#[must_use]
pub fn status_json() -> serde_json::Value {
    let cfg = load_semantic_config();
    let available = semantic_enabled() && EmbeddingService::load_from_config().is_ok();
    serde_json::json!({
        "enabled": cfg.enabled,
        "backend": cfg.backend,
        "httpBaseUrl": cfg.http.base_url,
        "available": available,
        "knowledgeSearchMode": cfg.defaults.knowledge_search_mode,
    })
}
