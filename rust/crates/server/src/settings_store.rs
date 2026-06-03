//! 项目设置读写（与桌面端 `.yunxi/settings.json` 对齐）。

use std::fs;
use std::path::PathBuf;

use runtime::{ConfigLoader, PermissionMode, ResolvedPermissionMode, RuntimeConfig};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::agent_bridge;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YunxiSettings {
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_router: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appearance: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_keys: Option<JsonValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desktop: Option<JsonValue>,
}

fn default_model() -> String {
    "deepseek-v4-pro".to_string()
}

pub fn settings_path() -> PathBuf {
    agent_bridge::workspace_root()
        .join(".yunxi")
        .join("settings.json")
}

pub fn load_merged_config() -> Result<RuntimeConfig, String> {
    ConfigLoader::default_for(agent_bridge::workspace_root())
        .load()
        .map_err(|e| e.to_string())
}

pub fn get_settings() -> Result<YunxiSettings, String> {
    let config = load_merged_config()?;
    let rendered = config.as_json().render();
    let root: JsonValue =
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

pub fn save_settings(settings: &YunxiSettings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut existing: JsonValue = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));

    let obj = existing
        .as_object_mut()
        .ok_or("settings root must be object")?;

    obj.insert("model".to_string(), json!(settings.model));
    if let Some(router) = &settings.model_router {
        obj.insert("modelRouter".to_string(), router.clone());
    }
    if let Some(perms) = &settings.permissions {
        obj.insert("permissions".to_string(), perms.clone());
    }
    if let Some(hooks) = &settings.hooks {
        obj.insert("hooks".to_string(), hooks.clone());
    }
    if let Some(appearance) = &settings.appearance {
        obj.insert("appearance".to_string(), appearance.clone());
    }
    if let Some(keys) = &settings.api_keys {
        obj.insert("apiKeys".to_string(), keys.clone());
    }
    if let Some(desktop) = &settings.desktop {
        obj.insert("desktop".to_string(), desktop.clone());
    }

    fs::write(
        &path,
        serde_json::to_string_pretty(obj).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// 从合并配置解析权限模式；HTTP Server 默认 `Prompt`（需用户确认工具）。
pub fn resolve_permission_mode() -> PermissionMode {
    match load_merged_config()
        .ok()
        .and_then(|cfg| cfg.permission_mode())
    {
        Some(ResolvedPermissionMode::ReadOnly) => PermissionMode::ReadOnly,
        Some(ResolvedPermissionMode::WorkspaceWrite) => PermissionMode::WorkspaceWrite,
        Some(ResolvedPermissionMode::DangerFullAccess) => PermissionMode::DangerFullAccess,
        None => PermissionMode::Prompt,
    }
}
