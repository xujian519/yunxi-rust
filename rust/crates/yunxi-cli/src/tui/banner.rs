#![allow(dead_code)]

use crate::tui::frame::{truncate_ansi_to_width, visible_width};
use crate::tui::ui_palette::{chat_body, chat_meta, BRAND_MARK};

/// 生成启动会话信息区（单行紧凑，模型/权限见顶栏与底栏状态条）。
pub(crate) fn render_startup_welcome(
    model: &str,
    permission_mode: &str,
    cwd: &str,
    session_id: &str,
) -> String {
    let _ = (model, permission_mode);
    let cwd_display = truncate_display(cwd, 40);
    let session_short = truncate_display(session_id, 24);
    format!(
        "{} {}  ·  {} {}  ·  {}",
        chat_meta("目录"),
        chat_body(&cwd_display),
        chat_meta("会话"),
        chat_body(&session_short),
        chat_meta("/help 查看命令")
    )
}

/// 兼容旧调用：等同 `render_startup_welcome`。
pub(crate) fn render_banner(
    model: &str,
    permission_mode: &str,
    cwd: &str,
    session_id: &str,
) -> String {
    render_startup_welcome(model, permission_mode, cwd, session_id)
}

/// 渲染简短版本 banner（用于 --version）。
pub(crate) fn render_version_banner(version: &str) -> String {
    format!("云熙智能体 (YunXi Agent) v{version} {BRAND_MARK}")
}

fn truncate_display(text: &str, max_cols: usize) -> String {
    if usize::from(visible_width(text)) <= max_cols {
        text.to_string()
    } else {
        truncate_ansi_to_width(text, max_cols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_info_contains_session_fields() {
        let banner = render_startup_welcome("deepseek-v4-pro", "dontAsk", "/tmp", "sess-1");
        assert!(!banner.contains('\n'));
        assert!(banner.contains("/tmp"));
        assert!(banner.contains("sess-1"));
        assert!(banner.contains("/help"));
    }

    #[test]
    fn startup_info_has_no_logo_block() {
        let banner = render_startup_welcome("m", "read-only", "/x", "s");
        assert!(!banner.contains('│'));
        assert!(!banner.contains(BRAND_MARK));
    }

    #[test]
    fn version_banner_contains_version() {
        let banner = render_version_banner("0.1.0");
        assert!(banner.contains("0.1.0"));
        assert!(banner.contains("云熙智能体"));
        assert!(banner.contains(BRAND_MARK));
    }
}
