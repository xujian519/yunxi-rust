//! TUI 内斜杠命令处理（无需离开全屏）。

use std::sync::Arc;

use commands::{render_full_slash_help, resolve_custom_prompt, SlashCommand};
use runtime::CompactionConfig;

use crate::cli_action::{
    normalize_permission_mode, permission_mode_from_label, resolve_model_alias,
};
use crate::format_report::{
    format_compact_report, format_cost_report, format_model_report, format_model_switch_report,
    format_permissions_report, format_permissions_switch_report, format_resume_report,
    format_status_report, git_output, render_config_report, render_connect_report,
    render_conversation_search, render_diff_report, render_export_text,
    render_last_tool_debug_report, render_memory_report, render_session_list,
    render_teleport_report, status_context, undo_last_interaction, StatusUsage,
};
use crate::runtime_bridge::build_runtime;
use crate::session_mgr::{list_managed_sessions, resolve_session_reference};
use crate::slash_sync::{bughunter_prompt, run_commit, run_issue, run_pr, ultraplan_prompt};
use crate::tui::app::TuiApp;
use crate::tui::tool_viz::render_colored_diff;
use crate::VERSION;

use runtime::Session;

use super::init_dispatch::dispatch_init_command;
use super::runner::TuiState;

/// 斜杠命令处理结果。
pub(crate) enum SlashDispatch {
    /// 已处理，无需 LLM 轮次。
    Handled,
    /// 启动一轮 agent 对话（提示词已构造）。
    AgentTurn(String),
}

/// 处理斜杠命令；返回 `Ok(false)` 表示非斜杠输入。
pub(crate) fn handle_slash_command(
    app: &mut TuiApp,
    state: &mut TuiState,
    input: &str,
    width: u16,
    height: u16,
) -> Result<Option<SlashDispatch>, Box<dyn std::error::Error>> {
    let trimmed = input.trim();
    if matches!(trimmed, "/exit" | "/quit") {
        app.push_system_message("使用 q 键或 Ctrl+C 退出 TUI 模式。");
        return Ok(Some(SlashDispatch::Handled));
    }
    if trimmed == "/init" {
        return Ok(Some(dispatch_init_command(app, state, width, height)?));
    }

    if input.trim() == "/semantic" {
        let section = crate::routing::format_athena_status_section(
            state.last_routing.as_ref(),
            state.allowed_tools.as_ref(),
        );
        app.push_output("Semantic", &section, width, height);
        return Ok(Some(SlashDispatch::Handled));
    }

    if let Some(rest) = input.strip_prefix("/flow").map(str::trim) {
        let parts: Vec<&str> = rest.split_whitespace().collect();
        let output = match parts.as_slice() {
            [] | ["list"] | ["status" | "ls"] => tools::execute_tool(
                "FlowTool",
                &serde_json::json!({ "operation": "list_suspended" }),
            ),
            ["resume", flow_id, run_id] | ["continue", flow_id, run_id] => {
                let result = crate::session_meta::execute_flow_resume(flow_id, run_id, true);
                if result.is_ok() {
                    state.suspended_flows.retain(|f| {
                        !(f.flow_id == *flow_id && f.run_id == *run_id)
                    });
                    app.set_pending_flow_hitl(None);
                    let _ = state.persist_session(app);
                    refresh_status(app, state);
                }
                result
            }
            ["clear" | "clean"] => {
                let count = state.suspended_flows.len();
                state.suspended_flows.clear();
                app.set_pending_flow_hitl(None);
                let _ = state.persist_session(app);
                Ok(format!("已清除 {count} 个挂起的流程。"))
            }
            _ => Ok(
                "用法:\n  /flow list          列出挂起的流程\n  /flow resume <id1> <id2>  恢复指定流程\n  /flow clear         清除所有挂起流程\n\n  示例: /flow resume patent-review run-001"
                    .to_string(),
            ),
        };
        app.push_output("Flow", &output.unwrap_or_else(|e| e), width, height);
        return Ok(Some(SlashDispatch::Handled));
    }

    if let Some(rest) = input.strip_prefix("/route").map(str::trim) {
        let query = if rest.is_empty() {
            "专利新颖性创造性分析"
        } else {
            rest
        };
        let decision = crate::routing::route(query);
        state.last_route_hint = Some(crate::routing::format_route_label(&decision));
        state.last_routing = Some(crate::routing::RoutingSnapshot::from_decision(&decision));
        let json = serde_json::to_string_pretty(&crate::routing::route_debug_json(query))
            .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
        app.push_output("Route", &json, width, height);
        refresh_status(app, state);
        return Ok(Some(SlashDispatch::Handled));
    }

    let Some(command) = SlashCommand::parse(input) else {
        return Ok(None);
    };

    let dispatch = match command {
        SlashCommand::Help => {
            app.push_output("帮助", &render_full_slash_help(), width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Status => {
            let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
            let cumulative = runtime.usage().cumulative_usage();
            let latest = runtime.usage().current_turn_usage();
            let mut report = format_status_report(
                app.model(),
                StatusUsage {
                    message_count: runtime.session().messages.len(),
                    turns: runtime.usage().turns(),
                    latest,
                    cumulative,
                    estimated_tokens: runtime.estimated_tokens(),
                },
                state.permission_mode.as_str(),
                &status_context(Some(&state.session_handle.path))?,
            );
            report.push_str("\n\n");
            report.push_str(&crate::routing::format_athena_status_section(
                state.last_routing.as_ref(),
                state.allowed_tools.as_ref(),
            ));
            if let Some(ref decision) = state.last_auto_decision {
                report.push_str(&format!("\n\n模型路由\n  上次决策  {decision}"));
            }
            if !state.suspended_flows.is_empty() {
                report.push_str("\n\n挂起工作流\n");
                for f in &state.suspended_flows {
                    report.push_str(&format!(
                        "  flow_id={} run_id={}\n  恢复: /flow resume {} {}\n",
                        f.flow_id, f.run_id, f.flow_id, f.run_id
                    ));
                }
            }
            drop(runtime);
            app.push_output("Status", &report, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Model { model } => {
            if let Some(model) = model {
                switch_model(app, state, &model)?;
            } else {
                let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
                let report = format_model_report(
                    app.model(),
                    runtime.session().messages.len(),
                    runtime.usage().turns(),
                );
                drop(runtime);
                app.push_output("Model", &report, width, height);
            }
            SlashDispatch::Handled
        }
        SlashCommand::Permissions { mode } => {
            if let Some(mode) = mode {
                switch_permissions(app, state, &mode)?;
            } else {
                app.push_output(
                    "Permissions",
                    &format_permissions_report(state.permission_mode.as_str()),
                    width,
                    height,
                );
            }
            SlashDispatch::Handled
        }
        SlashCommand::Cost => {
            let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
            let cumulative = runtime.usage().cumulative_usage();
            drop(runtime);
            app.push_output("Cost", &format_cost_report(cumulative), width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Version => {
            app.push_system_message(&format!("云熙智能体 v{VERSION}"));
            SlashDispatch::Handled
        }
        SlashCommand::Compact => {
            compact_session(app, state)?;
            SlashDispatch::Handled
        }
        SlashCommand::Diff => {
            let report = render_diff_report()?;
            let colored = if let Some(diff) = report.strip_prefix("Diff\n\n") {
                format!("Diff\n\n{}", render_colored_diff(diff))
            } else {
                report
            };
            app.push_output("Diff", &colored, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Session { action, target } => {
            handle_session_command(
                app,
                state,
                action.as_deref(),
                target.as_deref(),
                width,
                height,
            )?;
            SlashDispatch::Handled
        }
        SlashCommand::Config { section } => {
            app.push_output(
                "Config",
                &render_config_report(section.as_deref())?,
                width,
                height,
            );
            SlashDispatch::Handled
        }
        SlashCommand::Memory => {
            app.push_output("Memory", &render_memory_report()?, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Export { path } => {
            let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
            let export_path =
                crate::format_report::resolve_export_path(path.as_deref(), runtime.session())?;
            let text = render_export_text(runtime.session());
            let message_count = runtime.session().messages.len();
            drop(runtime);
            std::fs::write(&export_path, text)?;
            app.push_system_message(&format!(
                "Export\n  Result           wrote transcript\n  File             {}\n  Messages         {message_count}",
                export_path.display()
            ));
            SlashDispatch::Handled
        }
        SlashCommand::Clear { confirm } => {
            if confirm {
                state
                    .runtime
                    .lock()
                    .map_err(|_| "runtime lock poisoned")?
                    .session_mut()
                    .messages
                    .clear();
                app.clear_chat();
                state.persist_session(app)?;
                app.push_system_message("已新建会话：对话区与历史消息已清空。");
            } else {
                app.push_system_message("清空会话请使用 /new 或 /clear --confirm");
            }
            SlashDispatch::Handled
        }
        SlashCommand::Search { query } => {
            let Some(query) = query else {
                app.push_system_message("用法: /search <关键词>");
                return Ok(Some(SlashDispatch::Handled));
            };
            let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
            let report = render_conversation_search(runtime.session(), &query);
            drop(runtime);
            app.push_output("Search", &report, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Undo => {
            let mut runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
            let report = undo_last_interaction(runtime.session_mut())?;
            drop(runtime);
            state.persist_session(app)?;
            app.push_system_message(&report);
            SlashDispatch::Handled
        }
        SlashCommand::Resume { session_path } => {
            resume_session(app, state, session_path.as_deref())?;
            SlashDispatch::Handled
        }
        SlashCommand::Init => dispatch_init_command(app, state, width, height)?,
        SlashCommand::Teleport { target } => {
            let Some(target) = target.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
                app.push_system_message("Usage: /teleport <symbol-or-path>");
                return Ok(Some(SlashDispatch::Handled));
            };
            app.push_output("Teleport", &render_teleport_report(target)?, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::DebugToolCall => {
            let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
            let report = render_last_tool_debug_report(runtime.session())?;
            drop(runtime);
            app.push_output("Debug", &report, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Bughunter { scope } => {
            app.push_system_message("运行 /bughunter …");
            SlashDispatch::AgentTurn(bughunter_prompt(scope.as_deref()))
        }
        SlashCommand::Ultraplan { task } => {
            app.push_system_message("运行 /ultraplan …");
            SlashDispatch::AgentTurn(ultraplan_prompt(task.as_deref()))
        }
        SlashCommand::Commit => {
            app.push_system_message("正在生成提交…");
            app.set_thinking(true);
            let report = run_commit(
                Arc::clone(&state.runtime),
                app.model().to_string(),
                state.system_prompt.clone(),
                state.allowed_tools.clone(),
                state.permission_mode,
            )?;
            app.set_thinking(false);
            app.push_output("Commit", &report, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Pr { context } => {
            app.push_system_message("正在生成 PR…");
            app.set_thinking(true);
            let report = run_pr(
                Arc::clone(&state.runtime),
                app.model().to_string(),
                state.system_prompt.clone(),
                state.allowed_tools.clone(),
                state.permission_mode,
                context.as_deref(),
            )?;
            app.set_thinking(false);
            app.push_output("PR", &report, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Issue { context } => {
            app.push_system_message("正在生成 Issue…");
            app.set_thinking(true);
            let report = run_issue(
                Arc::clone(&state.runtime),
                app.model().to_string(),
                state.system_prompt.clone(),
                state.allowed_tools.clone(),
                state.permission_mode,
                context.as_deref(),
            )?;
            app.set_thinking(false);
            app.push_output("Issue", &report, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::Connect => {
            app.push_output("Connect", &render_connect_report()?, width, height);
            SlashDispatch::Handled
        }
        SlashCommand::ThinkingToggle => {
            let on = app.toggle_show_reasoning();
            app.push_system_message(&format!(
                "推理过程显示: {}",
                if on { "开启" } else { "关闭" }
            ));
            SlashDispatch::Handled
        }
        SlashCommand::Custom { name, arguments } => {
            let Some(prompt) = resolve_custom_prompt(&name, arguments.as_deref()) else {
                app.push_system_message(&format!("自定义命令 /{name} 未找到。"));
                return Ok(Some(SlashDispatch::Handled));
            };
            app.push_system_message(&format!("运行自定义命令 /{name} …"));
            SlashDispatch::AgentTurn(prompt)
        }
        SlashCommand::Unknown(name) => {
            app.push_system_message(&format!("未知命令: /{name}，输入 /help 查看帮助。"));
            SlashDispatch::Handled
        }
    };

    refresh_status(app, state);
    Ok(Some(dispatch))
}

fn switch_model(
    app: &mut TuiApp,
    state: &mut TuiState,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let model = resolve_model_alias(model).to_string();
    if model == app.model() {
        let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
        let report = format_model_report(
            app.model(),
            runtime.session().messages.len(),
            runtime.usage().turns(),
        );
        drop(runtime);
        app.push_system_message(&report);
        return Ok(());
    }

    let previous = app.model().to_string();
    let session = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned")?
        .session()
        .clone();
    let message_count = session.messages.len();
    let new_runtime = build_runtime(
        session,
        model.clone(),
        state.system_prompt.clone(),
        true,
        false,
        state.allowed_tools.clone(),
        state.permission_mode,
    )?;
    *state.runtime.lock().map_err(|_| "runtime lock poisoned")? = new_runtime;
    app.set_model(model.clone());
    if model == "auto" {
        state.active_model = None;
        state.last_auto_decision = None;
    } else {
        state.active_model = Some(model.clone());
        state.last_auto_decision = None;
    }
    app.push_system_message(&format_model_switch_report(
        &previous,
        &model,
        message_count,
    ));
    Ok(())
}

fn switch_permissions(
    app: &mut TuiApp,
    state: &mut TuiState,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let normalized = normalize_permission_mode(mode).ok_or_else(|| {
        format!(
            "unsupported permission mode '{mode}'. Use read-only, workspace-write, or danger-full-access."
        )
    })?;

    if normalized == state.permission_mode.as_str() {
        app.push_system_message(&format_permissions_report(normalized));
        return Ok(());
    }

    let previous = state.permission_mode.as_str().to_string();
    let session = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned")?
        .session()
        .clone();
    state.permission_mode = permission_mode_from_label(normalized);
    let new_runtime = build_runtime(
        session,
        app.model().to_string(),
        state.system_prompt.clone(),
        true,
        false,
        state.allowed_tools.clone(),
        state.permission_mode,
    )?;
    *state.runtime.lock().map_err(|_| "runtime lock poisoned")? = new_runtime;
    app.push_system_message(&format_permissions_switch_report(&previous, normalized));
    Ok(())
}

fn compact_session(
    app: &mut TuiApp,
    state: &mut TuiState,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = state
        .runtime
        .lock()
        .map_err(|_| "runtime lock poisoned")?
        .compact(CompactionConfig::default());
    let removed = result.removed_message_count;
    let kept = result.compacted_session.messages.len();
    let skipped = removed == 0;
    let new_runtime = build_runtime(
        result.compacted_session,
        app.model().to_string(),
        state.system_prompt.clone(),
        true,
        false,
        state.allowed_tools.clone(),
        state.permission_mode,
    )?;
    *state.runtime.lock().map_err(|_| "runtime lock poisoned")? = new_runtime;
    state.persist_session(app)?;
    app.push_system_message(&format_compact_report(removed, kept, skipped));
    Ok(())
}

fn resume_session(
    app: &mut TuiApp,
    state: &mut TuiState,
    session_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(session_ref) = session_path else {
        app.push_system_message("Usage: /resume <session-path>");
        return Ok(());
    };
    let handle = resolve_session_reference(session_ref)?;
    let session = Session::load_from_path(&handle.path)?;
    let message_count = session.messages.len();
    let turns = {
        let runtime = state.runtime.lock().map_err(|_| "runtime lock poisoned")?;
        runtime.usage().turns()
    };
    let new_runtime = build_runtime(
        session,
        app.model().to_string(),
        state.system_prompt.clone(),
        true,
        false,
        state.allowed_tools.clone(),
        state.permission_mode,
    )?;
    *state.runtime.lock().map_err(|_| "runtime lock poisoned")? = new_runtime;
    state.session_handle = handle;
    app.push_system_message(&format_resume_report(
        &state.session_handle.path.display().to_string(),
        message_count,
        turns,
    ));
    Ok(())
}

pub(crate) fn refresh_status(app: &mut TuiApp, state: &TuiState) {
    let Ok(runtime) = state.runtime.lock() else {
        return;
    };
    let cumulative = runtime.usage().cumulative_usage();
    let latest = runtime.usage().current_turn_usage();
    let cost = cumulative.estimate_cost_usd().total_cost_usd();
    let turn_elapsed_secs = state
        .turn_started_at
        .map(|start| start.elapsed().as_secs_f64());
    app.update_status(crate::tui::status_bar::StatusBarSnapshot {
        model: app.model().to_string(),
        auto_decision: state.last_auto_decision.clone(),
        permission_mode: state.permission_mode.as_str().to_string(),
        session_id: state.session_handle.id.clone(),
        cumulative_input_tokens: u64::from(cumulative.input_tokens)
            + u64::from(latest.input_tokens),
        cumulative_output_tokens: u64::from(cumulative.output_tokens)
            + u64::from(latest.output_tokens),
        estimated_cost_usd: cost,
        git_branch: git_output(&["symbolic-ref", "--short", "HEAD"])
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        thinking: app.is_thinking(),
        turn_elapsed_secs: if app.is_thinking() {
            turn_elapsed_secs
        } else {
            None
        },
        active_tool: app.active_tool.clone(),
        turn_output_tokens: app.turn_progress().0,
        turn_output_max: app.turn_progress().1,
        route_hint: state.last_route_hint.clone(),
        semantic_on: embedding::semantic_enabled(),
        patent_case_hint: None,
        flow_hitl_hint: state.athena_meta().pending_flow_label(),
    });
}

fn handle_session_command(
    app: &mut TuiApp,
    state: &mut TuiState,
    action: Option<&str>,
    target: Option<&str>,
    width: u16,
    height: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        None => {
            let sessions = list_managed_sessions()?;
            app.open_session_picker(sessions, state.session_handle.id.clone());
        }
        Some("list") => {
            app.push_output(
                "Sessions",
                &render_session_list(&state.session_handle.id)?,
                width,
                height,
            );
        }
        Some("switch") => {
            let Some(target) = target else {
                let sessions = list_managed_sessions()?;
                app.open_session_picker(sessions, state.session_handle.id.clone());
                return Ok(());
            };
            apply_session_switch(app, state, target)?;
        }
        Some(other) => {
            app.push_system_message(&format!(
                "Unknown /session action '{other}'. Use /session, /session list, or /session switch <id>."
            ));
        }
    }
    Ok(())
}

/// 切换到指定会话（ID 或路径）。
pub(crate) fn apply_session_switch(
    app: &mut TuiApp,
    state: &mut TuiState,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let handle = resolve_session_reference(target)?;
    let session = Session::load_from_path(&handle.path)?;
    let message_count = session.messages.len();
    let new_runtime = build_runtime(
        session,
        app.model().to_string(),
        state.system_prompt.clone(),
        true,
        false,
        state.allowed_tools.clone(),
        state.permission_mode,
    )?;
    *state.runtime.lock().map_err(|_| "runtime lock poisoned")? = new_runtime;
    state.session_handle = handle;
    let athena = crate::session_meta::load_athena_meta(&state.session_handle.path);
    state.apply_athena_meta(&athena);
    app.push_system_message(&format!(
        "Session switched\n  Active session   {}\n  File             {}\n  Messages         {message_count}{}",
        state.session_handle.id,
        state.session_handle.path.display(),
        if athena.suspended_flows.is_empty() {
            String::new()
        } else {
            format!(
                "\n  挂起工作流       {} 个（/flow list）",
                athena.suspended_flows.len()
            )
        }
    ));
    refresh_status(app, state);
    Ok(())
}
