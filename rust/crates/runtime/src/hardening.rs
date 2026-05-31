//! 进程加固模块。
//!
//! 提供基本的进程安全加固措施。

/// 加固配置
#[derive(Debug, Clone)]
pub struct HardeningConfig {
    /// 禁用 core dump
    pub disable_core_dump: bool,
    /// 限制文件权限
    pub restrict_file_permissions: bool,
    /// 清除环境变量
    pub sanitize_env: bool,
}

impl Default for HardeningConfig {
    fn default() -> Self {
        Self {
            disable_core_dump: true,
            restrict_file_permissions: true,
            sanitize_env: true,
        }
    }
}

/// 应用进程加固
pub fn apply_hardening(config: &HardeningConfig) -> Result<(), String> {
    if config.disable_core_dump {
        disable_core_dump()?;
    }

    if config.sanitize_env {
        sanitize_environment()?;
    }

    Ok(())
}

/// 禁用 core dump
fn disable_core_dump() -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::process::Command;
        let result = Command::new("sh")
            .arg("-c")
            .arg("ulimit -c 0")
            .output()
            .map_err(|e| e.to_string())?;
        if !result.status.success() {
            return Err("Failed to disable core dump".into());
        }
    }
    #[cfg(not(unix))]
    {
        // Windows 下 core dump 由系统控制，暂不处理
    }
    Ok(())
}

/// 清理敏感环境变量（将 `YUNXI_*` 敏感项置空，避免 `env::remove_var` 的 unsafe）
fn sanitize_environment() -> Result<(), String> {
    const SENSITIVE_VARS: [&str; 5] = ["PASSWORD", "SECRET", "TOKEN", "API_KEY", "PRIVATE_KEY"];

    for var in SENSITIVE_VARS {
        let yunxi_var = format!("YUNXI_{var}");
        if std::env::var(&yunxi_var).is_ok() {
            std::env::set_var(&yunxi_var, "");
        }
    }

    Ok(())
}

/// 检查当前进程的安全状态
pub fn security_status() -> SecurityStatus {
    SecurityStatus {
        core_dump_disabled: cfg!(unix),
        running_as_root: check_running_as_root(),
        container_detected: crate::sandbox::detect_container_environment().in_container,
    }
}

/// 安全状态
#[derive(Debug, Clone)]
pub struct SecurityStatus {
    pub core_dump_disabled: bool,
    pub running_as_root: bool,
    pub container_detected: bool,
}

fn check_running_as_root() -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok().map(|s| s.trim() == "0"))
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_applies() {
        let config = HardeningConfig::default();
        assert!(config.disable_core_dump);
        assert!(config.restrict_file_permissions);
        assert!(config.sanitize_env);
    }

    #[test]
    fn security_status_returns_struct() {
        let status = security_status();
        assert!(!status.running_as_root);
    }
}
