//! 桌面端斜杠命令处理（与 TUI/REPL 对齐）。

use std::path::PathBuf;

use commands::{render_full_slash_help, resolve_custom_prompt, SlashCommand};
use runtime::{
    compact_session, estimate_session_tokens, CompactionConfig, ConfigLoader, PermissionMode,
    Session, TokenUsage,
};
use serde::Serialize;

use crate::cli_action::{normalize_permission_mode, resolve_model_alias};
use crate::format_report::{
    format_compact_report, format_cost_report, format_model_report, format_model_switch_report,
    format_permissions_report, format_permissions_switch_report, format_resume_report,
    format_status_report, init_claude_md, render_config_report, render_connect_report,
    render_conversation_search, render_diff_report, render_export_text, render_memory_report,
    render_session_list, render_teleport_report, render_last_tool_debug_report,
    resolve_export_path, status_context, undo_last_interaction, StatusUsage,
};
use crate::init::initialize_repo;
use crate::routing;
use crate::runtime_bridge::build_system_prompt_for;
use crate::session_mgr::{resolve_session_reference};
use crate::slash_sync::{
    bughunter_prompt, run_commit_for_session, run_issue_for_session, run_pr_for_session,
    ultraplan_prompt,
};
use crate::{doctor, VERSION};

/// 斜杠命令执行上下文。
pub struct DesktopSlashContext {
    pub session_id: String,
    pub session_path: PathBuf,
    pub model: String,
    pub permission_mode: PermissionMode,
    pub workspace_root: PathBuf,
}

/// 斜杠命令执行结果。
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SlashExecuteResult {
    /// 直接返回 Markdown 文本。
    Message { content: String },
    /// 需要启动 Agent 轮次。
    AgentTurn { prompt: String },
    /// 会话已变更，需重新加载。
    SessionUpdated {
        content: String,
        session_json: String,
    },
}

/// 执行斜杠命令；非斜杠输入返回 `None`。
pub fn execute_desktop_slash(
    input: &str,
    ctx: &DesktopSlashContext,
) -> Result<Option<SlashExecuteResult>, String> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return Ok(None);
    }

    if matches!(trimmed, "/exit" | "/quit") {
        return Ok(Some(SlashExecuteResult::Message {
            content: "桌面端请使用窗口关闭按钮退出。".to_string(),
        }));
    }

    if trimmed == "/doctor" {
        let report = doctor::collect_doctor_report();
        let lines: Vec<String> = report
            .checks
            .iter()
            .map(|c| {
                let icon = match c.status.as_str() {
                    "fail" => "✗",
                    "warn" => "!",
                    _ => "✓",
                };
                format!("{icon} **{}** — {}", c.name, c.detail)
            })
            .collect();
        return Ok(Some(SlashExecuteResult::Message {
            content: format!(
                "**环境检查 (doctor)**\n\n{}\n\n**结果：** {}",
                lines.join("\n"),
                report.summary
            ),
        }));
    }

    if let Some(rest) = trimmed.strip_prefix("/route") {
        let query = rest.trim();
        let query = if query.is_empty() {
            "专利新颖性创造性分析"
        } else {
            query
        };
        let decision = routing::route(query);
        let json = serde_json::to_string_pretty(&routing::route_debug_json(query))
            .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
        return Ok(Some(SlashExecuteResult::Message {
            content: format!(
                "**路由决策**\n\n- 领域：{}\n- 复杂度：{}\n- 意图：{}\n- 置信度：{:.0}%\n\n```json\n{}\n```",
                decision.domain,
                decision.complexity,
                decision.intent_name,
                decision.confidence * 100.0,
                json
            ),
        }));
    }

    if let Some(rest) = trimmed.strip_prefix("/search") {
        let query = rest.trim();
        if query.is_empty() {
            return Ok(Some(SlashExecuteResult::Message {
                content: "用法：`/search <关键词>`".to_string(),
            }));
        }
        let session = Session::load_from_path(&ctx.session_path).map_err(|e| e.to_string())?;
        return Ok(Some(SlashExecuteResult::Message {
            content: render_conversation_search(&session, query),
        }));
    }

    let Some(command) = SlashCommand::parse(input) else {
        return Ok(None);
    };

    let mut session = Session::load_from_path(&ctx.session_path).map_err(|e| e.to_string())?;

    let result = match command {
        SlashCommand::Help => SlashExecuteResult::Message {
            content: render_full_slash_help(),
        },
        SlashCommand::Status => {
            let cumulative = aggregate_session_usage(&session);
            let report = format_status_report(
                &ctx.model,
                StatusUsage {
                    message_count: session.messages.len(),
                    turns: count_assistant_turns(&session),
                    latest: cumulative,
                    cumulative,
                    estimated_tokens: estimate_session_tokens(&session),
                },
                ctx.permission_mode.as_str(),
                &status_context(Some(&ctx.session_path)).map_err(|e| e.to_string())?,
            );
            SlashExecuteResult::Message { content: report }
        }
        SlashCommand::Model { model } => {
            if let Some(model) = model {
                let resolved = resolve_model_alias(&model).to_string();
                SlashExecuteResult::Message {
                    content: format_model_switch_report(&ctx.model, &resolved, session.messages.len()),
                }
            } else {
                SlashExecuteResult::Message {
                    content: format_model_report(
                        &ctx.model,
                        session.messages.len(),
                        count_assistant_turns(&session),
                    ),
                }
            }
        }
        SlashCommand::Permissions { mode } => {
            if let Some(mode) = mode {
                let normalized = normalize_permission_mode(&mode)
                    .ok_or_else(|| format!("unknown permission mode: {mode}"))?;
                SlashExecuteResult::Message {
                    content: format_permissions_switch_report(
                        ctx.permission_mode.as_str(),
                        normalized,
                    ),
                }
            } else {
                SlashExecuteResult::Message {
                    content: format_permissions_report(ctx.permission_mode.as_str()),
                }
            }
        }
        SlashCommand::Cost => SlashExecuteResult::Message {
            content: format_cost_report(aggregate_session_usage(&session)),
        },
        SlashCommand::Version => SlashExecuteResult::Message {
            content: format!("云熙智能体 v{VERSION}"),
        },
        SlashCommand::Compact => {
            let compacted = compact_session(&session, CompactionConfig::default());
            let removed = compacted.removed_message_count;
            let kept = compacted.compacted_session.messages.len();
            let skipped = removed == 0;
            session = compacted.compacted_session;
            save_session(&ctx.session_path, &session)?;
            SlashExecuteResult::SessionUpdated {
                content: format_compact_report(removed, kept, skipped),
                session_json: session.to_json().render(),
            }
        }
        SlashCommand::Diff => SlashExecuteResult::Message {
            content: render_diff_report().map_err(|e| e.to_string())?,
        },
        SlashCommand::Config { section } => SlashExecuteResult::Message {
            content: render_config_report(section.as_deref()).map_err(|e| e.to_string())?,
        },
        SlashCommand::Memory => SlashExecuteResult::Message {
            content: render_memory_report().map_err(|e| e.to_string())?,
        },
        SlashCommand::Export { path } => {
            let export_path = resolve_export_path(path.as_deref(), &session)
                .map_err(|e| e.to_string())?;
            let text = render_export_text(&session);
            let count = session.messages.len();
            std::fs::write(&export_path, text).map_err(|e| e.to_string())?;
            SlashExecuteResult::Message {
                content: format!(
                    "**Export**\n- 文件：`{}`\n- 消息数：{count}",
                    export_path.display()
                ),
            }
        }
        SlashCommand::Clear { confirm } => {
            if confirm {
                session.messages.clear();
                save_session(&ctx.session_path, &session)?;
                SlashExecuteResult::SessionUpdated {
                    content: "已清空会话消息。".to_string(),
                    session_json: session.to_json().render(),
                }
            } else {
                SlashExecuteResult::Message {
                    content: "清空会话请使用 `/new` 或 `/clear --confirm`。".to_string(),
                }
            }
        }
        SlashCommand::Undo => {
            let report = undo_last_interaction(&mut session).map_err(|e| e.to_string())?;
            save_session(&ctx.session_path, &session)?;
            SlashExecuteResult::SessionUpdated {
                content: report,
                session_json: session.to_json().render(),
            }
        }
        SlashCommand::Session { action, target } => {
            let content = handle_session_subcommand(&ctx.session_id, action.as_deref(), target.as_deref())?;
            SlashExecuteResult::Message { content }
        }
        SlashCommand::Resume { session_path } => {
            let Some(ref_path) = session_path else {
                return Ok(Some(SlashExecuteResult::Message {
                    content: "用法：`/resume <session-id>`".to_string(),
                }));
            };
            let handle = resolve_session_reference(&ref_path).map_err(|e| e.to_string())?;
            let loaded = Session::load_from_path(&handle.path).map_err(|e| e.to_string())?;
            let count = loaded.messages.len();
            save_session(&ctx.session_path, &loaded)?;
            SlashExecuteResult::SessionUpdated {
                content: format_resume_report(&handle.path.display().to_string(), count, 0),
                session_json: loaded.to_json().render(),
            }
        }
        SlashCommand::Init => {
            let report = initialize_repo(&ctx.workspace_root).map_err(|e| e.to_string())?;
            SlashExecuteResult::Message {
                content: report.render(),
            }
        }
        SlashCommand::Teleport { target } => {
            let Some(target) = target.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
                return Ok(Some(SlashExecuteResult::Message {
                    content: "用法：`/teleport <symbol-or-path>`".to_string(),
                }));
            };
            SlashExecuteResult::Message {
                content: render_teleport_report(target).map_err(|e| e.to_string())?,
            }
        }
        SlashCommand::DebugToolCall => SlashExecuteResult::Message {
            content: render_last_tool_debug_report(&session).map_err(|e| e.to_string())?,
        },
        SlashCommand::Bughunter { scope } => SlashExecuteResult::AgentTurn {
            prompt: bughunter_prompt(scope.as_deref()),
        },
        SlashCommand::Ultraplan { task } => SlashExecuteResult::AgentTurn {
            prompt: ultraplan_prompt(task.as_deref()),
        },
        SlashCommand::Connect => SlashExecuteResult::Message {
            content: render_connect_report().map_err(|e| e.to_string())?,
        },
        SlashCommand::ThinkingToggle => SlashExecuteResult::Message {
            content: "桌面端推理块显示请在聊天设置中切换。".to_string(),
        },
        SlashCommand::Custom { name, arguments } => {
            let Some(prompt) = resolve_custom_prompt(&name, arguments.as_deref()) else {
                return Ok(Some(SlashExecuteResult::Message {
                    content: format!("自定义命令 `/{name}` 未找到。"),
                }));
            };
            SlashExecuteResult::AgentTurn { prompt }
        }
        SlashCommand::Commit => {
            let system_prompt =
                build_system_prompt_for(ctx.workspace_root.clone()).map_err(|e| e.to_string())?;
            let report = run_commit_for_session(
                session,
                ctx.model.clone(),
                system_prompt,
                ctx.workspace_root.clone(),
                ctx.permission_mode,
            )
            .map_err(|e| e.to_string())?;
            SlashExecuteResult::Message { content: report }
        }
        SlashCommand::Pr { context } => {
            let system_prompt =
                build_system_prompt_for(ctx.workspace_root.clone()).map_err(|e| e.to_string())?;
            let report = run_pr_for_session(
                session,
                ctx.model.clone(),
                system_prompt,
                ctx.workspace_root.clone(),
                ctx.permission_mode,
                context.as_deref(),
            )
            .map_err(|e| e.to_string())?;
            SlashExecuteResult::Message { content: report }
        }
        SlashCommand::Issue { context } => {
            let system_prompt =
                build_system_prompt_for(ctx.workspace_root.clone()).map_err(|e| e.to_string())?;
            let report = run_issue_for_session(
                session,
                ctx.model.clone(),
                system_prompt,
                ctx.workspace_root.clone(),
                ctx.permission_mode,
                context.as_deref(),
            )
            .map_err(|e| e.to_string())?;
            SlashExecuteResult::Message { content: report }
        }
        SlashCommand::Unknown(name) => SlashExecuteResult::Message {
            content: format!("未知命令 `/{name}`，输入 `/help` 查看帮助。"),
        },
        SlashCommand::Search { .. } => unreachable!("handled above"),
    };

    Ok(Some(result))
}

fn save_session(path: &PathBuf, session: &Session) -> Result<(), String> {
    session.save_to_path(path).map_err(|e| e.to_string())
}

fn handle_session_subcommand(
    active_session_id: &str,
    action: Option<&str>,
    target: Option<&str>,
) -> Result<String, String> {
    let action = action.unwrap_or("list");
    match action {
        "list" | "ls" => render_session_list(active_session_id).map_err(|e| e.to_string()),
        "switch" => {
            let Some(target) = target else {
                return Ok("用法：`/session switch <id>`".to_string());
            };
            let handle = resolve_session_reference(target).map_err(|e| e.to_string())?;
            Ok(format!(
                "切换到会话 `{}`（{}）",
                handle.id,
                handle.path.display()
            ))
        }
        _ => Ok(format!(
            "用法：\n- `/session list`\n- `/session switch <id>`\n\n未知子命令：{action}"
        )),
    }
}

/// 初始化当前工作区（/init 的 IPC 版本）。
pub fn run_workspace_init(workspace_root: &PathBuf) -> Result<String, String> {
    initialize_repo(workspace_root)
        .map(|r| r.render())
        .map_err(|e| e.to_string())
}

/// 生成 CLAUDE.md 引导（与 CLI init 子命令一致）。
pub fn run_init_claude_md() -> Result<String, String> {
    init_claude_md().map_err(|e| e.to_string())
}

/// 加载 MCP 配置状态。
pub fn load_mcp_status(workspace_root: &PathBuf) -> Result<crate::mcp_runtime::McpStatusReport, String> {
    let config = ConfigLoader::default_for(workspace_root)
        .load()
        .map_err(|e| e.to_string())?;
    Ok(crate::mcp_runtime::mcp_config_status(&config))
}

fn aggregate_session_usage(session: &Session) -> TokenUsage {
    session.messages.iter().fold(TokenUsage::default(), |acc, msg| {
        let Some(u) = msg.usage else {
            return acc;
        };
        TokenUsage {
            input_tokens: acc.input_tokens.saturating_add(u.input_tokens),
            output_tokens: acc.output_tokens.saturating_add(u.output_tokens),
            cache_creation_input_tokens: acc
                .cache_creation_input_tokens
                .saturating_add(u.cache_creation_input_tokens),
            cache_read_input_tokens: acc
                .cache_read_input_tokens
                .saturating_add(u.cache_read_input_tokens),
        }
    })
}

fn count_assistant_turns(session: &Session) -> u32 {
    session
        .messages
        .iter()
        .filter(|m| matches!(m.role, runtime::MessageRole::Assistant))
        .count() as u32
}
