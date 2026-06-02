//! TUI / REPL 共用的斜杠命令与参数补全。

use std::path::Path;

use commands::{all_slash_command_names, slash_command_specs, SlashCommandSpec};

use crate::session_mgr::list_managed_sessions;

/// 单个补全项的丰富信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashCompletionItem {
    /// Unicode 图标（如 "❓"、"📊"）。
    pub icon: String,
    /// 显示文本（如 "/help"）。
    pub display: String,
    /// 整行替换文本（如 "/help" 或 "/model [模型]"）。
    pub replacement: String,
    /// 命令描述（如 "显示可用斜杠命令"）。
    pub description: String,
}

impl SlashCompletionItem {
    fn from_spec(spec: &SlashCommandSpec) -> Self {
        let display = format!("/{}", spec.name);
        let replacement = match spec.argument_hint {
            Some(hint) => format!("/{} {}", spec.name, hint),
            None => format!("/{}", spec.name),
        };
        Self {
            icon: spec.icon.to_string(),
            display,
            replacement,
            description: spec.summary.to_string(),
        }
    }

    fn from_name(name: &str) -> Self {
        Self {
            icon: "⌨".to_string(),
            display: format!("/{}", name),
            replacement: format!("/{} ", name),
            description: String::new(),
        }
    }
}

/// 返回补全候选列表。
pub(crate) fn slash_line_completions(line: &str, cursor_at_end: bool) -> Vec<SlashCompletionItem> {
    if !cursor_at_end || !line.starts_with('/') || line.contains('\n') {
        return Vec::new();
    }

    let Some(ctx) = parse_slash_line(line) else {
        return Vec::new();
    };

    if ctx.completing_command {
        let mut items: Vec<SlashCompletionItem> = slash_command_specs()
            .iter()
            .map(SlashCompletionItem::from_spec)
            .collect();
        let custom_items: Vec<SlashCompletionItem> = all_slash_command_names()
            .into_iter()
            .filter_map(|name| {
                if slash_command_specs().iter().any(|s| s.name == name) {
                    return None;
                }
                Some(SlashCompletionItem::from_name(&name))
            })
            .collect();
        items.extend(custom_items);
        return items
            .into_iter()
            .filter(|item| {
                item.display.starts_with(&ctx.command_token)
                    || ctx.command_token.starts_with(&item.display)
            })
            .collect();
    }

    argument_completions(&ctx, line)
}

#[derive(Debug)]
struct SlashParse {
    command_token: String,
    completing_command: bool,
    subcommand: Option<String>,
    partial: String,
}

fn parse_slash_line(line: &str) -> Option<SlashParse> {
    let body = line.trim_start();
    if !body.starts_with('/') {
        return None;
    }

    let rest = body.trim_start_matches('/');
    let first_space = rest.find(char::is_whitespace);
    let ends_with_space = body.ends_with(' ');

    match first_space {
        None => Some(SlashParse {
            command_token: format!("/{}", rest),
            completing_command: true,
            subcommand: None,
            partial: rest.to_string(),
        }),
        Some(idx) => {
            let cmd = rest[..idx].to_ascii_lowercase();
            let tail = rest[idx + 1..].trim_end();
            let (subcommand, partial) = if ends_with_space {
                (None, String::new())
            } else if let Some(last_space) = tail.rfind(char::is_whitespace) {
                (
                    Some(tail[..last_space].to_string()),
                    tail[last_space + 1..].to_string(),
                )
            } else {
                (None, tail.to_string())
            };

            Some(SlashParse {
                command_token: format!("/{}", cmd),
                completing_command: false,
                subcommand,
                partial,
            })
        }
    }
}

fn argument_completions(ctx: &SlashParse, line: &str) -> Vec<SlashCompletionItem> {
    let cmd = ctx.command_token.trim_start_matches('/').trim_start();

    let options: Vec<String> = match cmd {
        "model" => MODEL_CANDIDATES
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        "permissions" => PERMISSION_MODES
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        "config" => CONFIG_SECTIONS
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
        "session" => session_argument_candidates(ctx),
        "export" => export_path_candidates(&ctx.partial),
        _ => Vec::new(),
    };

    if options.is_empty() {
        return Vec::new();
    }

    let prefix = ctx.partial.to_ascii_lowercase();
    options
        .into_iter()
        .filter(|opt| prefix.is_empty() || opt.to_ascii_lowercase().starts_with(&prefix))
        .map(|opt| {
            let replacement = build_replacement_line(cmd, ctx, line, &opt);
            SlashCompletionItem {
                icon: "⌨".to_string(),
                display: opt.clone(),
                replacement,
                description: String::new(),
            }
        })
        .collect()
}

fn build_replacement_line(cmd: &str, ctx: &SlashParse, line: &str, choice: &str) -> String {
    let body = line.trim_start();
    let rest = body.trim_start_matches('/');
    let cmd_end = rest.find(char::is_whitespace).map_or(rest.len(), |i| i);
    let prefix = format!("/{}", &rest[..cmd_end]);

    match cmd {
        "session" if ctx.subcommand.is_none() => {
            if choice == "list" || choice == "switch" {
                format!("{} {}", prefix, choice)
            } else {
                format!("{} switch {}", prefix, choice)
            }
        }
        "session" => format!(
            "{} {} {}",
            prefix,
            ctx.subcommand.as_deref().unwrap_or("switch"),
            choice
        ),
        _ => format!("{} {}", prefix, choice),
    }
}

fn session_argument_candidates(ctx: &SlashParse) -> Vec<String> {
    if ctx.subcommand.is_none() {
        let mut out = vec!["list".to_string(), "switch".to_string()];
        if let Ok(sessions) = list_managed_sessions() {
            for session in sessions.into_iter().take(12) {
                out.push(session.id);
            }
        }
        return out;
    }

    if ctx.subcommand.as_deref() == Some("switch") {
        return list_managed_sessions()
            .map(|sessions| sessions.into_iter().map(|s| s.id).take(20).collect())
            .unwrap_or_default();
    }

    Vec::new()
}

fn export_path_candidates(partial: &str) -> Vec<String> {
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => return Vec::new(),
    };
    let partial_path = Path::new(partial);
    let (dir, file_prefix) = if partial.is_empty() {
        (cwd, "")
    } else if partial.ends_with('/') || partial.ends_with('\\') {
        (cwd.join(partial), "")
    } else if let Some(parent) = partial_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        (
            cwd.join(parent),
            partial_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(""),
        )
    } else {
        (cwd.clone(), partial)
    };

    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut names = Vec::new();
    for entry in read_dir.flatten().take(40) {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if file_prefix.is_empty() || name.starts_with(file_prefix) {
            let path = if partial.is_empty() || partial.ends_with('/') || partial.ends_with('\\') {
                format!("{}{}", partial, name)
            } else if partial_path.parent().is_some()
                && partial_path.parent() != Some(Path::new(""))
            {
                format!(
                    "{}/{}",
                    partial_path.parent().unwrap().to_string_lossy(),
                    name
                )
            } else {
                name
            };
            names.push(path);
        }
    }
    names.sort();
    names
}

pub(crate) const MODEL_CANDIDATES: &[&str] = &["auto", "deepseek-v4-flash", "deepseek-v4-pro"];

const PERMISSION_MODES: &[&str] = &["read-only", "workspace-write", "danger-full-access"];

const CONFIG_SECTIONS: &[&str] = &["env", "hooks", "model"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_command_prefix() {
        let items = slash_line_completions("/he", true);
        assert!(items.iter().any(|item| item.replacement == "/help"));
    }

    #[test]
    fn completes_model_arguments() {
        let items = slash_line_completions("/model deep", true);
        assert!(items.iter().any(|item| item.display.contains("deepseek")));
    }

    #[test]
    fn completes_permissions_arguments() {
        let items = slash_line_completions("/permissions work", true);
        assert!(items
            .iter()
            .any(|item| item.replacement.contains("workspace-write")));
    }

    #[test]
    fn ignores_non_slash() {
        assert!(slash_line_completions("hello", true).is_empty());
    }

    #[test]
    fn command_items_have_icons() {
        let items = slash_line_completions("/", true);
        let help = items.iter().find(|i| i.display == "/help");
        assert!(help.is_some());
        assert_eq!(help.unwrap().icon, "❓");
        assert!(!help.unwrap().description.is_empty());
    }
}
