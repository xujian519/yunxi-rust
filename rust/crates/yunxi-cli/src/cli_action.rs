use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::PathBuf;

use runtime::PermissionMode;
use tools::mvp_tool_specs;

use crate::llm_auth::llm_auth_configured;
use crate::model_routing::default_model_from_config;
use crate::DEFAULT_DATE;

const PATENT_TUI_REMOVED_MSG: &str =
    "专利专屏 TUI 已移除。请使用 yunxi-desktop 桌面客户端（`cargo run -p yunxi-cli --bin yunxi-desktop --features desktop`）。";

const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_SERVER_PORT: u16 = 8765;

pub(crate) type AllowedToolSet = BTreeSet<String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CliAction {
    DumpManifests,
    BootstrapPlan,
    PrintSystemPrompt {
        cwd: PathBuf,
        date: String,
    },
    Version,
    ResumeSession {
        session_path: PathBuf,
        commands: Vec<String>,
    },
    Prompt {
        prompt: String,
        model: String,
        output_format: CliOutputFormat,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
    },
    Login,
    Logout,
    Init,
    Doctor,
    Config {
        action: crate::config::ConfigSubAction,
    },
    Server {
        host: String,
        port: u16,
    },
    SelfUpdate,
    Tui {
        model: String,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
        resume_session: Option<PathBuf>,
    },
    Repl {
        model: String,
        allowed_tools: Option<AllowedToolSet>,
        permission_mode: PermissionMode,
    },
    // prompt-mode formatting is only supported for non-interactive runs
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliOutputFormat {
    Text,
    Json,
}

impl CliOutputFormat {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            other => Err(format!(
                "unsupported value for --output-format: {other} (expected text or json)"
            )),
        }
    }
}

#[allow(clippy::too_many_lines)]
pub(crate) fn parse_args(args: &[String]) -> Result<CliAction, String> {
    let mut model = normalize_startup_model(&default_model_from_config());
    let mut output_format = CliOutputFormat::Text;
    let mut permission_mode = default_permission_mode();
    let mut wants_version = false;
    let mut cli_profile: Option<String> = None;
    let mut allowed_tool_values = Vec::new();
    let mut rest = Vec::new();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--patent" => return Err(PATENT_TUI_REMOVED_MSG.to_string()),
            "--profile" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --profile".to_string())?;
                cli_profile = Some(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--profile=") => {
                cli_profile = Some(flag[10..].to_string());
                index += 1;
            }
            "--version" | "-V" => {
                wants_version = true;
                index += 1;
            }
            "--model" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --model".to_string())?;
                model = normalize_startup_model(value);
                index += 2;
            }
            flag if flag.starts_with("--model=") => {
                model = normalize_startup_model(&flag[8..]);
                index += 1;
            }
            "--output-format" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --output-format".to_string())?;
                output_format = CliOutputFormat::parse(value)?;
                index += 2;
            }
            "--permission-mode" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --permission-mode".to_string())?;
                permission_mode = parse_permission_mode_arg(value)?;
                index += 2;
            }
            flag if flag.starts_with("--output-format=") => {
                output_format = CliOutputFormat::parse(&flag[16..])?;
                index += 1;
            }
            flag if flag.starts_with("--permission-mode=") => {
                permission_mode = parse_permission_mode_arg(&flag[18..])?;
                index += 1;
            }
            "--dangerously-skip-permissions" => {
                permission_mode = PermissionMode::DangerFullAccess;
                index += 1;
            }
            "--repl" => {
                reject_removed_patent_ui(cli_profile.as_deref())?;
                return Ok(CliAction::Repl {
                    model: resolve_model_alias(&model).to_string(),
                    allowed_tools: normalize_allowed_tools(&allowed_tool_values)?,
                    permission_mode,
                });
            }
            "-p" => {
                // Yunxi compat: -p "prompt" = one-shot prompt
                let prompt = args[index + 1..].join(" ");
                if prompt.trim().is_empty() {
                    return Err("-p requires a prompt string".to_string());
                }
                return Ok(CliAction::Prompt {
                    prompt,
                    model: resolve_model_alias(&model).to_string(),
                    output_format,
                    allowed_tools: normalize_allowed_tools(&allowed_tool_values)?,
                    permission_mode,
                });
            }
            "--print" => {
                // Yunxi compat: --print makes output non-interactive
                output_format = CliOutputFormat::Text;
                index += 1;
            }
            "--allowedTools" | "--allowed-tools" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --allowedTools".to_string())?;
                allowed_tool_values.push(value.clone());
                index += 2;
            }
            flag if flag.starts_with("--allowedTools=") => {
                allowed_tool_values.push(flag[15..].to_string());
                index += 1;
            }
            flag if flag.starts_with("--allowed-tools=") => {
                allowed_tool_values.push(flag[16..].to_string());
                index += 1;
            }
            "--resume" => {
                rest.push("--resume".to_string());
                index += 1;
            }
            other => {
                rest.push(other.to_string());
                index += 1;
            }
        }
    }

    if wants_version {
        return Ok(CliAction::Version);
    }

    let allowed_tools = normalize_allowed_tools(&allowed_tool_values)?;
    reject_removed_patent_ui(cli_profile.as_deref())?;

    if rest.is_empty() {
        return Ok(CliAction::Tui {
            model,
            allowed_tools,
            permission_mode,
            resume_session: None,
        });
    }
    if matches!(rest.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(CliAction::Help);
    }
    // --resume with slash commands → ResumeSession (REPL mode)
    // --resume <path> → TUI resume
    if rest.first().map(String::as_str) == Some("--resume") {
        let resume_args = &rest[1..];
        // Slash commands: '/word' single-segment, no dot, no nested '/', and NOT an existing filesystem path.
        // This correctly handles edge cases like /tmp or /home which are real paths but
        // would otherwise match the slash-command pattern.
        let has_slash_commands = resume_args.iter().any(|a| {
            if !a.starts_with('/') { return false; }
            if std::path::Path::new(a).exists() { return false; }
            let rest = &a[1..];
            !rest.contains('/') && !rest.contains('.')
        });
        if has_slash_commands {
            return parse_resume_args(resume_args);
        }
        // Otherwise: TUI resume with session path
        if let Some(path) = resume_args.first() {
            return Ok(CliAction::Tui {
                model,
                allowed_tools,
                permission_mode,
                resume_session: Some(PathBuf::from(path)),
            });
        }
        return Err("--resume requires a session path or slash command".to_string());
    }

    match rest[0].as_str() {
        "dump-manifests" => Ok(CliAction::DumpManifests),
        "bootstrap-plan" => Ok(CliAction::BootstrapPlan),
        "system-prompt" => parse_system_prompt_args(&rest[1..]),
        "login" => Ok(CliAction::Login),
        "logout" => Ok(CliAction::Logout),
        "init" => Ok(CliAction::Init),
        "doctor" => Ok(CliAction::Doctor),
        "config" => {
            let action = crate::config::parse_config_args(&rest[1..])?;
            Ok(CliAction::Config { action })
        }
        "server" => {
            let (host, port) = parse_server_args(&rest[1..])?;
            Ok(CliAction::Server { host, port })
        }
        "self-update" => Ok(CliAction::SelfUpdate),
        "prompt" => {
            let prompt = rest[1..].join(" ");
            if prompt.trim().is_empty() {
                return Err("prompt subcommand requires a prompt string".to_string());
            }
            Ok(CliAction::Prompt {
                prompt,
                model,
                output_format,
                allowed_tools,
                permission_mode,
            })
        }
        other if !other.starts_with('/') => Ok(CliAction::Prompt {
            prompt: rest.join(" "),
            model,
            output_format,
            allowed_tools,
            permission_mode,
        }),
        other => Err(format!("unknown subcommand: {other}")),
    }
}

/// 启动 TUI/REPL/单次 prompt 时使用的模型：默认 DeepSeek，auto 映射为 pro，缺失密钥时回退。
#[must_use]
pub fn normalize_startup_model(model: &str) -> String {
    let mut m = resolve_model_alias(model).to_string();
    if m == "auto" {
        m = "deepseek-v4-pro".to_string();
    }
    if !llm_auth_configured(&m) && llm_auth_configured("deepseek-v4-pro") {
        m = "deepseek-v4-pro".to_string();
    }
    m
}

pub(crate) fn resolve_model_alias(model: &str) -> &str {
    match model {
        // Auto mode
        "auto" => "auto",
        // Anthropic Claude 系列
        "opus" => "claude-opus-4-6",
        "sonnet" => "claude-sonnet-4-6",
        "haiku" => "claude-haiku-4-5-20251213",
        // DeepSeek 系列（通过 Anthropic 协议接入）
        "deepseek" | "ds" => "deepseek-v4-pro",
        "deepseek-flash" | "dsf" => "deepseek-v4-flash",
        _ => model,
    }
}

pub(crate) fn normalize_allowed_tools(values: &[String]) -> Result<Option<AllowedToolSet>, String> {
    if values.is_empty() {
        return Ok(None);
    }

    let canonical_names = mvp_tool_specs()
        .into_iter()
        .map(|spec| spec.name.to_string())
        .collect::<Vec<_>>();
    let mut name_map = canonical_names
        .iter()
        .map(|name| (normalize_tool_name(name), name.clone()))
        .collect::<BTreeMap<_, _>>();

    for (alias, canonical) in [
        ("read", "read_file"),
        ("write", "write_file"),
        ("edit", "edit_file"),
        ("glob", "glob_search"),
        ("grep", "grep_search"),
    ] {
        name_map.insert(alias.to_string(), canonical.to_string());
    }

    let mut allowed = AllowedToolSet::new();
    for value in values {
        for token in value
            .split(|ch: char| ch == ',' || ch.is_whitespace())
            .filter(|token| !token.is_empty())
        {
            let normalized = normalize_tool_name(token);
            let canonical = name_map.get(&normalized).ok_or_else(|| {
                format!(
                    "unsupported tool in --allowedTools: {token} (expected one of: {})",
                    canonical_names.join(", ")
                )
            })?;
            allowed.insert(canonical.clone());
        }
    }

    Ok(Some(allowed))
}

pub(crate) fn normalize_tool_name(value: &str) -> String {
    value.trim().replace('-', "_").to_ascii_lowercase()
}

pub(crate) fn reject_removed_patent_ui(cli_profile: Option<&str>) -> Result<(), String> {
    if cli_profile.is_some_and(|p| p.eq_ignore_ascii_case("patent")) {
        return Err(PATENT_TUI_REMOVED_MSG.to_string());
    }
    if env::var("YUNXI_UI_MODE")
        .ok()
        .is_some_and(|m| m.eq_ignore_ascii_case("patent"))
    {
        return Err(PATENT_TUI_REMOVED_MSG.to_string());
    }
    Ok(())
}

fn parse_permission_mode_arg(value: &str) -> Result<PermissionMode, String> {
    normalize_permission_mode(value)
        .ok_or_else(|| {
            format!(
                "unsupported permission mode '{value}'. Use read-only, workspace-write, or danger-full-access."
            )
        })
        .map(permission_mode_from_label)
}

pub(crate) fn permission_mode_from_label(mode: &str) -> PermissionMode {
    match mode {
        "read-only" => PermissionMode::ReadOnly,
        "workspace-write" => PermissionMode::WorkspaceWrite,
        "danger-full-access" => PermissionMode::DangerFullAccess,
        other => panic!("unsupported permission mode label: {other}"),
    }
}

pub(crate) fn default_permission_mode() -> PermissionMode {
    env::var("RUSTY_CLAUDE_PERMISSION_MODE")
        .ok()
        .as_deref()
        .and_then(normalize_permission_mode)
        .map_or(PermissionMode::DangerFullAccess, permission_mode_from_label)
}

#[allow(dead_code)]
pub(crate) fn filter_tool_specs(allowed_tools: Option<&AllowedToolSet>) -> Vec<tools::ToolSpec> {
    mvp_tool_specs()
        .into_iter()
        .filter(|spec| allowed_tools.is_none_or(|allowed| allowed.contains(spec.name)))
        .collect()
}

pub(crate) fn normalize_permission_mode(mode: &str) -> Option<&'static str> {
    match mode.trim() {
        "read-only" => Some("read-only"),
        "workspace-write" => Some("workspace-write"),
        "danger-full-access" => Some("danger-full-access"),
        _ => None,
    }
}

fn parse_system_prompt_args(args: &[String]) -> Result<CliAction, String> {
    let mut cwd = env::current_dir().map_err(|error| error.to_string())?;
    let mut date = DEFAULT_DATE.to_string();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--cwd" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --cwd".to_string())?;
                cwd = PathBuf::from(value);
                index += 2;
            }
            "--date" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --date".to_string())?;
                date.clone_from(value);
                index += 2;
            }
            other => return Err(format!("unknown system-prompt option: {other}")),
        }
    }

    Ok(CliAction::PrintSystemPrompt { cwd, date })
}

fn parse_server_args(args: &[String]) -> Result<(String, u16), String> {
    let mut host = DEFAULT_SERVER_HOST.to_string();
    let mut port = DEFAULT_SERVER_PORT;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                host = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --host".to_string())?
                    .clone();
                index += 2;
            }
            flag if flag.starts_with("--host=") => {
                host = flag[7..].to_string();
                index += 1;
            }
            "--port" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --port".to_string())?;
                port = value
                    .parse()
                    .map_err(|_| format!("invalid --port value: {value}"))?;
                index += 2;
            }
            flag if flag.starts_with("--port=") => {
                port = flag[7..]
                    .parse()
                    .map_err(|_| format!("invalid --port value: {}", &flag[7..]))?;
                index += 1;
            }
            other => return Err(format!("unknown server option: {other}")),
        }
    }

    Ok((host, port))
}

fn parse_resume_args(args: &[String]) -> Result<CliAction, String> {
    let session_path = args
        .first()
        .ok_or_else(|| "missing session path for --resume".to_string())
        .map(PathBuf::from)?;
    let commands = args[1..].to_vec();
    if commands
        .iter()
        .any(|command| !command.trim_start().starts_with('/'))
    {
        return Err("--resume trailing arguments must be slash commands".to_string());
    }
    Ok(CliAction::ResumeSession {
        session_path,
        commands,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        filter_tool_specs, normalize_permission_mode, parse_args, resolve_model_alias, CliAction,
        CliOutputFormat,
    };
    use crate::DEFAULT_MODEL;
    use runtime::PermissionMode;
    use std::path::PathBuf;

    #[test]
    fn defaults_to_tui_when_no_args() {
        assert_eq!(
            parse_args(&[]).expect("args should parse"),
            CliAction::Tui {
                model: DEFAULT_MODEL.to_string(),
                allowed_tools: None,
                permission_mode: PermissionMode::DangerFullAccess,
                resume_session: None,
            }
        );
    }

    #[test]
    fn rejects_patent_flag() {
        let error = parse_args(&["--patent".to_string()]).expect_err("patent flag should fail");
        assert!(error.contains("专利专屏 TUI 已移除"));
    }

    #[test]
    fn parses_server_subcommand_with_port() {
        assert_eq!(
            parse_args(&[
                "server".to_string(),
                "--host".to_string(),
                "0.0.0.0".to_string(),
                "--port".to_string(),
                "9000".to_string(),
            ])
            .expect("args should parse"),
            CliAction::Server {
                host: "0.0.0.0".to_string(),
                port: 9000,
            }
        );
    }

    #[test]
    fn parses_self_update_subcommand() {
        assert_eq!(
            parse_args(&["self-update".to_string()]).expect("args should parse"),
            CliAction::SelfUpdate
        );
    }

    #[test]
    fn parses_prompt_subcommand() {
        let args = vec![
            "prompt".to_string(),
            "hello".to_string(),
            "world".to_string(),
        ];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::Prompt {
                prompt: "hello world".to_string(),
                model: DEFAULT_MODEL.to_string(),
                output_format: CliOutputFormat::Text,
                allowed_tools: None,
                permission_mode: PermissionMode::DangerFullAccess,
            }
        );
    }

    #[test]
    fn parses_bare_prompt_and_json_output_flag() {
        let args = vec![
            "--output-format=json".to_string(),
            "--model".to_string(),
            "deepseek-v4-pro".to_string(),
            "explain".to_string(),
            "this".to_string(),
        ];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::Prompt {
                prompt: "explain this".to_string(),
                model: "deepseek-v4-pro".to_string(),
                output_format: CliOutputFormat::Json,
                allowed_tools: None,
                permission_mode: PermissionMode::DangerFullAccess,
            }
        );
    }

    #[test]
    fn resolves_model_aliases_in_args() {
        let args = vec![
            "--model".to_string(),
            "ds".to_string(),
            "explain".to_string(),
            "this".to_string(),
        ];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::Prompt {
                prompt: "explain this".to_string(),
                model: "deepseek-v4-pro".to_string(),
                output_format: CliOutputFormat::Text,
                allowed_tools: None,
                permission_mode: PermissionMode::DangerFullAccess,
            }
        );
    }

    #[test]
    fn resolves_known_model_aliases() {
        assert_eq!(resolve_model_alias("opus"), "claude-opus-4-6");
        assert_eq!(resolve_model_alias("sonnet"), "claude-sonnet-4-6");
        assert_eq!(resolve_model_alias("haiku"), "claude-haiku-4-5-20251213");
        assert_eq!(resolve_model_alias("claude-opus"), "claude-opus");
        // DeepSeek 别名
        assert_eq!(resolve_model_alias("deepseek"), "deepseek-v4-pro");
        assert_eq!(resolve_model_alias("ds"), "deepseek-v4-pro");
        assert_eq!(resolve_model_alias("deepseek-flash"), "deepseek-v4-flash");
        assert_eq!(resolve_model_alias("dsf"), "deepseek-v4-flash");
    }

    #[test]
    fn parses_version_flags_without_initializing_prompt_mode() {
        assert_eq!(
            parse_args(&["--version".to_string()]).expect("args should parse"),
            CliAction::Version
        );
        assert_eq!(
            parse_args(&["-V".to_string()]).expect("args should parse"),
            CliAction::Version
        );
    }

    #[test]
    fn parses_permission_mode_flag() {
        let args = vec!["--permission-mode=read-only".to_string()];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::Tui {
                model: DEFAULT_MODEL.to_string(),
                allowed_tools: None,
                permission_mode: PermissionMode::ReadOnly,
                resume_session: None,
            }
        );
    }

    #[test]
    fn parses_allowed_tools_flags_with_aliases_and_lists() {
        let args = vec![
            "--allowedTools".to_string(),
            "read,glob".to_string(),
            "--allowed-tools=write_file".to_string(),
        ];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::Tui {
                model: DEFAULT_MODEL.to_string(),
                allowed_tools: Some(
                    ["glob_search", "read_file", "write_file"]
                        .into_iter()
                        .map(str::to_string)
                        .collect()
                ),
                permission_mode: PermissionMode::DangerFullAccess,
                resume_session: None,
            }
        );
    }

    #[test]
    fn rejects_unknown_allowed_tools() {
        let error = parse_args(&["--allowedTools".to_string(), "teleport".to_string()])
            .expect_err("tool should be rejected");
        assert!(error.contains("unsupported tool in --allowedTools: teleport"));
    }

    #[test]
    fn parses_system_prompt_options() {
        let args = vec![
            "system-prompt".to_string(),
            "--cwd".to_string(),
            "/tmp/project".to_string(),
            "--date".to_string(),
            "2026-04-01".to_string(),
        ];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::PrintSystemPrompt {
                cwd: PathBuf::from("/tmp/project"),
                date: "2026-04-01".to_string(),
            }
        );
    }

    #[test]
    fn parses_config_subcommands() {
        assert_eq!(
            parse_args(&["config".to_string()]).expect("config should parse"),
            CliAction::Config {
                action: crate::config::ConfigSubAction::Guide,
            }
        );
        assert_eq!(
            parse_args(&[
                "config".to_string(),
                "init".to_string(),
                "--interactive".to_string(),
            ])
            .expect("config init should parse"),
            CliAction::Config {
                action: crate::config::ConfigSubAction::Init {
                    scope: crate::config::ConfigInitScope::User,
                    interactive: true,
                },
            }
        );
        assert_eq!(
            parse_args(&["config".to_string(), "show".to_string(), "env".to_string()])
                .expect("config show should parse"),
            CliAction::Config {
                action: crate::config::ConfigSubAction::Show {
                    section: Some("env".to_string()),
                },
            }
        );
    }

    #[test]
    fn parses_login_and_logout_subcommands() {
        assert_eq!(
            parse_args(&["login".to_string()]).expect("login should parse"),
            CliAction::Login
        );
        assert_eq!(
            parse_args(&["logout".to_string()]).expect("logout should parse"),
            CliAction::Logout
        );
        assert_eq!(
            parse_args(&["init".to_string()]).expect("init should parse"),
            CliAction::Init
        );
    }

    #[test]
    fn parses_resume_flag_with_slash_command() {
        let args = vec![
            "--resume".to_string(),
            "session.json".to_string(),
            "/compact".to_string(),
        ];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::ResumeSession {
                session_path: PathBuf::from("session.json"),
                commands: vec!["/compact".to_string()],
            }
        );
    }

    #[test]
    fn parses_resume_flag_with_multiple_slash_commands() {
        let args = vec![
            "--resume".to_string(),
            "session.json".to_string(),
            "/status".to_string(),
            "/compact".to_string(),
            "/cost".to_string(),
        ];
        assert_eq!(
            parse_args(&args).expect("args should parse"),
            CliAction::ResumeSession {
                session_path: PathBuf::from("session.json"),
                commands: vec![
                    "/status".to_string(),
                    "/compact".to_string(),
                    "/cost".to_string(),
                ],
            }
        );
    }

    #[test]
    fn filtered_tool_specs_respect_allowlist() {
        let allowed = ["read_file", "grep_search"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let filtered = filter_tool_specs(Some(&allowed));
        let names = filtered
            .into_iter()
            .map(|spec| spec.name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["read_file", "grep_search"]);
    }

    #[test]
    fn normalizes_supported_permission_modes() {
        assert_eq!(normalize_permission_mode("read-only"), Some("read-only"));
        assert_eq!(
            normalize_permission_mode("workspace-write"),
            Some("workspace-write")
        );
        assert_eq!(
            normalize_permission_mode("danger-full-access"),
            Some("danger-full-access")
        );
        assert_eq!(normalize_permission_mode("unknown"), None);
    }

    #[test]
    fn test_auto_mode_alias() {
        assert_eq!(resolve_model_alias("auto"), "auto");
    }
}
