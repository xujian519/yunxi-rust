//! 从 `.yunxi/commands/*.md` 与 `~/.yunxi/commands/*.md` 加载自定义斜杠命令。

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

/// 用户定义的斜杠命令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomSlashCommand {
    pub name: String,
    pub description: String,
    pub template: String,
}

static CUSTOM_REGISTRY: OnceLock<HashMap<String, CustomSlashCommand>> = OnceLock::new();

fn command_directories() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd.join(".yunxi").join("commands"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".yunxi").join("commands"));
    }
    dirs
}

/// 扫描并加载全部自定义命令（项目目录优先，用户目录覆盖同名）。
pub fn load_custom_slash_commands() -> HashMap<String, CustomSlashCommand> {
    let mut merged = HashMap::new();
    for dir in command_directories() {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(cmd) = parse_command_markdown(name, &content) {
                    merged.insert(cmd.name.clone(), cmd);
                }
            }
        }
    }
    merged
}

fn parse_command_markdown(name: &str, content: &str) -> Option<CustomSlashCommand> {
    let (frontmatter, body) = split_frontmatter(content);
    let description = frontmatter
        .get("description")
        .cloned()
        .unwrap_or_else(|| format!("自定义命令 /{name}"));
    let template = body.trim();
    if template.is_empty() {
        return None;
    }
    Some(CustomSlashCommand {
        name: name.to_string(),
        description,
        template: template.to_string(),
    })
}

fn split_frontmatter(content: &str) -> (HashMap<String, String>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (HashMap::new(), content);
    }
    let rest = trimmed.trim_start_matches("---").trim_start();
    let Some(end) = rest.find("\n---") else {
        return (HashMap::new(), content);
    };
    let header = &rest[..end];
    let body = rest[end + 4..].trim_start();
    let mut map = HashMap::new();
    for line in header.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        map.insert(key.trim().to_string(), value.trim().to_string());
    }
    (map, body)
}

/// 已加载的自定义命令表（进程内缓存，启动后刷新需重启 CLI）。
#[must_use]
pub fn custom_slash_registry() -> &'static HashMap<String, CustomSlashCommand> {
    CUSTOM_REGISTRY.get_or_init(load_custom_slash_commands)
}

/// 将 `$ARGUMENTS` 替换为用户输入的剩余参数。
#[must_use]
pub fn resolve_custom_prompt(name: &str, arguments: Option<&str>) -> Option<String> {
    let cmd = custom_slash_registry().get(name)?;
    let args = arguments.unwrap_or("");
    Some(
        cmd.template
            .replace("$ARGUMENTS", args)
            .replace("$arguments", args),
    )
}

/// 自定义命令帮助段落。
#[must_use]
pub fn render_custom_slash_help() -> String {
    let registry = custom_slash_registry();
    if registry.is_empty() {
        return String::new();
    }
    let mut lines = vec!["自定义命令（.yunxi/commands 或 ~/.yunxi/commands）".to_string()];
    let mut names: Vec<_> = registry.keys().collect();
    names.sort();
    for name in names {
        let cmd = &registry[name];
        lines.push(format!("  /{name:<18} {}", cmd.description));
    }
    lines.join("\n")
}

/// 自定义命令名称列表（用于补全）。
#[must_use]
pub fn custom_slash_command_names() -> Vec<String> {
    let mut names: Vec<String> = custom_slash_registry().keys().cloned().collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_and_body() {
        let md = "---\ndescription: Run tests\n---\nRun tests with focus on $ARGUMENTS";
        let cmd = parse_command_markdown("test", md).expect("command");
        assert_eq!(cmd.description, "Run tests");
        assert!(cmd.template.contains("$ARGUMENTS"));
    }

    #[test]
    fn template_replaces_arguments_placeholder() {
        let cmd = CustomSlashCommand {
            name: "demo".to_string(),
            description: "d".to_string(),
            template: "Do $ARGUMENTS now".to_string(),
        };
        let prompt = cmd
            .template
            .replace("$ARGUMENTS", "unit tests")
            .replace("$arguments", "unit tests");
        assert_eq!(prompt, "Do unit tests now");
    }
}
