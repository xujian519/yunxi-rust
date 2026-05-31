//! 桌面端数据目录与工作区初始化。

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// 初始化桌面环境：配置目录、工作区 cwd、子目录。
pub fn init_desktop_environment() -> Result<(), String> {
    let yunxi_home = yunxi_home_dir()?;
    fs::create_dir_all(&yunxi_home).map_err(|e| e.to_string())?;
    fs::create_dir_all(yunxi_home.join("sessions")).map_err(|e| e.to_string())?;
    fs::create_dir_all(yunxi_home.join("cases")).map_err(|e| e.to_string())?;

    std::env::set_var("YUNXI_CONFIG_HOME", yunxi_home.to_string_lossy().as_ref());

    let workspace = resolve_workspace_root(&yunxi_home)?;
    fs::create_dir_all(&workspace).map_err(|e| e.to_string())?;
    fs::create_dir_all(workspace.join(".yunxi")).map_err(|e| e.to_string())?;
    fs::create_dir_all(workspace.join(".yunxi").join("sessions")).map_err(|e| e.to_string())?;

    std::env::set_var("YUNXI_WORKSPACE", workspace.to_string_lossy().as_ref());

    // 切勿修改进程 cwd —— Tauri 按 crate 目录解析 frontendDist，chdir 会导致白屏
    Ok(())
}

pub fn yunxi_home_dir() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".yunxi"))
        .ok_or_else(|| "无法解析 HOME 目录".to_string())
}

pub fn resolve_workspace_root(yunxi_home: &Path) -> Result<PathBuf, String> {
    if let Ok(from_env) = std::env::var("YUNXI_WORKSPACE") {
        let path = PathBuf::from(from_env);
        if path.is_dir() {
            return Ok(path);
        }
    }

    let desktop_cfg = yunxi_home.join("desktop.json");
    if desktop_cfg.is_file() {
        if let Ok(text) = fs::read_to_string(&desktop_cfg) {
            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                if let Some(root) = value.get("workspaceRoot").and_then(Value::as_str) {
                    let path = PathBuf::from(root);
                    if path.is_dir() {
                        return Ok(path);
                    }
                }
            }
        }
    }

    let candidates = [
        PathBuf::from("/Users/xujian/projects/YunXi"),
        yunxi_home.join("workspace"),
    ];
    for path in candidates {
        if path.is_dir() {
            return Ok(path);
        }
    }

    Ok(yunxi_home.join("workspace"))
}

pub fn user_settings_path() -> Result<PathBuf, String> {
    Ok(yunxi_home_dir()?.join("settings.json"))
}

pub fn cases_dir() -> Result<PathBuf, String> {
    Ok(yunxi_home_dir()?.join("cases"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yunxi_home_ends_with_yunxi() {
        if std::env::var_os("HOME").is_some() {
            let home = yunxi_home_dir().expect("home");
            assert!(home.ends_with(".yunxi"));
        }
    }
}
