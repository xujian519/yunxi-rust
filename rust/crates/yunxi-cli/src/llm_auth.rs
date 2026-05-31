//! 按模型解析 LLM 鉴权环境变量，供 TUI / CLI 预检与错误提示。

use std::path::PathBuf;

use llm::config::LlmConfig;
use llm::provider::Provider;

/// 返回该模型对应的 API Key 环境变量名。
#[must_use]
pub fn api_key_env_for_model(model: &str) -> String {
    let config = LlmConfig::load().unwrap_or_default();
    let resolved = config.resolve_alias(model);
    Provider::detect(&resolved, &config.providers)
        .api_key_env()
        .to_string()
}

/// 环境变量或运行时配置 `env` 段中是否已配置非空密钥。
#[must_use]
pub(crate) fn api_key_configured(env_var: &str) -> bool {
    if std::env::var(env_var)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .is_some()
    {
        return true;
    }
    let cwd = crate::session_mgr::workspace_root()
        .or_else(|_| std::env::current_dir())
        .unwrap_or_else(|_| PathBuf::from("."));
    let Ok(cfg) = runtime::ConfigLoader::default_for(cwd).load() else {
        return false;
    };
    cfg.get("env")
        .and_then(|v| v.as_object())
        .and_then(|obj| obj.get(env_var))
        .and_then(|v| v.as_str())
        .is_some_and(|v| !v.trim().is_empty())
}

/// 模型鉴权配置提示（用于错误页脚）。
#[must_use]
pub(crate) fn auth_hint_for_model(model: &str) -> String {
    let env_var = api_key_env_for_model(model);
    format!(
        "当前模型 {model} 需要 {env_var}；请在 export 或配置 env 段中设置有效密钥，或用 /model 切换模型。"
    )
}

/// 格式化为 TUI 可显示的 LLM 失败信息。
#[must_use]
pub(crate) fn format_llm_error(model: &str, error: &dyn std::fmt::Display) -> String {
    format!(
        "\x1b[31m请求失败:\x1b[0m\n{error}\n\n\x1b[2m提示: {}\x1b[0m",
        auth_hint_for_model(model)
    )
}

/// 当前模型是否已配置可用 API Key（环境变量或 settings `env` 段）。
#[must_use]
pub fn llm_auth_configured(model: &str) -> bool {
    missing_api_key_message(model).is_none()
}

/// 启动轮次前预检；缺失密钥时返回用户可读错误。
#[must_use]
pub(crate) fn missing_api_key_message(model: &str) -> Option<String> {
    let env_var = api_key_env_for_model(model);
    if api_key_configured(&env_var) {
        return None;
    }
    Some(format!(
        "\x1b[31m无法发起对话:\x1b[0m 未配置 {env_var}\n\n\x1b[2m{}\x1b[0m",
        auth_hint_for_model(model)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glm_model_uses_glm_key_env() {
        assert_eq!(api_key_env_for_model("glm-5.1"), "GLM_API_KEY");
    }

    #[test]
    fn deepseek_model_uses_deepseek_key_env() {
        assert_eq!(api_key_env_for_model("deepseek-v4-pro"), "DEEPSEEK_API_KEY");
    }
}
