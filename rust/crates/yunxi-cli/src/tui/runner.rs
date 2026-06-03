//! TUI 全屏 REPL（通用对话布局）。

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossterm::event::{
    self, Event, KeyCode, KeyEvent as CrosstermKey, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};

use crate::cli_action::AllowedToolSet;
use crate::model_routing::{load_router_config, select_model_for_request};
use crate::runtime_bridge::{build_runtime, build_system_prompt};
use crate::session_meta::{execute_flow_resume, load_athena_meta, AthenaSessionMeta};
use crate::session_mgr::create_managed_session_handle;
use crate::session_mgr::list_managed_sessions;
use crate::tui::app::{KeyEvent, MouseAction, TuiAction, TuiApp};
use crate::tui::hitl::{
    close_human_guide, ingest_flow_tool_result, open_human_guide, sync_pending_flow_overlay,
    wrap_human_intervention,
};
use crate::tui::slash::{handle_slash_command, refresh_status, SlashDispatch};
use crate::tui::turn::{spawn_turn, ActiveTurn, TurnEvent};
use crate::VERSION;

use crate::llm_auth::{format_llm_error, missing_api_key_message};
use crate::routing::RoutingSnapshot;
use runtime::{PermissionMode, Session};

/// 启动 TUI 全屏 REPL。
pub(crate) fn run_tui_repl(
    model: String,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    resume_session: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let system_prompt = build_system_prompt()?;
    let session_handle = create_managed_session_handle()?;
    let session = match &resume_session {
        Some(path) => match Session::load_from_path(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("警告: 无法加载会话 {}: {e}，将使用新会话", path.display());
                Session::new()
            }
        },
        None => Session::new(),
    };
    let runtime = Arc::new(Mutex::new(build_runtime(
        session,
        model.clone(),
        system_prompt.clone(),
        true,
        false,
        allowed_tools.clone(),
        permission_mode,
    )?));

    let athena = load_athena_meta(&session_handle.path);
    let mut state = TuiState {
        runtime,
        session_handle,
        system_prompt,
        allowed_tools,
        permission_mode,
        last_routing: athena.last_routing,
        suspended_flows: athena.suspended_flows,
        last_route_hint: None,
        active_model: None,
        last_auto_decision: None,
        turn_started_at: None,
    };

    let mut app = TuiApp::new(model, VERSION.to_string());
    sync_pending_flow_overlay(&mut app, &mut state);

    let cwd = std::env::current_dir()
        .map_or_else(|_| "<unknown>".to_string(), |p| p.display().to_string());
    let banner = crate::tui::banner::render_banner(
        &app.model,
        state.permission_mode.as_str(),
        &cwd,
        &state.session_handle.id,
    );
    app.push_system_message(&banner);
    app.push_system_message("\x1b[2mF3/Ctrl+P 命令 · Ctrl+B 工具面板 · Ctrl+D 主题 · Ctrl+G 引导 · Ctrl+I 中断\x1b[0m");

    refresh_status(&mut app, &state);

    let mut terminal = crate::tui::terminal::TuiTerminal::setup()?;

    let result = run_event_loop(&mut app, &mut state, &mut terminal.terminal);

    terminal.restore()?;

    state.persist_session(&app)?;

    result
}

/// TUI 运行时状态。
pub(crate) struct TuiState {
    pub runtime: crate::tui::turn::SharedRuntime,
    pub session_handle: crate::session_mgr::SessionHandle,
    pub system_prompt: Vec<String>,
    pub allowed_tools: Option<AllowedToolSet>,
    pub permission_mode: PermissionMode,
    pub last_routing: Option<RoutingSnapshot>,
    pub suspended_flows: Vec<crate::session_meta::SuspendedFlowRecord>,
    pub last_route_hint: Option<String>,
    pub(crate) active_model: Option<String>,
    pub(crate) last_auto_decision: Option<String>,
    pub(crate) turn_started_at: Option<Instant>,
}

impl TuiState {
    #[must_use]
    pub fn athena_meta(&self) -> AthenaSessionMeta {
        AthenaSessionMeta {
            last_routing: self.last_routing.clone(),
            suspended_flows: self.suspended_flows.clone(),
        }
    }

    pub fn apply_athena_meta(&mut self, meta: &AthenaSessionMeta) {
        self.last_routing = meta.last_routing.clone();
        self.suspended_flows = meta.suspended_flows.clone();
    }

    pub fn persist_session(&mut self, app: &TuiApp) -> Result<(), Box<dyn std::error::Error>> {
        let session = self
            .runtime
            .lock()
            .map_err(|_| "runtime lock poisoned")?
            .session()
            .clone();
        crate::session_meta::merge_save_session(
            &self.session_handle.path,
            &session,
            &self.athena_meta(),
        )?;
        let _ = app;
        Ok(())
    }
}

fn run_event_loop(
    app: &mut TuiApp,
    state: &mut TuiState,
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut active_turn: Option<ActiveTurn> = None;
    let mut needs_render = true;

    loop {
        let mut state_changed = false;

        if let Some(turn) = active_turn.as_mut() {
            for event in turn.poll() {
                state_changed = true;
                match event {
                    TurnEvent::Stream(stream_event) => {
                        app.append_stream_event(&stream_event);
                    }
                    TurnEvent::ToolUse { name } => {
                        app.active_tool = Some(name);
                    }
                    TurnEvent::Permission(req) => {
                        app.set_pending_permission(Some(req));
                    }
                    TurnEvent::Done(result) => {
                        let streamed = app.turn_was_streamed();
                        app.end_turn_stream();
                        if !turn.is_cancelled() {
                            match result {
                                Ok(summary) => {
                                    ingest_turn_summary(app, state, &summary, streamed);
                                }
                                Err(error) => {
                                    let effective_model =
                                        state.active_model.as_deref().unwrap_or(app.model());
                                    let msg = format_llm_error(effective_model, &error);
                                    if app.last_assistant_text_is_empty() {
                                        app.set_last_assistant_text(msg);
                                    } else {
                                        app.push_assistant_message(&msg);
                                    }
                                }
                            }
                        } else {
                            app.push_system_message(
                                "\x1b[2m已中断上一轮；新指示已发送或可在底栏继续编辑。\x1b[0m",
                            );
                        }
                        app.set_thinking(false);
                        app.active_tool = None;
                        app.set_pending_permission(None);
                        state.turn_started_at = None;
                        refresh_status(app, state);
                        let _ = state.persist_session(app);
                    }
                }
            }
            if turn.is_finished() {
                let finished = active_turn.take().expect("turn");
                finished.join();
            }
        }

        if app.is_thinking() {
            app.tick_spinner();
            state_changed = true;
        }

        if needs_render || state_changed {
            terminal.draw(|frame| app.render_frame(frame))?;
            needs_render = false;
        }

        if !event::poll(std::time::Duration::from_millis(50))? {
            continue;
        }

        needs_render = true;

        let (width, height) = crossterm::terminal::size()?;
        let event = event::read()?;
        match event {
            Event::Key(key_event) => {
                let key = convert_key(key_event);
                let action = app.handle_key(&key);

                if app.should_quit() {
                    break;
                }

                if let Some(action) = action {
                    dispatch_action(app, state, &mut active_turn, action, width, height)?;
                }
            }
            Event::Mouse(mouse_event) => {
                if let Some((col, row, action)) = convert_mouse(mouse_event) {
                    app.handle_mouse(col, row, action, width, height);
                }
            }
            Event::Resize(_, _) | Event::FocusGained | Event::FocusLost => {
                needs_render = true;
            }
            Event::Paste(text) => {
                if !app.has_blocking_modal() {
                    app.set_input_content(text);
                }
            }
        }
    }

    Ok(())
}

fn dispatch_action(
    app: &mut TuiApp,
    state: &mut TuiState,
    active_turn: &mut Option<ActiveTurn>,
    action: TuiAction,
    width: u16,
    height: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        TuiAction::PermissionDecision(allow) => {
            if let Some(turn) = active_turn.as_mut() {
                let _ = turn.permission_tx.send(allow);
            }
            app.set_pending_permission(None);
        }
        TuiAction::FlowResume { flow_id, run_id } => {
            handle_flow_resume(app, state, &flow_id, &run_id)?;
        }
        TuiAction::InterruptTurn => {
            if let Some(turn) = active_turn.take() {
                turn.request_cancel();
                turn.join();
            }
            app.end_turn_stream();
            app.set_thinking(false);
            app.active_tool = None;
            app.set_pending_permission(None);
            state.turn_started_at = None;
            open_human_guide(app);
            app.push_system_message(
                "\x1b[38;5;183m已请求中断当前轮次。\x1b[0m 请在底栏编辑引导内容后 Enter 发送。",
            );
        }
        TuiAction::SaveSession => {
            let _ = state.persist_session(app);
            app.push_system_message("\x1b[32m会话已保存。\x1b[0m");
            refresh_status(app, state);
        }
        TuiAction::OpenSessionPicker => match list_managed_sessions() {
            Ok(sessions) => {
                app.open_session_picker(sessions, state.session_handle.id.clone());
            }
            Err(e) => {
                app.push_system_message(&format!("\x1b[31m无法加载会话列表:\x1b[0m {e}"));
            }
        },
        TuiAction::RefreshStatus => {
            refresh_status(app, state);
            app.push_system_message("\x1b[2m状态已刷新。\x1b[0m");
        }
        TuiAction::NewSession => {
            app.clear_chat();
            app.tools = crate::tui::components::tool_panel::ToolPanel::new();
            app.push_system_message(
                "\x1b[2m对话区已清空。如需全新 runtime 会话请使用 /new。\x1b[0m",
            );
        }
        TuiAction::Submit {
            text,
            as_intervention,
            mut interrupted_turn,
        } => {
            if active_turn.is_some() {
                if let Some(turn) = active_turn.take() {
                    turn.request_cancel();
                    turn.join();
                }
                interrupted_turn = true;
                app.end_turn_stream();
                app.set_thinking(false);
                app.active_tool = None;
                app.set_pending_permission(None);
                state.turn_started_at = None;
            }

            let prompt = if as_intervention || interrupted_turn {
                wrap_human_intervention(&text, interrupted_turn)
            } else {
                text.clone()
            };

            if text.trim().starts_with('/') {
                match handle_slash_command(app, state, &text, width, height)? {
                    Some(SlashDispatch::Handled) => {}
                    Some(SlashDispatch::AgentTurn(agent_prompt)) => {
                        start_turn(app, state, active_turn, &agent_prompt);
                    }
                    None => {
                        app.push_system_message("未识别的斜杠命令。输入 /help 查看可用命令。");
                    }
                }
            } else {
                start_turn(app, state, active_turn, &prompt);
            }
        }
    }
    Ok(())
}

fn handle_flow_resume(
    app: &mut TuiApp,
    state: &mut TuiState,
    flow_id: &str,
    run_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match execute_flow_resume(flow_id, run_id, true) {
        Ok(output) => {
            state
                .suspended_flows
                .retain(|f| !(f.flow_id == flow_id && f.run_id == run_id));
            app.set_pending_flow_hitl(None);
            sync_pending_flow_overlay(app, &mut *state);
            app.push_assistant_message(&format!(
                "\x1b[32m工作流已恢复\x1b[0m ({flow_id} / {run_id})\n\n{output}"
            ));
            let _ = state.persist_session(app);
            refresh_status(app, state);
        }
        Err(e) => {
            app.push_system_message(&format!("\x1b[31m恢复失败:\x1b[0m {e}"));
        }
    }
    Ok(())
}

fn start_turn(
    app: &mut TuiApp,
    state: &mut TuiState,
    active_turn: &mut Option<ActiveTurn>,
    text: &str,
) {
    let mut effective_model = app.model().to_string();

    if app.model() == "auto" {
        let history_rounds = state
            .runtime
            .lock()
            .map_or(0, |r| r.session().messages.len());
        let router_config = load_router_config();
        let resolved = select_model_for_request("auto", text, history_rounds, 0, &router_config);

        let need_rebuild = match &state.active_model {
            None => true,
            Some(current) => current != &resolved,
        };

        if need_rebuild {
            match state.runtime.lock().map(|r| r.session().clone()) {
                Ok(session) => match build_runtime(
                    session,
                    resolved.clone(),
                    state.system_prompt.clone(),
                    true,
                    false,
                    state.allowed_tools.clone(),
                    state.permission_mode,
                ) {
                    Ok(new_runtime) => {
                        if let Ok(mut rt) = state.runtime.lock() {
                            *rt = new_runtime;
                        }
                    }
                    Err(e) => {
                        app.push_system_message(&format!(
                            "\x1b[33mauto 路由切换失败，保持当前模型: {e}\x1b[0m"
                        ));
                    }
                },
                Err(e) => {
                    app.push_system_message(&format!("\x1b[33mauto 路由读取会话失败: {e}\x1b[0m"));
                }
            }
            state.last_auto_decision = Some(format!("auto→{}", resolved));
        }
        state.active_model = Some(resolved);
        effective_model = state.active_model.clone().unwrap_or(effective_model);
    } else if state.active_model.is_none() {
        state.active_model = Some(app.model().to_string());
        effective_model = state.active_model.clone().unwrap_or(effective_model);
    } else {
        effective_model = state.active_model.clone().unwrap_or(effective_model);
    }

    if let Some(msg) = missing_api_key_message(&effective_model) {
        app.push_assistant_message(&msg);
        return;
    }

    app.set_thinking(true);
    app.begin_turn_stream();
    state.turn_started_at = Some(Instant::now());
    app.reset_turn_progress();
    close_human_guide(app);
    refresh_status(app, state);
    // Workflow routing: classify input, inject context prefix, merge suggested tools
    let decision = crate::routing::route(text);
    let snapshot = crate::routing::RoutingSnapshot::from_decision(&decision);
    let prefix = crate::routing::routing_user_prefix(&decision);
    crate::routing::merge_suggested_tools(&mut state.allowed_tools, &decision);
    state.last_routing = Some(snapshot);
    state.last_route_hint = Some(crate::routing::format_route_label(&decision));
    let routed_input = format!("{prefix}{text}");
    *active_turn = Some(spawn_turn(Arc::clone(&state.runtime), routed_input));
}

fn assistant_blocks_to_chat_text(blocks: &[runtime::ContentBlock]) -> String {
    use runtime::ContentBlock;

    let mut out = String::new();
    for block in blocks {
        match block {
            ContentBlock::Reasoning { text } | ContentBlock::Text { text } => {
                if !out.is_empty() {
                    out.push_str("\n\n");
                }
                out.push_str(text);
            }
            _ => {}
        }
    }
    out
}

fn ingest_turn_summary(
    app: &mut TuiApp,
    state: &mut TuiState,
    summary: &runtime::TurnSummary,
    streamed: bool,
) {
    use runtime::ContentBlock;

    for msg in &summary.assistant_messages {
        let text = assistant_blocks_to_chat_text(&msg.blocks);
        if !text.is_empty() {
            if streamed {
                if app.last_assistant_text_is_empty() {
                    app.set_last_assistant_text(text.clone());
                }
            } else {
                app.push_assistant_message(&text);
            }
        }
        for block in &msg.blocks {
            if let ContentBlock::ToolUse { name, .. } = block {
                app.push_tool_entry(crate::tui::components::tool_panel::ToolEntry {
                    name: name.clone(),
                    detail: String::new(),
                    is_error: false,
                    collapsed: false,
                });
            }
        }
    }

    for msg in &summary.tool_results {
        for block in &msg.blocks {
            if let ContentBlock::ToolResult {
                tool_name,
                output,
                is_error,
                ..
            } = block
            {
                ingest_flow_tool_result(app, state, tool_name, output, *is_error);

                app.push_tool_entry(crate::tui::components::tool_panel::ToolEntry {
                    name: tool_name.clone(),
                    detail: output.clone(),
                    is_error: *is_error,
                    collapsed: output.lines().count() > 10,
                });
            }
        }
    }
}

fn convert_mouse(event: MouseEvent) -> Option<(u16, u16, MouseAction)> {
    let action = match event.kind {
        MouseEventKind::ScrollUp => MouseAction::ScrollUp,
        MouseEventKind::ScrollDown => MouseAction::ScrollDown,
        MouseEventKind::Down(MouseButton::Left) => MouseAction::LeftClick,
        _ => return None,
    };
    Some((event.column, event.row, action))
}

fn convert_key(key: CrosstermKey) -> KeyEvent {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.modifiers.contains(KeyModifiers::SHIFT)
    {
        if let KeyCode::Char(c) = key.code {
            return KeyEvent::CtrlShift(c.to_ascii_lowercase());
        }
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            return KeyEvent::Ctrl(c.to_ascii_lowercase());
        }
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) && matches!(key.code, KeyCode::Enter) {
        return KeyEvent::ShiftEnter;
    }

    match key.code {
        KeyCode::Tab => KeyEvent::Tab,
        KeyCode::Enter => KeyEvent::Enter,
        KeyCode::Backspace => KeyEvent::Backspace,
        KeyCode::Delete => KeyEvent::Delete,
        KeyCode::Left => KeyEvent::Left,
        KeyCode::Right => KeyEvent::Right,
        KeyCode::Up => KeyEvent::Up,
        KeyCode::Down => KeyEvent::Down,
        KeyCode::Home => KeyEvent::Home,
        KeyCode::End => KeyEvent::End,
        KeyCode::Esc => KeyEvent::Esc,
        KeyCode::F(n) => KeyEvent::F(n),
        KeyCode::Char(c) => KeyEvent::Char(c),
        _ => KeyEvent::Char('\0'),
    }
}
