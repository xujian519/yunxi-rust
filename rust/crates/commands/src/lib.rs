mod custom_slash;
mod slash_registry;

pub use custom_slash::{
    custom_slash_command_names, custom_slash_registry, load_custom_slash_commands,
    resolve_custom_prompt, CustomSlashCommand,
};
pub use slash_registry::{all_slash_command_names, render_full_slash_help, SLASH_ALIAS_HELP_LINES};

use runtime::{compact_session, CompactionConfig, Session};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandManifestEntry {
    pub name: String,
    pub source: CommandSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSource {
    Builtin,
    InternalOnly,
    FeatureGated,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandRegistry {
    entries: Vec<CommandManifestEntry>,
}

impl CommandRegistry {
    #[must_use]
    pub fn new(entries: Vec<CommandManifestEntry>) -> Self {
        Self { entries }
    }

    #[must_use]
    pub fn entries(&self) -> &[CommandManifestEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlashCommandSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub argument_hint: Option<&'static str>,
    pub resume_supported: bool,
    /// Unicode 图标（如 "❓"、"📊" 等），用于 TUI 补全菜单展示。
    pub icon: &'static str,
}

const SLASH_COMMAND_SPECS: &[SlashCommandSpec] = &[
    SlashCommandSpec {
        name: "help",
        summary: "显示可用斜杠命令",
        argument_hint: None,
        resume_supported: true,
        icon: "❓",
    },
    SlashCommandSpec {
        name: "status",
        summary: "显示当前会话状态",
        argument_hint: None,
        resume_supported: true,
        icon: "📊",
    },
    SlashCommandSpec {
        name: "compact",
        summary: "压缩本地会话历史",
        argument_hint: None,
        resume_supported: true,
        icon: "🗜️",
    },
    SlashCommandSpec {
        name: "model",
        summary: "显示或切换当前模型",
        argument_hint: Some("[模型]"),
        resume_supported: false,
        icon: "🤖",
    },
    SlashCommandSpec {
        name: "permissions",
        summary: "显示或切换权限模式",
        argument_hint: Some("[read-only|workspace-write|danger-full-access]"),
        resume_supported: false,
        icon: "🔒",
    },
    SlashCommandSpec {
        name: "clear",
        summary: "开始新的本地会话",
        argument_hint: Some("[--confirm]"),
        resume_supported: true,
        icon: "🆕",
    },
    SlashCommandSpec {
        name: "cost",
        summary: "显示本次会话的累计 Token 用量",
        argument_hint: None,
        resume_supported: true,
        icon: "💰",
    },
    SlashCommandSpec {
        name: "resume",
        summary: "加载已保存的会话",
        argument_hint: Some("<会话路径>"),
        resume_supported: false,
        icon: "📂",
    },
    SlashCommandSpec {
        name: "config",
        summary: "查看配置文件或合并后的配置项",
        argument_hint: Some("[env|hooks|model]"),
        resume_supported: true,
        icon: "⚙️",
    },
    SlashCommandSpec {
        name: "memory",
        summary: "查看已加载的指导记忆文件",
        argument_hint: None,
        resume_supported: true,
        icon: "🧠",
    },
    SlashCommandSpec {
        name: "init",
        summary: "为当前仓库生成 CLAUDE.md",
        argument_hint: None,
        resume_supported: true,
        icon: "📝",
    },
    SlashCommandSpec {
        name: "diff",
        summary: "显示工作区的 git diff",
        argument_hint: None,
        resume_supported: true,
        icon: "🔄",
    },
    SlashCommandSpec {
        name: "version",
        summary: "显示 CLI 版本和构建信息",
        argument_hint: None,
        resume_supported: true,
        icon: "ℹ️",
    },
    SlashCommandSpec {
        name: "bughunter",
        summary: "检查代码库中的潜在缺陷",
        argument_hint: Some("[范围]"),
        resume_supported: false,
        icon: "🐛",
    },
    SlashCommandSpec {
        name: "commit",
        summary: "生成提交信息并创建 git commit",
        argument_hint: None,
        resume_supported: false,
        icon: "💾",
    },
    SlashCommandSpec {
        name: "pr",
        summary: "基于对话起草或创建 Pull Request",
        argument_hint: Some("[上下文]"),
        resume_supported: false,
        icon: "🔀",
    },
    SlashCommandSpec {
        name: "issue",
        summary: "基于对话起草或创建 GitHub Issue",
        argument_hint: Some("[上下文]"),
        resume_supported: false,
        icon: "📋",
    },
    SlashCommandSpec {
        name: "ultraplan",
        summary: "运行深度规划提示词（多步推理）",
        argument_hint: Some("[任务]"),
        resume_supported: false,
        icon: "🎯",
    },
    SlashCommandSpec {
        name: "teleport",
        summary: "跳转到文件或符号",
        argument_hint: Some("<符号或路径>"),
        resume_supported: false,
        icon: "🚀",
    },
    SlashCommandSpec {
        name: "debug-tool-call",
        summary: "回放上次工具调用并显示调试详情",
        argument_hint: None,
        resume_supported: false,
        icon: "🔧",
    },
    SlashCommandSpec {
        name: "export",
        summary: "导出当前对话到文件",
        argument_hint: Some("[文件]"),
        resume_supported: true,
        icon: "📤",
    },
    SlashCommandSpec {
        name: "session",
        summary: "列出或切换本地会话",
        argument_hint: Some("[list|switch <会话ID>]"),
        resume_supported: false,
        icon: "💬",
    },
    SlashCommandSpec {
        name: "search",
        summary: "在会话历史中搜索关键词",
        argument_hint: Some("<关键词>"),
        resume_supported: true,
        icon: "🔍",
    },
    SlashCommandSpec {
        name: "undo",
        summary: "撤销上一次交互",
        argument_hint: None,
        resume_supported: false,
        icon: "↩️",
    },
    SlashCommandSpec {
        name: "connect",
        summary: "查看 API 与 OAuth 配置指引",
        argument_hint: None,
        resume_supported: false,
        icon: "🔗",
    },
    SlashCommandSpec {
        name: "thinking",
        summary: "切换推理过程在对话区的显示",
        argument_hint: None,
        resume_supported: false,
        icon: "💭",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Help,
    Status,
    Compact,
    Bughunter {
        scope: Option<String>,
    },
    Commit,
    Pr {
        context: Option<String>,
    },
    Issue {
        context: Option<String>,
    },
    Ultraplan {
        task: Option<String>,
    },
    Teleport {
        target: Option<String>,
    },
    DebugToolCall,
    Model {
        model: Option<String>,
    },
    Permissions {
        mode: Option<String>,
    },
    Clear {
        confirm: bool,
    },
    Cost,
    Resume {
        session_path: Option<String>,
    },
    Config {
        section: Option<String>,
    },
    Memory,
    Init,
    Diff,
    Version,
    Export {
        path: Option<String>,
    },
    Session {
        action: Option<String>,
        target: Option<String>,
    },
    Search {
        query: Option<String>,
    },
    Undo,
    Connect,
    ThinkingToggle,
    Custom {
        name: String,
        arguments: Option<String>,
    },
    Unknown(String),
}

impl SlashCommand {
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let mut parts = trimmed.trim_start_matches('/').split_whitespace();
        let command = parts.next().unwrap_or_default();
        let command_key = command.to_ascii_lowercase();
        Some(match command_key.as_str() {
            "" => Self::Help,
            "help" => Self::Help,
            "status" => Self::Status,
            "compact" | "summarize" => Self::Compact,
            "new" => Self::Clear { confirm: true },
            "models" => Self::Model { model: None },
            "sessions" | "continue" => Self::Session {
                action: None,
                target: None,
            },
            "connect" => Self::Connect,
            "thinking" => Self::ThinkingToggle,
            "bughunter" => Self::Bughunter {
                scope: remainder_after_command(trimmed, command),
            },
            "commit" => Self::Commit,
            "pr" => Self::Pr {
                context: remainder_after_command(trimmed, command),
            },
            "issue" => Self::Issue {
                context: remainder_after_command(trimmed, command),
            },
            "ultraplan" => Self::Ultraplan {
                task: remainder_after_command(trimmed, command),
            },
            "teleport" => Self::Teleport {
                target: remainder_after_command(trimmed, command),
            },
            "debug-tool-call" => Self::DebugToolCall,
            "model" => Self::Model {
                model: parts.next().map(ToOwned::to_owned),
            },
            "permissions" => Self::Permissions {
                mode: parts.next().map(ToOwned::to_owned),
            },
            "clear" => Self::Clear {
                confirm: parts.next() == Some("--confirm"),
            },
            "cost" => Self::Cost,
            "resume" => Self::Resume {
                session_path: parts.next().map(ToOwned::to_owned),
            },
            "config" => Self::Config {
                section: parts.next().map(ToOwned::to_owned),
            },
            "memory" => Self::Memory,
            "init" => Self::Init,
            "diff" => Self::Diff,
            "version" => Self::Version,
            "export" => Self::Export {
                path: parts.next().map(ToOwned::to_owned),
            },
            "session" => Self::Session {
                action: parts.next().map(ToOwned::to_owned),
                target: parts.next().map(ToOwned::to_owned),
            },
            "search" => Self::Search {
                query: remainder_after_command(trimmed, command),
            },
            "undo" => Self::Undo,
            other => {
                if custom_slash::custom_slash_registry().contains_key(other) {
                    Self::Custom {
                        name: other.to_string(),
                        arguments: remainder_after_command(trimmed, command),
                    }
                } else {
                    Self::Unknown(command.to_string())
                }
            }
        })
    }
}

fn remainder_after_command(input: &str, command: &str) -> Option<String> {
    input
        .trim()
        .strip_prefix(&format!("/{command}"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

#[must_use]
pub fn slash_command_specs() -> &'static [SlashCommandSpec] {
    SLASH_COMMAND_SPECS
}

#[must_use]
pub fn resume_supported_slash_commands() -> Vec<&'static SlashCommandSpec> {
    slash_command_specs()
        .iter()
        .filter(|spec| spec.resume_supported)
        .collect()
}

#[must_use]
pub fn render_slash_command_help() -> String {
    let mut lines = vec![
        "斜杠命令".to_string(),
        "  [resume] 表示该命令也支持 --resume SESSION.json".to_string(),
    ];
    for spec in slash_command_specs() {
        let name = match spec.argument_hint {
            Some(argument_hint) => format!("/{} {}", spec.name, argument_hint),
            None => format!("/{}", spec.name),
        };
        let resume = if spec.resume_supported {
            " [resume]"
        } else {
            ""
        };
        lines.push(format!("  {name:<20} {}{}", spec.summary, resume));
    }
    lines.join("\n")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashCommandResult {
    pub message: String,
    pub session: Session,
}

#[must_use]
pub fn handle_slash_command(
    input: &str,
    session: &Session,
    compaction: CompactionConfig,
) -> Option<SlashCommandResult> {
    match SlashCommand::parse(input)? {
        SlashCommand::Compact => {
            let result = compact_session(session, compaction);
            let message = if result.removed_message_count == 0 {
                "Compaction skipped: session is below the compaction threshold.".to_string()
            } else {
                format!(
                    "Compacted {} messages into a resumable system summary.",
                    result.removed_message_count
                )
            };
            Some(SlashCommandResult {
                message,
                session: result.compacted_session,
            })
        }
        SlashCommand::Help => Some(SlashCommandResult {
            message: render_full_slash_help(),
            session: session.clone(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        handle_slash_command, render_full_slash_help, render_slash_command_help,
        resume_supported_slash_commands, slash_command_specs, SlashCommand,
    };
    use runtime::{CompactionConfig, ContentBlock, ConversationMessage, MessageRole, Session};

    #[test]
    fn parses_supported_slash_commands() {
        assert_eq!(SlashCommand::parse("/help"), Some(SlashCommand::Help));
        assert_eq!(SlashCommand::parse("/"), Some(SlashCommand::Help));
        assert_eq!(SlashCommand::parse(" /status "), Some(SlashCommand::Status));
        assert_eq!(
            SlashCommand::parse("/bughunter runtime"),
            Some(SlashCommand::Bughunter {
                scope: Some("runtime".to_string())
            })
        );
        assert_eq!(SlashCommand::parse("/commit"), Some(SlashCommand::Commit));
        assert_eq!(
            SlashCommand::parse("/pr ready for review"),
            Some(SlashCommand::Pr {
                context: Some("ready for review".to_string())
            })
        );
        assert_eq!(
            SlashCommand::parse("/issue flaky test"),
            Some(SlashCommand::Issue {
                context: Some("flaky test".to_string())
            })
        );
        assert_eq!(
            SlashCommand::parse("/ultraplan ship both features"),
            Some(SlashCommand::Ultraplan {
                task: Some("ship both features".to_string())
            })
        );
        assert_eq!(
            SlashCommand::parse("/teleport conversation.rs"),
            Some(SlashCommand::Teleport {
                target: Some("conversation.rs".to_string())
            })
        );
        assert_eq!(
            SlashCommand::parse("/debug-tool-call"),
            Some(SlashCommand::DebugToolCall)
        );
        assert_eq!(
            SlashCommand::parse("/model claude-opus"),
            Some(SlashCommand::Model {
                model: Some("claude-opus".to_string()),
            })
        );
        assert_eq!(
            SlashCommand::parse("/model"),
            Some(SlashCommand::Model { model: None })
        );
        assert_eq!(
            SlashCommand::parse("/permissions read-only"),
            Some(SlashCommand::Permissions {
                mode: Some("read-only".to_string()),
            })
        );
        assert_eq!(
            SlashCommand::parse("/clear"),
            Some(SlashCommand::Clear { confirm: false })
        );
        assert_eq!(
            SlashCommand::parse("/clear --confirm"),
            Some(SlashCommand::Clear { confirm: true })
        );
        assert_eq!(SlashCommand::parse("/cost"), Some(SlashCommand::Cost));
        assert_eq!(
            SlashCommand::parse("/resume session.json"),
            Some(SlashCommand::Resume {
                session_path: Some("session.json".to_string()),
            })
        );
        assert_eq!(
            SlashCommand::parse("/config"),
            Some(SlashCommand::Config { section: None })
        );
        assert_eq!(
            SlashCommand::parse("/config env"),
            Some(SlashCommand::Config {
                section: Some("env".to_string())
            })
        );
        assert_eq!(SlashCommand::parse("/memory"), Some(SlashCommand::Memory));
        assert_eq!(SlashCommand::parse("/init"), Some(SlashCommand::Init));
        assert_eq!(SlashCommand::parse("/diff"), Some(SlashCommand::Diff));
        assert_eq!(SlashCommand::parse("/version"), Some(SlashCommand::Version));
        assert_eq!(
            SlashCommand::parse("/export notes.txt"),
            Some(SlashCommand::Export {
                path: Some("notes.txt".to_string())
            })
        );
        assert_eq!(
            SlashCommand::parse("/session switch abc123"),
            Some(SlashCommand::Session {
                action: Some("switch".to_string()),
                target: Some("abc123".to_string())
            })
        );
    }

    #[test]
    fn renders_help_from_shared_specs() {
        let help = render_slash_command_help();
        assert!(help.contains("也支持 --resume SESSION.json"));
        assert!(help.contains("/help"));
        assert!(help.contains("/status"));
        assert!(help.contains("/compact"));
        assert!(help.contains("/bughunter [范围]"));
        assert!(help.contains("/commit"));
        assert!(help.contains("/pr [上下文]"));
        assert!(help.contains("/issue [上下文]"));
        assert!(help.contains("/ultraplan [任务]"));
        assert!(help.contains("/teleport <符号或路径>"));
        assert!(help.contains("/debug-tool-call"));
        assert!(help.contains("/model [模型]"));
        assert!(help.contains("/permissions [read-only|workspace-write|danger-full-access]"));
        assert!(help.contains("/clear [--confirm]"));
        assert!(help.contains("/cost"));
        assert!(help.contains("/resume <会话路径>"));
        assert!(help.contains("/config [env|hooks|model]"));
        assert!(help.contains("/memory"));
        assert!(help.contains("/init"));
        assert!(help.contains("/diff"));
        assert!(help.contains("/version"));
        assert!(help.contains("/export [文件]"));
        assert!(help.contains("/session [list|switch <会话ID>]"));
        assert!(slash_command_specs().len() >= 24);
        assert!(render_full_slash_help().contains("/new"));
        assert_eq!(
            SlashCommand::parse("/new"),
            Some(SlashCommand::Clear { confirm: true })
        );
        assert_eq!(
            SlashCommand::parse("/models"),
            Some(SlashCommand::Model { model: None })
        );
        assert_eq!(SlashCommand::parse("/connect"), Some(SlashCommand::Connect));
        assert_eq!(
            SlashCommand::parse("/thinking"),
            Some(SlashCommand::ThinkingToggle)
        );
        assert_eq!(resume_supported_slash_commands().len(), 12);
    }

    #[test]
    fn compacts_sessions_via_slash_command() {
        let session = Session {
            version: 1,
            messages: vec![
                ConversationMessage::user_text("a ".repeat(200)),
                ConversationMessage::assistant(vec![ContentBlock::Text {
                    text: "b ".repeat(200),
                }]),
                ConversationMessage::tool_result("1", "bash", "ok ".repeat(200), false),
                ConversationMessage::assistant(vec![ContentBlock::Text {
                    text: "recent".to_string(),
                }]),
            ],
        };

        let result = handle_slash_command(
            "/compact",
            &session,
            CompactionConfig {
                preserve_recent_messages: 2,
                max_estimated_tokens: 1,
            },
        )
        .expect("slash command should be handled");

        assert!(result.message.contains("Compacted 2 messages"));
        assert_eq!(result.session.messages[0].role, MessageRole::System);
    }

    #[test]
    fn help_command_is_non_mutating() {
        let session = Session::new();
        let result = handle_slash_command("/help", &session, CompactionConfig::default())
            .expect("help command should be handled");
        assert_eq!(result.session, session);
        assert!(result.message.contains("斜杠命令"));
    }

    #[test]
    fn ignores_unknown_or_runtime_bound_slash_commands() {
        let session = Session::new();
        assert!(handle_slash_command("/unknown", &session, CompactionConfig::default()).is_none());
        assert!(handle_slash_command("/status", &session, CompactionConfig::default()).is_none());
        assert!(
            handle_slash_command("/bughunter", &session, CompactionConfig::default()).is_none()
        );
        assert!(handle_slash_command("/commit", &session, CompactionConfig::default()).is_none());
        assert!(handle_slash_command("/pr", &session, CompactionConfig::default()).is_none());
        assert!(handle_slash_command("/issue", &session, CompactionConfig::default()).is_none());
        assert!(
            handle_slash_command("/ultraplan", &session, CompactionConfig::default()).is_none()
        );
        assert!(
            handle_slash_command("/teleport foo", &session, CompactionConfig::default()).is_none()
        );
        assert!(
            handle_slash_command("/debug-tool-call", &session, CompactionConfig::default())
                .is_none()
        );
        assert!(
            handle_slash_command("/model claude", &session, CompactionConfig::default()).is_none()
        );
        assert!(handle_slash_command(
            "/permissions read-only",
            &session,
            CompactionConfig::default()
        )
        .is_none());
        assert!(handle_slash_command("/clear", &session, CompactionConfig::default()).is_none());
        assert!(
            handle_slash_command("/clear --confirm", &session, CompactionConfig::default())
                .is_none()
        );
        assert!(handle_slash_command("/cost", &session, CompactionConfig::default()).is_none());
        assert!(handle_slash_command(
            "/resume session.json",
            &session,
            CompactionConfig::default()
        )
        .is_none());
        assert!(handle_slash_command("/config", &session, CompactionConfig::default()).is_none());
        assert!(
            handle_slash_command("/config env", &session, CompactionConfig::default()).is_none()
        );
        assert!(handle_slash_command("/diff", &session, CompactionConfig::default()).is_none());
        assert!(handle_slash_command("/version", &session, CompactionConfig::default()).is_none());
        assert!(
            handle_slash_command("/export note.txt", &session, CompactionConfig::default())
                .is_none()
        );
        assert!(
            handle_slash_command("/session list", &session, CompactionConfig::default()).is_none()
        );
    }
}
