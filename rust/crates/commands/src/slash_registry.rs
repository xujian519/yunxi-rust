//! 斜杠命令注册表：内置清单 + 自定义命令，统一 help 与补全名称。

use crate::custom_slash::{custom_slash_command_names, render_custom_slash_help};
use crate::{render_slash_command_help, slash_command_specs};

/// 内置 + OpenCode 兼容别名的补充说明（仅 help 展示，解析在 `SlashCommand::parse`）。
pub const SLASH_ALIAS_HELP_LINES: &[&str] = &[
    "  /new                 新建会话（等同 /clear --confirm）",
    "  /models              显示当前模型（等同 /model）",
    "  /sessions            列出/切换会话（等同 /session）",
    "  /summarize           压缩会话（等同 /compact）",
    "  /connect             查看 API 与 OAuth 配置指引",
    "  /thinking            切换推理过程在对话区的显示",
];

/// 完整斜杠帮助（内置 + 别名 + 自定义）。
#[must_use]
pub fn render_full_slash_help() -> String {
    let mut out = render_slash_command_help();
    out.push_str("\n\n别名\n");
    for line in SLASH_ALIAS_HELP_LINES {
        out.push_str(line);
        out.push('\n');
    }
    let custom = render_custom_slash_help();
    if !custom.is_empty() {
        out.push('\n');
        out.push_str(&custom);
    }
    out
}

/// 用于 Tab 补全的全部命令名（不含 `/` 前缀）。
#[must_use]
pub fn all_slash_command_names() -> Vec<String> {
    let mut names: Vec<String> = slash_command_specs()
        .iter()
        .map(|spec| spec.name.to_string())
        .collect();
    names.extend([
        "new".to_string(),
        "models".to_string(),
        "sessions".to_string(),
        "continue".to_string(),
        "summarize".to_string(),
        "connect".to_string(),
        "thinking".to_string(),
    ]);
    names.extend(custom_slash_command_names());
    names.sort();
    names.dedup();
    names
}
