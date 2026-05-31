//! 智能模型路由：将 `auto` 解析为 deepseek-v4-pro / deepseek-v4-flash。

use std::path::PathBuf;

use model_router::{ModelSelector, RouterConfig, TaskContext, UserInput};
use runtime::ConfigLoader;
use serde_json::Value;

/// 从合并后的运行时配置读取 modelRouter 段。
#[must_use]
pub fn load_router_config() -> RouterConfig {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let Ok(runtime_config) = ConfigLoader::default_for(cwd).load() else {
        return RouterConfig::default();
    };
    let Ok(parsed) = serde_json::from_str::<Value>(&runtime_config.as_json().render()) else {
        return RouterConfig::default();
    };
    parsed
        .get("modelRouter")
        .and_then(parse_router_value)
        .unwrap_or_default()
}

fn parse_router_value(value: &Value) -> Option<RouterConfig> {
    let obj = value.as_object()?;
    Some(RouterConfig {
        enabled: obj.get("enabled")?.as_bool()?,
        threshold: obj
            .get("threshold")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8),
        fallback_model: obj
            .get("fallbackModel")
            .and_then(|v| v.as_str())
            .unwrap_or("deepseek-v4-pro")
            .to_string(),
        logging: None,
    })
}

/// 为单次请求解析实际模型名；非 `auto` 时原样返回。
#[must_use]
pub fn select_model_for_request(
    configured_model: &str,
    user_input: &str,
    history_rounds: usize,
    files_involved: usize,
    router_config: &RouterConfig,
) -> String {
    if configured_model != "auto" {
        return configured_model.to_string();
    }

    if !router_config.enabled {
        return router_config.fallback_model.clone();
    }

    let ctx = TaskContext::new(UserInput::new(user_input))
        .with_history(history_rounds)
        .with_files(files_involved);

    let selector = ModelSelector::new();
    selector.select_model_safe(&ctx).model
}

/// 读取项目/用户配置中的默认模型（如 `"auto"`）。
#[must_use]
pub fn default_model_from_config() -> String {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    ConfigLoader::default_for(cwd)
        .load()
        .ok()
        .and_then(|cfg| cfg.model().map(str::to_string))
        .unwrap_or_else(|| "deepseek-v4-pro".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_selects_concrete_model() {
        let cfg = RouterConfig::default();
        let model = select_model_for_request("auto", "你好", 0, 0, &cfg);
        assert!(
            model == "deepseek-v4-pro" || model == "deepseek-v4-flash",
            "unexpected model: {model}"
        );
    }

    #[test]
    fn explicit_model_passthrough() {
        let cfg = RouterConfig::default();
        assert_eq!(
            select_model_for_request("deepseek-v4-pro", "分析专利", 0, 0, &cfg),
            "deepseek-v4-pro"
        );
    }
}
