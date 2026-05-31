//! `yunxi config` — 配置引导与状态查看

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use runtime::{ConfigLoader, ConfigSource};
use serde_json::{json, Value};

const USER_LOCAL_TEMPLATE: &str = include_str!("../../../../.yunxi/settings.semantic.example.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigInitScope {
    User,
    Project,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigSubAction {
    Guide,
    Init {
        scope: ConfigInitScope,
        interactive: bool,
    },
    Show {
        section: Option<String>,
    },
}

pub(crate) fn parse_config_args(args: &[String]) -> Result<ConfigSubAction, String> {
    let mut scope = ConfigInitScope::User;
    let mut interactive = false;
    let sub = args.first().map(String::as_str);
    if sub == Some("init") {
        let mut index = 1usize;
        while index < args.len() {
            match args[index].as_str() {
                "--project" => {
                    scope = ConfigInitScope::Project;
                    index += 1;
                }
                "--user" => {
                    scope = ConfigInitScope::User;
                    index += 1;
                }
                "--interactive" | "-i" => {
                    interactive = true;
                    index += 1;
                }
                other => {
                    return Err(format!("unknown config init option: {other}"));
                }
            }
        }
        return Ok(ConfigSubAction::Init { scope, interactive });
    }

    if sub == Some("show") {
        let section = args.get(1).cloned();
        return Ok(ConfigSubAction::Show { section });
    }

    if let Some(other) = sub {
        return Err(format!(
            "unknown config subcommand: {other} (可用: init, show)"
        ));
    }

    Ok(ConfigSubAction::Guide)
}

pub(crate) fn run_config(action: ConfigSubAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ConfigSubAction::Guide => run_config_guide(),
        ConfigSubAction::Init { scope, interactive } => run_config_init(scope, interactive),
        ConfigSubAction::Show { section } => {
            println!(
                "{}",
                crate::format_report::render_config_report(section.as_deref())?
            );
            Ok(())
        }
    }
}

fn run_config_guide() -> Result<(), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let loader = ConfigLoader::default_for(&cwd);
    let config_home = config_home_dir();
    let runtime_config = loader.load()?;
    let discovered = loader.discover();

    println!("云熙智能体 — 配置概览 (yunxi config)\n");

    println!("配置目录  {}", config_home.display());
    println!("工作目录  {}", cwd.display());
    println!(
        "已加载    {}/{} 个配置文件",
        runtime_config.loaded_entries().len(),
        discovered.len()
    );
    println!();

    println!("配置文件");
    for entry in &discovered {
        let source = match entry.source {
            ConfigSource::User => "user",
            ConfigSource::Project => "project",
            ConfigSource::Local => "local",
        };
        let exists = entry.path.is_file();
        let loaded = runtime_config
            .loaded_entries()
            .iter()
            .any(|loaded_entry| loaded_entry.path == entry.path);
        let status = if loaded {
            "loaded"
        } else if exists {
            "skipped"
        } else {
            "missing"
        };
        println!("  {source:<7} {status:<7} {}", entry.path.display());
    }
    println!();

    print_capability_hints(&runtime_config);
    print_next_steps(&cwd, &discovered);

    Ok(())
}

fn run_config_init(
    scope: ConfigInitScope,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let target = match scope {
        ConfigInitScope::User => config_home_dir().join("settings.local.json"),
        ConfigInitScope::Project => cwd.join(".yunxi/settings.local.json"),
    };

    println!("云熙智能体 — 配置初始化 (yunxi config init)\n");
    println!("目标文件  {}", target.display());

    if target.is_file() {
        println!("状态      已存在，跳过创建（使用 --interactive 可补充密钥）");
    } else {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let template = load_local_template(&cwd);
        fs::write(&target, template)?;
        println!("状态      已从模板创建");
    }

    if interactive {
        let mut document = read_json_object(&target)?;
        let changed = prompt_and_merge_secrets(&mut document)?;
        if changed {
            write_json_object(&target, &document)?;
            println!("状态      已写入交互式配置");
        }
    }

    println!();
    println!("下一步：");
    println!("  1. 运行 yunxi config        查看配置状态");
    println!("  2. 运行 yunxi doctor        检查本机环境");
    println!("  3. 运行 yunxi               启动交互式智能体");
    if scope == ConfigInitScope::User {
        println!();
        println!("提示：密钥与语义服务配置请放在 ~/.yunxi/settings.local.json，勿提交到 Git。");
    }

    Ok(())
}

fn print_capability_hints(runtime_config: &runtime::RuntimeConfig) {
    println!("能力检查");

    let deepseek =
        env_var_set("DEEPSEEK_API_KEY") || config_env_set(runtime_config, "DEEPSEEK_API_KEY");
    let anthropic = env_var_set("ANTHROPIC_API_KEY")
        || env_var_set("ANTHROPIC_AUTH_TOKEN")
        || config_env_set(runtime_config, "ANTHROPIC_API_KEY");
    if deepseek {
        println!("  ✓ LLM         DEEPSEEK_API_KEY 已配置");
    } else if anthropic {
        println!("  ✓ LLM         Anthropic 凭据已配置");
    } else {
        println!("  ✗ LLM         未设置 DEEPSEEK_API_KEY 或 ANTHROPIC_API_KEY");
    }

    let semantic_enabled = runtime_config
        .get("semantic")
        .and_then(|v| v.as_object())
        .and_then(|o| o.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let omlx_key = env_var_set("OMLX_API_KEY") || config_env_set(runtime_config, "OMLX_API_KEY");
    if semantic_enabled {
        let base = runtime_config
            .get("semantic")
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("http"))
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("baseUrl"))
            .and_then(|v| v.as_str())
            .unwrap_or("http://127.0.0.1:8009");
        if omlx_key {
            println!("  ✓ 语义检索    已启用 ({base})");
        } else {
            println!("  ! 语义检索    已启用但缺少 OMLX_API_KEY（混合检索可能降级）");
        }
    } else {
        println!("  - 语义检索    未启用（可在 settings.local.json 配置 semantic 块）");
    }

    let model = runtime_config
        .model()
        .or_else(|| runtime_config.get("model").and_then(|v| v.as_str()))
        .unwrap_or("deepseek-v4-pro");
    println!("  · 默认模型    {model}");
    println!();
}

fn print_next_steps(cwd: &Path, discovered: &[runtime::ConfigEntry]) {
    let user_local = config_home_dir().join("settings.local.json");
    let user_local_missing = !user_local.is_file();
    let project_local_missing = !cwd.join(".yunxi/settings.local.json").is_file();
    let project_settings_missing = !cwd.join(".yunxi/settings.json").is_file();
    let project_local_in_repo = discovered
        .iter()
        .any(|entry| entry.path.ends_with(".yunxi/settings.local.json") && entry.path.is_file());

    println!("推荐操作");
    if user_local_missing && !project_local_in_repo {
        println!("  → yunxi config init              创建 ~/.yunxi/settings.local.json");
    }
    if project_local_missing && project_settings_missing {
        println!("  → yunxi config init --project    创建项目 .yunxi/settings.local.json");
    }
    if !env_var_set("DEEPSEEK_API_KEY") && !env_var_set("ANTHROPIC_API_KEY") {
        println!("  → export DEEPSEEK_API_KEY=...    或 yunxi config init -i 交互写入");
    }
    println!("  → yunxi config show              查看合并后的完整配置");
    println!("  → yunxi config show env          查看 env 块");
    println!("  → yunxi doctor                   本机环境健康检查");
}

fn config_home_dir() -> PathBuf {
    env::var_os("YUNXI_CONFIG_HOME")
        .or_else(|| env::var_os("CLAUDE_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".yunxi")))
        .unwrap_or_else(|| PathBuf::from(".yunxi"))
}

fn load_local_template(cwd: &Path) -> String {
    if let Some(root) = locate_repo_root_from(cwd) {
        let example = root.join(".yunxi/settings.semantic.example.json");
        if example.is_file() {
            if let Ok(content) = fs::read_to_string(&example) {
                return content;
            }
        }
    }
    USER_LOCAL_TEMPLATE.to_string()
}

fn locate_repo_root_from(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        if ancestor.join("rust/Cargo.toml").is_file() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn env_var_set(name: &str) -> bool {
    env::var(name)
        .ok()
        .is_some_and(|value| !value.trim().is_empty())
}

fn config_env_set(runtime_config: &runtime::RuntimeConfig, key: &str) -> bool {
    runtime_config
        .get("env")
        .and_then(|v| v.as_object())
        .and_then(|map| map.get(key))
        .and_then(|v| v.as_str())
        .is_some_and(|value| !value.trim().is_empty())
}

fn prompt_and_merge_secrets(document: &mut serde_json::Map<String, Value>) -> io::Result<bool> {
    let stdin = io::stdin();
    let mut changed = false;

    let deepseek = prompt_secret(
        &stdin,
        "DEEPSEEK_API_KEY",
        env_var_set("DEEPSEEK_API_KEY") || json_env_set(document, "DEEPSEEK_API_KEY"),
    )?;
    if let Some(value) = deepseek {
        set_json_env(document, "DEEPSEEK_API_KEY", &value);
        changed = true;
    }

    let omlx = prompt_secret(
        &stdin,
        "OMLX_API_KEY",
        env_var_set("OMLX_API_KEY") || json_env_set(document, "OMLX_API_KEY"),
    )?;
    if let Some(value) = omlx {
        set_json_env(document, "OMLX_API_KEY", &value);
        ensure_semantic_block(document);
        set_nested_string(document, &["semantic", "http", "apiKey"], &value);
        changed = true;
    }

    Ok(changed)
}

fn prompt_secret(stdin: &io::Stdin, name: &str, already_set: bool) -> io::Result<Option<String>> {
    let mut stderr = io::stderr();
    if already_set {
        writeln!(stderr, "{name} 已存在，按 Enter 跳过")?;
    } else {
        writeln!(stderr, "请输入 {name}（留空跳过）:")?;
    }
    stderr.flush()?;
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.to_string()))
}

fn json_env_set(document: &serde_json::Map<String, Value>, key: &str) -> bool {
    document
        .get("env")
        .and_then(|v| v.as_object())
        .and_then(|map| map.get(key))
        .and_then(|v| v.as_str())
        .is_some_and(|value| !value.trim().is_empty())
}

fn set_json_env(document: &mut serde_json::Map<String, Value>, key: &str, value: &str) {
    let env_entry = document
        .entry("env".to_string())
        .or_insert_with(|| json!({}));
    if let Some(map) = env_entry.as_object_mut() {
        map.insert(key.to_string(), Value::String(value.to_string()));
    }
}

fn ensure_semantic_block(document: &mut serde_json::Map<String, Value>) {
    if document.contains_key("semantic") {
        return;
    }
    document.insert(
        "semantic".to_string(),
        json!({
            "enabled": true,
            "backend": "http",
            "http": {
                "baseUrl": "http://127.0.0.1:8009",
                "embedPath": "/v1/embeddings",
                "apiStyle": "openai",
                "model": "bge-m3-mlx-8bit",
                "apiKey": "",
                "timeoutSecs": 120
            },
            "defaults": {
                "knowledgeSearchMode": "hybrid",
                "semanticCompareAuto": true
            }
        }),
    );
}

fn set_nested_string(root: &mut serde_json::Map<String, Value>, path: &[&str], value: &str) {
    let Some((first, rest)) = path.split_first() else {
        return;
    };
    if rest.is_empty() {
        root.insert(first.to_string(), Value::String(value.to_string()));
        return;
    }
    let entry = root
        .entry((*first).to_string())
        .or_insert_with(|| json!({}));
    if let Some(map) = entry.as_object_mut() {
        set_nested_string(map, rest, value);
    }
}

fn read_json_object(path: &Path) -> io::Result<serde_json::Map<String, Value>> {
    let contents = fs::read_to_string(path)?;
    if contents.trim().is_empty() {
        return Ok(serde_json::Map::new());
    }
    serde_json::from_str(&contents)
        .map_err(io::Error::other)
        .and_then(|value: Value| {
            value
                .as_object()
                .cloned()
                .ok_or_else(|| io::Error::other("config file must contain a JSON object"))
        })
}

fn write_json_object(path: &Path, value: &serde_json::Map<String, Value>) -> io::Result<()> {
    let rendered = serde_json::to_string_pretty(value).map_err(io::Error::other)?;
    fs::write(path, format!("{rendered}\n"))
}

#[cfg(test)]
mod tests {
    use super::{
        load_local_template, parse_config_args, set_json_env, ConfigInitScope, ConfigSubAction,
    };
    use std::path::Path;

    #[test]
    fn parses_config_guide_by_default() {
        assert_eq!(
            parse_config_args(&[]).expect("parse"),
            ConfigSubAction::Guide
        );
    }

    #[test]
    fn parses_config_init_with_flags() {
        assert_eq!(
            parse_config_args(&[
                "init".to_string(),
                "--project".to_string(),
                "--interactive".to_string(),
            ])
            .expect("parse"),
            ConfigSubAction::Init {
                scope: ConfigInitScope::Project,
                interactive: true,
            }
        );
    }

    #[test]
    fn parses_config_show_with_section() {
        assert_eq!(
            parse_config_args(&["show".to_string(), "env".to_string()]).expect("parse"),
            ConfigSubAction::Show {
                section: Some("env".to_string()),
            }
        );
    }

    #[test]
    fn rejects_unknown_config_subcommand() {
        let error = parse_config_args(&["reset".to_string()]).expect_err("should fail");
        assert!(error.contains("unknown config subcommand"));
    }

    #[test]
    fn embedded_template_is_valid_json() {
        let template = load_local_template(Path::new("/tmp"));
        let parsed: serde_json::Value =
            serde_json::from_str(&template).expect("template should be valid json");
        assert!(parsed.get("semantic").is_some());
    }

    #[test]
    fn set_json_env_creates_env_object() {
        let mut doc = serde_json::Map::new();
        set_json_env(&mut doc, "DEEPSEEK_API_KEY", "sk-test");
        assert_eq!(
            doc.get("env")
                .and_then(|v| v.get("DEEPSEEK_API_KEY"))
                .and_then(|v| v.as_str()),
            Some("sk-test")
        );
    }
}
