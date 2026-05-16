#![allow(dead_code)]

/// 生成可定制的启动 banner。
pub(crate) fn render_banner(
    model: &str,
    permission_mode: &str,
    cwd: &str,
    session_id: &str,
) -> String {
    format!(
        "\x1b[38;5;213m    ✿\n\
   ✿ ✿ ✿\x1b[0m\n\n\
  \x1b[1m\x1b[38;5;183m云熙智能体\x1b[0m \x1b[2mYunXi Agent\x1b[0m \x1b[38;5;213m✿\x1b[0m\n\
  \x1b[2m专业专利智能体\x1b[0m\n\n\
  \x1b[2m模型\x1b[0m            {model}\n\
  \x1b[2m权限\x1b[0m            {permission_mode}\n\
  \x1b[2m工作目录\x1b[0m        {cwd}\n\
  \x1b[2m会话\x1b[0m            {session_id}\n\n\
  输入 \x1b[1m/help\x1b[0m 查看命令 · \x1b[2mShift+Enter\x1b[0m 换行",
    )
}

/// 渲染简短版本 banner（用于 --version）。
pub(crate) fn render_version_banner(version: &str) -> String {
    format!("云熙智能体 (YunXi Agent) v{version} ✿")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banner_contains_model_and_brand() {
        let banner = render_banner(
            "claude-opus-4-6",
            "danger-full-access",
            "/tmp/project",
            "abc123",
        );
        assert!(banner.contains("claude-opus-4-6"));
        assert!(banner.contains("danger-full-access"));
        assert!(banner.contains("云熙智能体"));
        assert!(banner.contains("YunXi Agent"));
        assert!(banner.contains("专业专利智能体"));
        assert!(banner.contains("✿"));
        assert!(banner.contains("/help"));
    }

    #[test]
    fn version_banner_contains_version() {
        let banner = render_version_banner("0.1.0");
        assert!(banner.contains("0.1.0"));
        assert!(banner.contains("云熙智能体"));
        assert!(banner.contains("✿"));
    }
}
