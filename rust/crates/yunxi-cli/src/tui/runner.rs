//! TUI 全屏 REPL（通用 + 专利专屏）。

use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossterm::cursor::Show;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent as CrosstermKey,
    KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use crate::cli_action::AllowedToolSet;
use crate::model_routing::{load_router_config, select_model_for_request};
use crate::runtime_bridge::{build_runtime, build_system_prompt};
use crate::session_meta::{execute_flow_resume, load_athena_meta, AthenaSessionMeta};
use crate::session_mgr::create_managed_session_handle;
use crate::tui::app::{KeyEvent, MouseAction, TuiAction, TuiApp};
use crate::tui::hitl::{
    close_human_guide, ingest_flow_tool_result, open_human_guide, sync_pending_flow_overlay,
    wrap_human_intervention,
};
use crate::tui::patent::default_tools::resolve_patent_allowed_tools;
use crate::tui::patent::session_store::hydrate_workspace;
use crate::tui::patent::yunxi_md::{is_patent_project_yunxi_md, load_patent_working_agreement};
use crate::tui::slash::{handle_slash_command, refresh_status, SlashDispatch};
use crate::tui::turn::{spawn_turn, ActiveTurn, TurnEvent};
use crate::tui::ui_mode::UiMode;
use crate::VERSION;

use crate::routing::RoutingSnapshot;
use runtime::{PermissionMode, Session};

/// 启动 TUI 全屏 REPL。
pub(crate) fn run_tui_repl(
    model: String,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    ui_mode: UiMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let allowed_tools = match ui_mode {
        UiMode::Patent => resolve_patent_allowed_tools(allowed_tools),
        UiMode::General => allowed_tools,
    };

    let system_prompt = build_system_prompt()?;
    let session_handle = create_managed_session_handle()?;
    let runtime = Arc::new(Mutex::new(build_runtime(
        Session::new(),
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

    let mut app = TuiApp::new(model, VERSION.to_string(), ui_mode);
    sync_pending_flow_overlay(&mut app, &mut state);

    if app.is_patent_mode() {
        let cwd = std::env::current_dir()?;
        hydrate_workspace(
            &mut app.patent,
            &state.session_handle.path,
            state
                .runtime
                .lock()
                .map_err(|_| "runtime lock poisoned")?
                .session(),
        );
        if is_patent_project_yunxi_md(&cwd) {
            if let Some(agreement) = load_patent_working_agreement(&cwd) {
                app.push_assistant_message(&format!(
                    "专利专屏已启动。工作约定已加载。\n建议先运行 /init 扫描案件材料。\n\n{agreement}"
                ));
            }
        } else {
            app.push_assistant_message(
                "专利专屏已启动。当前目录无案件 YUNXI.md，建议在案件文件夹内运行 /init。\n\n底栏 Enter 发送消息；按 6 或 /view chat 查看完整对话。",
            );
        }
        app.push_system_message(
            "\x1b[2m人机协作：Ctrl+G 打开引导 · Ctrl+I 中断并引导 · 工作流挂起时按 y/n\x1b[0m",
        );
    } else {
        let cwd = std::env::current_dir()
            .map_or_else(|_| "<unknown>".to_string(), |p| p.display().to_string());
        let banner = crate::tui::banner::render_banner(
            &app.model,
            state.permission_mode.as_str(),
            &cwd,
            &state.session_handle.id,
        );
        app.push_assistant_message(&banner);
        app.push_system_message(
            "\x1b[2m人机协作：Ctrl+G 引导 · Ctrl+I 中断轮次 · Flow 挂起时 y 继续 / n 稍后\x1b[0m",
        );
    }

    refresh_status(&mut app, &state);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let result = run_event_loop(&mut app, &mut state, &mut stdout);

    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show)?;
    disable_raw_mode()?;

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
        if app.is_patent_mode() {
            crate::tui::patent::session_store::merge_save_session(
                &self.session_handle.path,
                &session,
                Some(&app.patent.case),
                &self.athena_meta(),
            )?;
        } else {
            crate::session_meta::merge_save_session(
                &self.session_handle.path,
                &session,
                &self.athena_meta(),
                None,
            )?;
        }
        Ok(())
    }
}

fn run_event_loop(
    app: &mut TuiApp,
    state: &mut TuiState,
    stdout: &mut io::Stdout,
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
                                    let msg = error.to_string();
                                    app.push_assistant_message(&format!(
                                        "\x1b[31m请求失败:\x1b[0m\n{msg}\n\n\
                                         \x1b[2m提示: 默认模型 deepseek-v4-pro 会映射为 deepseek-chat；\
                                         请确认 DEEPSEEK_API_KEY 有效，或用 /model 切换模型。\x1b[0m"
                                    ));
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
            let (width, height) = crossterm::terminal::size()?;
            let rendered = app.render_with_cursor(width, height);
            execute!(stdout, Print(&rendered))?;
            stdout.flush()?;
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
                        app.push_system_message(
                            "未识别的斜杠命令。输入 /help 查看可用命令；专利材料扫描请用 /init。",
                        );
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
    } else if state.active_model.is_none() {
        state.active_model = Some(app.model().to_string());
    }

    app.set_thinking(true);
    app.begin_turn_stream();
    state.turn_started_at = Some(Instant::now());
    app.reset_turn_progress();
    close_human_guide(app);
    refresh_status(app, state);
    *active_turn = Some(spawn_turn(Arc::clone(&state.runtime), text.to_string()));
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
        if app.is_patent_mode() && !text.is_empty() {
            crate::tui::patent::ingest::ingest_assistant_text(&mut app.patent, &text);
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

                if app.is_patent_mode() {
                    crate::tui::patent::ingest::ingest_tool_result(
                        &mut app.patent,
                        tool_name,
                        output,
                        output,
                        *is_error,
                    );
                }
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
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            return KeyEvent::Ctrl(c);
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
        KeyCode::Esc => KeyEvent::Esc,
        KeyCode::F(n) => KeyEvent::F(n),
        KeyCode::Char(c) => KeyEvent::Char(c),
        _ => KeyEvent::Char('\0'),
    }
}

/// ratatui 版本的 TUI 运行函数（通过 YUNXI_RATATUI=1 触发）。
pub(crate) fn run_tui_ratatui(
    model: String,
    _allowed_tools: Option<AllowedToolSet>,
    _permission_mode: PermissionMode,
    ui_mode: UiMode,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::tui::app::TuiApp;
    use crate::tui::terminal::{restore_terminal, setup_terminal};
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::crossterm;

    let mut app = TuiApp::new(model, crate::VERSION.to_string(), ui_mode);

    let mut terminal = setup_terminal()?;

    app.render_ratatui(&mut terminal);

    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
        app.spinner_frame = app.spinner_frame.wrapping_add(1);
        app.render_ratatui(&mut terminal);
    }

    restore_terminal(terminal)?;
    Ok(())
}
