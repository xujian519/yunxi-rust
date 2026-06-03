//! 设置读写 IPC。

use std::fs;
use std::path::PathBuf;

use runtime::{pricing_for_model, ConfigLoader, ModelPricing};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as SerdeValue};
use yunxi_cli::llm_auth::{api_key_env_for_model, llm_auth_configured as is_llm_auth_configured};
use yunxi_cli::model_routing::default_model_from_config;

#[derive(Debug, Serialize, Deserialize)]
pub struct YunxiSettings {
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_router: Option<SerdeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<SerdeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<SerdeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<SerdeValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_keys: Option<SerdeValue>,
    /// 桌面端 UI 偏好（general / appearance / editor / cost 等）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desktop: Option<SerdeValue>,
}

fn default_model() -> String {
    "deepseek-v4-pro".to_string()
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub estimated_cost: f64,
}

fn workspace_root() -> Result<PathBuf, String> {
    yunxi_cli::session_mgr::workspace_root().map_err(|e| e.to_string())
}

fn project_settings_path() -> Result<PathBuf, String> {
    Ok(workspace_root()?.join(".yunxi").join("settings.json"))
}

fn load_merged_config() -> Result<runtime::RuntimeConfig, String> {
    let root = workspace_root()?;
    ConfigLoader::default_for(root)
        .load()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings() -> Result<YunxiSettings, String> {
    let config = load_merged_config()?;
    let rendered = config.as_json().render();
    let root: SerdeValue =
        serde_json::from_str(&rendered).map_err(|e| format!("settings parse error: {e}"))?;
    let obj = root.as_object().ok_or("settings must be object")?;

    Ok(YunxiSettings {
        model: obj
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("deepseek-v4-pro")
            .to_string(),
        model_router: obj.get("modelRouter").cloned(),
        permissions: obj.get("permissions").cloned(),
        hooks: obj.get("hooks").cloned(),
        appearance: obj.get("appearance").cloned(),
        api_keys: obj.get("apiKeys").cloned(),
        desktop: obj.get("desktop").cloned(),
    })
}

#[tauri::command]
pub fn save_settings(settings: YunxiSettings) -> Result<(), String> {
    let path = project_settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut existing: SerdeValue = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    let obj = existing
        .as_object_mut()
        .ok_or("settings root must be object")?;

    obj.insert("model".to_string(), json!(settings.model));
    if let Some(router) = settings.model_router {
        obj.insert("modelRouter".to_string(), router);
    }
    if let Some(perms) = settings.permissions {
        obj.insert("permissions".to_string(), perms);
    }
    if let Some(hooks) = settings.hooks {
        obj.insert("hooks".to_string(), hooks);
    }
    if let Some(appearance) = settings.appearance {
        obj.insert("appearance".to_string(), appearance);
    }
    if let Some(keys) = settings.api_keys {
        obj.insert("apiKeys".to_string(), keys);
    }
    if let Some(desktop) = settings.desktop {
        obj.insert("desktop".to_string(), desktop);
    }

    fs::write(
        &path,
        serde_json::to_string_pretty(obj).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_usage() -> Result<UsageSummary, String> {
    let mut input_tokens = 0u32;
    let mut output_tokens = 0u32;

    if let Ok(sessions) = yunxi_cli::session_mgr::list_managed_sessions() {
        for summary in sessions {
            if let Ok(session) = runtime::Session::load_from_path(&summary.path) {
                for message in &session.messages {
                    if let Some(usage) = &message.usage {
                        input_tokens = input_tokens.saturating_add(usage.input_tokens);
                        output_tokens = output_tokens.saturating_add(usage.output_tokens);
                    }
                }
            }
        }
    }

    let model = default_model_from_config();
    let pricing = pricing_for_model(&model).unwrap_or(ModelPricing::default_sonnet_tier());
    let usage = runtime::TokenUsage {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
    };
    let cost = usage.estimate_cost_usd_with_pricing(pricing);

    Ok(UsageSummary {
        input_tokens,
        output_tokens,
        estimated_cost: cost.total_cost_usd(),
    })
}

/// 检查指定模型（或默认模型）是否已配置 LLM API Key
#[tauri::command]
pub fn llm_auth_configured(model: Option<String>) -> Result<bool, String> {
    let m = model.unwrap_or_else(default_model_from_config);
    Ok(is_llm_auth_configured(&m))
}

/// 为模型写入 API Key 到项目 settings.json 的 `env` 段
#[tauri::command]
pub fn save_llm_api_key(model: String, api_key: String) -> Result<YunxiSettings, String> {
    let trimmed = api_key.trim();
    if trimmed.len() < 8 {
        return Err("API Key 过短，请检查后重试".to_string());
    }
    let env_var = api_key_env_for_model(model.trim());
    let path = project_settings_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut existing: SerdeValue = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    let obj = existing
        .as_object_mut()
        .ok_or("settings root must be object")?;

    let env = obj.entry("env").or_insert_with(|| json!({}));
    if let Some(env_obj) = env.as_object_mut() {
        env_obj.insert(env_var.clone(), json!(trimmed));
    }
    obj.insert("model".to_string(), json!(model.trim()));

    fs::write(
        &path,
        serde_json::to_string_pretty(obj).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    get_settings()
}
