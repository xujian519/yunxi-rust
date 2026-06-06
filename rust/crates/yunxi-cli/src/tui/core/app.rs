//! New-architecture TUI application: combines event loop, state, rendering, and LLM integration.
//! Replaces the old TuiApp + runner.rs + app_ratatui.rs architecture.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crossterm::event::Event as CrosstermEvent;

use runtime::PermissionMode;

use crate::cli_action::AllowedToolSet;
use crate::model_routing::{load_router_config, select_model_for_request};
use crate::runtime_bridge::{build_runtime, build_system_prompt};
use crate::session_meta::{execute_flow_resume, AthenaSessionMeta};
use crate::session_mgr::{create_managed_session_handle, list_managed_sessions, SessionHandle};
use crate::tui::components::chat_view::{ChatEntry, ChatRole, ChatView};
use crate::tui::components::command_palette::CommandPalette;
use crate::tui::components::input_bar::InputBar;
use crate::tui::components::session_picker::SessionPicker;
use crate::tui::components::tool_panel::{ToolEntry, ToolPanel};
use crate::tui::core::action::{Action, ActionResult};
use crate::tui::core::event::{Event, EventDispatcher, InputEvent};
use crate::tui::keymap::KeyMap;
use crate::tui::state::global::GlobalState;
use crate::tui::theme::ThemeManager;
use crate::tui::turn::{spawn_turn, ActiveTurn, SharedRuntime, TurnEvent};
use crate::VERSION;

/// TUI application state.
pub(crate) struct App {
    // ── UI state ──
    pub chat: ChatView,
    pub tools: ToolPanel,
    pub input: InputBar,
    pub show_help: bool,
    pub show_tool_panel: bool,
    pub show_sidebar: bool,
    pub show_guide: bool,
    pub thinking: bool,
    pub model: String,
    pub version: String,
    pub session_picker: Option<SessionPicker>,
    pub command_palette: CommandPalette,
    pub slash_completion: Option<crate::tui::slash_complete::SlashCompletion>,
    pub active_tool: Option<String>,
    pub keymap: KeyMap,

    // ── Runtime state ──
    pub runtime: SharedRuntime,
    pub session_handle: SessionHandle,
    pub system_prompt: Vec<String>,
    pub allowed_tools: Option<AllowedToolSet>,
    pub permission_mode: PermissionMode,
    pub active_model: Option<String>,
    pub last_auto_decision: Option<String>,
    pub turn_started_at: Option<Instant>,

    // ── Architecture 2 state ──
    pub global_state: GlobalState,
    pub theme_manager: ThemeManager,
    pub event_dispatcher: EventDispatcher,

    // ── Stream state ──
    pub(crate) stream_state_active: bool,
    pub(crate) stream_state_saw_text: bool,
    pub(crate) spinner_frame: usize,

    // ── Internal flags ──
    pub(crate) should_quit: bool,
    pub(crate) active_turn: Option<ActiveTurn>,
    pub(crate) needs_render: bool,
    clipboard: Option<String>,
}

impl App {
    pub fn new(model: String) -> Result<Self, Box<dyn std::error::Error>> {
        let system_prompt = build_system_prompt()?;
        let session_handle = create_managed_session_handle()?;
        let session = runtime::Session::new();
        let runtime = Arc::new(Mutex::new(build_runtime(
            session,
            model.clone(),
            system_prompt.clone(),
            true,
            false,
            None,
            PermissionMode::Allow,
        )?));

        let mut theme_manager = ThemeManager::default();
        theme_manager.set_auto_theme();

        let mut command_palette = CommandPalette::new();
        App::register_commands(&mut command_palette);

        let keymap = KeyMap::new();

        let mut global_state = GlobalState::new();
        global_state.theme.current_theme = theme_manager.current_name().to_string();
        global_state.theme.is_dark = theme_manager.get_theme().is_dark;
        crate::tui::ui_palette::active::apply(theme_manager.get_theme().clone());

        Ok(Self {
            chat: ChatView::new(),
            tools: ToolPanel::new(),
            input: InputBar::new(),
            show_help: false,
            show_tool_panel: true,
            show_sidebar: false,
            show_guide: false,
            thinking: false,
            model: model.clone(),
            version: VERSION.to_string(),
            session_picker: None,
            command_palette,
            slash_completion: None,
            active_tool: None,
            keymap,

            runtime,
            session_handle,
            system_prompt,
            allowed_tools: None,
            permission_mode: PermissionMode::Allow,
            active_model: Some(model),
            last_auto_decision: None,
            turn_started_at: None,

            global_state,
            theme_manager,
            event_dispatcher: EventDispatcher::new(),

            stream_state_active: false,
            stream_state_saw_text: false,
            spinner_frame: 0,

            should_quit: false,
            active_turn: None,
            needs_render: true,
            clipboard: None,
        })
    }

    fn register_commands(palette: &mut CommandPalette) {
        use crate::tui::keymap::Command;
        let commands = [
            Command::new("HumanGuide", "打开人机引导面板", Action::ShowGuide),
            Command::new("InterruptTurn", "中断当前轮次", Action::InterruptTurn),
            Command::new("SessionList", "打开会话列表", Action::ShowSessionPicker),
            Command::new("CopyChat", "复制对话到剪贴板", Action::CopySelection),
            Command::new("ToggleToolPanel", "切换工具面板", Action::ToggleSidebar),
            Command::new("CycleTheme", "切换主题", Action::ToggleDarkMode),
        ];
        for cmd in commands {
            palette.register_command(cmd);
        }
    }

    // ── Public interface ──

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn needs_render(&self) -> bool {
        self.needs_render
    }

    pub fn clear_render_flag(&mut self) {
        self.needs_render = false;
    }

    pub fn has_blocking_modal(&self) -> bool {
        self.pending_permission().is_some() || self.pending_flow_hitl().is_some()
    }

    // ── Event loop ──

    pub fn run_event_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Poll active turn events
            // Collect turn events first, then process them to avoid borrow conflicts
            let turn_events: Vec<TurnEvent> = self
                .active_turn
                .as_mut()
                .map(|t| t.poll())
                .unwrap_or_default();
            let turn_finished = self
                .active_turn
                .as_ref()
                .map(|t| t.is_finished())
                .unwrap_or(false);

            for event in turn_events {
                self.needs_render = true;
                self.handle_turn_event(event);
            }
            if turn_finished {
                let finished = self.active_turn.take().expect("turn");
                finished.join();
            }

            // Tick spinner when thinking
            if self.thinking {
                self.tick_spinner();
                self.needs_render = true;
            }

            // Wait for input (non-blocking poll)
            if !crossterm::event::poll(std::time::Duration::from_millis(50))? {
                continue;
            }

            self.needs_render = true;

            match crossterm::event::read()? {
                CrosstermEvent::Key(key_event) => {
                    let key = convert_key(&key_event);
                    let action = self.handle_key(&key);

                    // Dispatch to event dispatcher for component-based handling
                    let event = Event::Input(InputEvent::Key(key_event));
                    let _results = self.event_dispatcher.dispatch(&event);

                    if let Some(action) = action {
                        self.dispatch_action(action)?;
                    }
                }
                CrosstermEvent::Mouse(mouse_event) => {
                    self.handle_mouse(mouse_event);
                }
                CrosstermEvent::Resize(_, _) => {}
                CrosstermEvent::Paste(text) => {
                    if !self.has_blocking_modal() {
                        self.input.set_content(text);
                    }
                }
                _ => {}
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// Public wrapper for runner
    pub(crate) fn handle_turn_event_wrapped(&mut self, event: TurnEvent) {
        self.handle_turn_event(event);
    }

    fn handle_turn_event(&mut self, event: TurnEvent) {
        match event {
            TurnEvent::Stream(stream_event) => {
                self.append_stream_event(&stream_event);
            }
            TurnEvent::ToolUse { name } => {
                self.active_tool = Some(name);
            }
            TurnEvent::Permission(req) => {
                self.set_pending_permission(Some(req));
            }
            TurnEvent::Done(result) => {
                let is_streamed = self.turn_was_streamed();
                self.end_turn_stream();
                if !self.active_turn.as_ref().is_some_and(|t| t.is_cancelled()) {
                    match result {
                        Ok(summary) => self.ingest_turn_summary(&summary, is_streamed),
                        Err(error) => {
                            let msg = crate::llm_auth::format_llm_error(self.model(), &error);
                            if self.current_assistant_empty() {
                                self.set_last_assistant_text(msg);
                            } else {
                                self.push_assistant_message(&msg);
                            }
                        }
                    }
                } else {
                    self.push_system_message("已中断上一轮；新指示已发送。");
                }
                self.set_thinking(false);
                self.active_tool = None;
                self.set_pending_permission(None);
                self.turn_started_at = None;
                self.persist_session();
            }
        }
    }

    /// Public wrapper for runner
    pub(crate) fn dispatch_action_wrapped(
        &mut self,
        action: Action,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.dispatch_action(action)
    }

    fn dispatch_action(&mut self, action: Action) -> Result<(), Box<dyn std::error::Error>> {
        match action {
            Action::Submit(text, as_intervention) => {
                self.handle_submit(text, as_intervention)?;
            }
            Action::InterruptTurn => {
                self.interrupt_turn();
            }
            Action::ShowHelp => {
                self.show_help = true;
            }
            Action::ShowGuide => {
                self.open_human_guide();
            }
            Action::ShowSessionPicker => {
                if let Ok(sessions) = list_managed_sessions() {
                    self.session_picker =
                        Some(SessionPicker::new(sessions, self.session_handle.id.clone()));
                }
            }
            Action::OpenSessionPicker => {
                if let Ok(sessions) = list_managed_sessions() {
                    self.session_picker =
                        Some(SessionPicker::new(sessions, self.session_handle.id.clone()));
                }
            }
            Action::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
            }
            Action::ToggleDarkMode => {
                self.cycle_theme();
            }
            Action::SwitchTheme(name) => {
                self.set_theme_by_name(&name);
            }
            Action::CopySelection => {
                self.copy_visible_text_to_clipboard();
            }
            Action::SaveSession => {
                self.persist_session();
            }
            Action::PermissionDecision(allow) => {
                if let Some(turn) = self.active_turn.as_mut() {
                    let _ = turn.permission_tx.send(allow);
                }
                self.set_pending_permission(None);
            }
            Action::FlowResume(flow_id, run_id) => {
                self.handle_flow_resume(&flow_id, &run_id)?;
            }
            Action::Quit => {
                self.should_quit = true;
            }
            Action::ExecuteCommand(cmd) => match cmd.as_str() {
                "sessions" => {
                    if let Ok(sessions) = list_managed_sessions() {
                        self.session_picker =
                            Some(SessionPicker::new(sessions, self.session_handle.id.clone()));
                    }
                }
                "interrupt" => self.interrupt_turn(),
                "edit_keymap" => {
                    let bindings: Vec<String> = self
                        .keymap
                        .list_all_bindings()
                        .iter()
                        .map(|b| format!("{}: {}", b.sequence, b.command))
                        .collect();
                    let msg = if bindings.is_empty() {
                        "暂无快捷键绑定".to_string()
                    } else {
                        format!(
                            "当前快捷键绑定 (共 {} 条):\n{}",
                            bindings.len(),
                            bindings.join("\n")
                        )
                    };
                    self.push_system_message(&msg);
                }
                _ => {
                    self.push_system_message(&format!("未接入的命令 '{}'", cmd));
                }
            },
            Action::ShowDialog(name) => match name.as_str() {
                "go_to_top" => self.chat.scroll_to_top(),
                "go_to_bottom" => self.chat.scroll_to_bottom(10),
                "navigate_down" => self.chat.scroll_down(10),
                "navigate_up" => self.chat.scroll_up(),
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    // ── Turn management ──

    fn handle_submit(
        &mut self,
        text: String,
        as_intervention: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut interrupted = false;
        if self.active_turn.is_some() {
            if let Some(turn) = self.active_turn.take() {
                turn.request_cancel();
                turn.join();
            }
            interrupted = true;
            self.end_turn_stream();
            self.set_thinking(false);
            self.active_tool = None;
            self.set_pending_permission(None);
            self.turn_started_at = None;
        }

        // Handle slash commands
        if text.trim().starts_with('/') {
            let result = self.handle_slash_command(&text)?;
            if !result {
                self.push_system_message("未识别的斜杠命令。输入 /help 查看可用命令。");
            }
            return Ok(());
        }

        let prompt = if as_intervention || interrupted {
            self.wrap_human_intervention(&text, interrupted)
        } else {
            text.clone()
        };

        self.start_llm_turn(&prompt);
        Ok(())
    }

    fn start_llm_turn(&mut self, text: &str) {
        let effective_model = self.resolve_model(text);

        if let Some(msg) = crate::llm_auth::missing_api_key_message(&effective_model) {
            self.push_assistant_message(&msg);
            return;
        }

        self.set_thinking(true);
        self.begin_turn_stream();
        self.turn_started_at = Some(Instant::now());
        self.active_tool = None;
        self.close_human_guide();

        // Routing
        let decision = crate::routing::route(text);
        let prefix = crate::routing::routing_user_prefix(&decision);
        let _snapshot = crate::routing::RoutingSnapshot::from_decision(&decision);
        crate::routing::merge_suggested_tools(&mut self.allowed_tools, &decision);

        let routed_input = format!("{prefix}{text}");
        self.active_turn = Some(spawn_turn(Arc::clone(&self.runtime), routed_input));
    }

    fn resolve_model(&mut self, text: &str) -> String {
        if self.model != "auto" {
            return self.model.clone();
        }

        let history_rounds = self
            .runtime
            .lock()
            .map_or(0, |r| r.session().messages.len());
        let router_config = load_router_config();
        let resolved = select_model_for_request("auto", text, history_rounds, 0, &router_config);

        let need_rebuild = match &self.active_model {
            None => true,
            Some(current) => current != &resolved,
        };

        if need_rebuild {
            let session = self.runtime.lock().ok().map(|r| r.session().clone());
            if let Some(session) = session {
                match build_runtime(
                    session,
                    resolved.clone(),
                    self.system_prompt.clone(),
                    true,
                    false,
                    self.allowed_tools.clone(),
                    self.permission_mode,
                ) {
                    Ok(new_runtime) => {
                        if let Ok(mut rt) = self.runtime.lock() {
                            *rt = new_runtime;
                        }
                    }
                    Err(e) => {
                        self.push_system_message(&format!("auto 路由切换失败: {e}"));
                    }
                }
            } else {
                self.push_system_message("auto 路由读取会话失败: runtime lock poisoned");
            }
            self.last_auto_decision = Some(format!("auto→{}", resolved));
        }
        self.active_model = Some(resolved.clone());
        resolved
    }

    fn interrupt_turn(&mut self) {
        if let Some(turn) = self.active_turn.take() {
            turn.request_cancel();
            turn.join();
        }
        self.end_turn_stream();
        self.set_thinking(false);
        self.active_tool = None;
        self.set_pending_permission(None);
        self.turn_started_at = None;
        self.open_human_guide();
        self.push_system_message("已请求中断当前轮次。请在底栏编辑引导内容后 Enter 发送。");
    }

    // ── Key handling ──

    pub fn handle_key(&mut self, key: &KeyEvent) -> Option<Action> {
        // Modal overlays take priority
        if let Some(action) = self.handle_modal_keys(key) {
            return Some(action);
        }

        // Command palette
        if self.command_palette.is_visible() {
            return self.handle_command_palette_key(key);
        }

        if self.show_guide && matches!(key, KeyEvent::Esc) {
            self.show_guide = false;
            return None;
        }

        if matches!(key, KeyEvent::CtrlShift('c')) {
            self.copy_visible_text_to_clipboard();
            return None;
        }

        match key {
            KeyEvent::Char('q') if self.input.is_empty() => {
                self.should_quit = true;
                None
            }
            KeyEvent::Ctrl('c') | KeyEvent::Esc => {
                if self.input.is_empty() {
                    self.should_quit = true;
                } else {
                    self.input = InputBar::new();
                }
                None
            }
            KeyEvent::Enter if !self.input.is_empty() && !self.has_blocking_modal() => {
                let text = self.input.take();
                let as_interv = self.show_guide;
                self.show_guide = false;
                self.chat.push(ChatEntry {
                    role: ChatRole::User,
                    text: text.clone(),
                    reasoning: None,
                });
                self.chat.scroll_to_bottom(10);
                Some(Action::Submit(text, as_interv))
            }
            KeyEvent::Ctrl('h') | KeyEvent::F(1) => Some(Action::ShowHelp),
            KeyEvent::Ctrl('p') | KeyEvent::F(3) => {
                self.command_palette.show();
                None
            }
            KeyEvent::Ctrl('b') => {
                self.show_sidebar = !self.show_sidebar;
                None
            }
            KeyEvent::Ctrl('e') => {
                self.tools.toggle_all();
                None
            }
            KeyEvent::Ctrl('d') => {
                let name = self.cycle_theme();
                self.push_system_message(&format!("主题: {name}"));
                None
            }
            KeyEvent::Ctrl('g') => {
                self.open_human_guide();
                None
            }
            KeyEvent::Ctrl('i') => Some(Action::InterruptTurn),
            KeyEvent::Ctrl('u') => {
                self.input.set_content("/import ".to_string());
                self.refresh_slash_completion();
                None
            }
            KeyEvent::Ctrl('f') => {
                self.input.set_content("/search ".to_string());
                self.refresh_slash_completion();
                None
            }
            KeyEvent::Tab => {
                let _ = self.apply_tab_completion();
                None
            }
            KeyEvent::ShiftEnter => {
                self.input.insert('\n');
                self.refresh_slash_completion();
                None
            }
            KeyEvent::F(2) => {
                self.show_tool_panel = !self.show_tool_panel;
                None
            }
            KeyEvent::Char('j') | KeyEvent::Down if self.input.is_empty() => {
                if self.show_tool_panel {
                    self.tools.scroll_down(1, 80);
                } else {
                    self.chat.scroll_down(10);
                }
                None
            }
            KeyEvent::Char('k') | KeyEvent::Up if self.input.is_empty() => {
                if self.show_tool_panel {
                    self.tools.scroll_up();
                } else {
                    self.chat.scroll_up();
                }
                None
            }
            KeyEvent::Char(c) => {
                self.input.insert(*c);
                self.refresh_slash_completion();
                None
            }
            KeyEvent::Backspace => {
                self.input.backspace();
                self.refresh_slash_completion();
                None
            }
            KeyEvent::Delete => {
                self.input.delete();
                self.refresh_slash_completion();
                None
            }
            KeyEvent::Left => {
                self.input.move_left();
                None
            }
            KeyEvent::Right => {
                self.input.move_right();
                None
            }
            KeyEvent::Home => {
                self.input.move_home();
                None
            }
            KeyEvent::End => {
                self.input.move_end();
                None
            }
            _ => None,
        }
    }

    fn handle_modal_keys(&self, key: &KeyEvent) -> Option<Action> {
        if self.pending_flow_hitl().is_some() {
            return match key {
                KeyEvent::Char('y') | KeyEvent::Char('Y') => {
                    let record = self.pending_flow_hitl().unwrap();
                    Some(Action::FlowResume(
                        record.flow_id.clone(),
                        record.run_id.clone(),
                    ))
                }
                KeyEvent::Char('n') | KeyEvent::Char('N') | KeyEvent::Esc => {
                    None // handled externally
                }
                _ => None,
            };
        }
        if self.pending_permission().is_some() {
            return match key {
                KeyEvent::Char('y') | KeyEvent::Char('Y') => Some(Action::PermissionDecision(true)),
                KeyEvent::Char('n') | KeyEvent::Char('N') | KeyEvent::Esc | KeyEvent::Ctrl('c') => {
                    Some(Action::PermissionDecision(false))
                }
                _ => None,
            };
        }
        None
    }

    fn handle_command_palette_key(&mut self, key: &KeyEvent) -> Option<Action> {
        use crate::tui::components::base::Component;
        let event = app_key_to_event(key);
        let _ = &event; // suppress unused warning during transition
        match self.command_palette.handle_event(&event) {
            ActionResult::Action(action) => self.map_core_action(action),
            ActionResult::Handled | ActionResult::Ignored => None,
            ActionResult::Actions(actions) => {
                for a in actions {
                    if let Some(mapped) = self.map_core_action(a) {
                        return Some(mapped);
                    }
                }
                None
            }
        }
    }

    fn map_core_action(&self, action: Action) -> Option<Action> {
        Some(action)
    }

    /// Public wrapper for runner
    pub(crate) fn handle_mouse_wrapped(&mut self, event: crossterm::event::MouseEvent) {
        self.handle_mouse(event);
    }

    fn handle_mouse(&mut self, event: crossterm::event::MouseEvent) {
        match event.kind {
            crossterm::event::MouseEventKind::ScrollUp => {
                if !self.input.is_empty() {
                    return;
                }
                self.chat.scroll_up_by(3);
            }
            crossterm::event::MouseEventKind::ScrollDown => {
                if !self.input.is_empty() {
                    return;
                }
                self.chat.scroll_down_by(3, 20);
            }
            _ => {}
        }
    }

    // ── Flow HITL ──

    pub fn pending_flow_hitl(&self) -> Option<&crate::session_meta::SuspendedFlowRecord> {
        None // simplified; not persisted as state in this version
    }

    pub fn set_pending_flow_hitl(
        &mut self,
        _record: Option<crate::session_meta::SuspendedFlowRecord>,
    ) {
    }

    fn handle_flow_resume(
        &mut self,
        flow_id: &str,
        run_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match execute_flow_resume(flow_id, run_id, true) {
            Ok(output) => {
                self.push_assistant_message(&format!(
                    "工作流已恢复 ({flow_id} / {run_id})\n\n{output}"
                ));
                self.persist_session();
            }
            Err(e) => {
                self.push_system_message(&format!("恢复失败: {e}"));
            }
        }
        Ok(())
    }

    // ── Permission ──

    pub fn pending_permission(&self) -> Option<&runtime::PermissionRequest> {
        None // simplified
    }

    pub fn set_pending_permission(&mut self, _request: Option<runtime::PermissionRequest>) {}

    // ── Stream handling ──

    fn begin_turn_stream(&mut self) {
        self.stream_state_active = true;
        self.stream_state_saw_text = false;
        self.push_assistant_message("");
    }

    fn end_turn_stream(&mut self) {
        self.stream_state_active = false;
        self.stream_state_saw_text = false;
    }

    fn turn_was_streamed(&self) -> bool {
        self.stream_state_active
    }

    fn append_stream_event(&mut self, event: &runtime::AssistantEvent) {
        use runtime::AssistantEvent;
        match event {
            AssistantEvent::ReasoningDelta(delta) => {
                self.chat.append_reasoning_to_last(delta);
            }
            AssistantEvent::TextDelta(delta) => {
                if !self.stream_state_saw_text && self.chat.last_assistant_has_content() {
                    self.append_assistant_text("\n\n");
                }
                self.stream_state_saw_text = true;
                self.append_assistant_text(delta);
            }
            AssistantEvent::ToolUse { name, .. } => {
                self.append_assistant_text(&format!("\n\n▸ 调用 {name}…"));
                self.active_tool = Some(name.clone());
            }
            AssistantEvent::Usage(_) | AssistantEvent::MessageStop => {}
        }
    }

    fn ingest_turn_summary(&mut self, summary: &runtime::TurnSummary, streamed: bool) {
        use runtime::ContentBlock;
        for msg in &summary.assistant_messages {
            let text = Self::blocks_to_text(&msg.blocks);
            if !text.is_empty() {
                if streamed && self.current_assistant_empty() {
                    self.set_last_assistant_text(text);
                } else if !streamed {
                    self.push_assistant_message(&text);
                }
            }
            for block in &msg.blocks {
                if let ContentBlock::ToolUse { name, .. } = block {
                    self.tools.push(ToolEntry::new(name.clone(), ""));
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
                    let mut entry = ToolEntry::new(tool_name.clone(), output.clone());
                    if *is_error {
                        entry = entry.with_error();
                    }
                    if output.lines().count() <= 10 {
                        entry = entry.with_collapsed(false);
                    }
                    self.tools.push(entry);
                }
            }
        }
    }

    fn blocks_to_text(blocks: &[runtime::ContentBlock]) -> String {
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

    fn current_assistant_empty(&self) -> bool {
        !self.chat.last_assistant_has_content()
    }

    // ── Slash commands ──

    fn handle_slash_command(&mut self, input: &str) -> Result<bool, Box<dyn std::error::Error>> {
        use commands::SlashCommand;
        match input.trim() {
            "/exit" | "/quit" => {
                self.should_quit = true;
                return Ok(true);
            }
            _ => {}
        }

        let command = SlashCommand::parse(input);
        let Some(cmd) = command else {
            return Ok(false);
        };

        match cmd {
            SlashCommand::Help => {
                self.push_output("帮助", &commands::render_full_slash_help(), 80, 24);
            }
            SlashCommand::Status => {
                let report = self.build_status_report();
                self.push_output("Status", &report, 80, 24);
            }
            SlashCommand::Version => {
                self.push_system_message(&format!("云熙智能体 v{VERSION}"));
            }
            SlashCommand::Model { model } => {
                if let Some(m) = model {
                    self.switch_model(&m)?;
                }
            }
            SlashCommand::Permissions { mode } => {
                if let Some(m) = mode {
                    self.switch_permissions(&m)?;
                }
            }
            SlashCommand::Cost => {
                let report = self.build_cost_report();
                self.push_output("Cost", &report, 80, 24);
            }
            SlashCommand::Clear { confirm } => {
                if confirm {
                    self.runtime
                        .lock()
                        .map_err(|_| "lock")?
                        .session_mut()
                        .messages
                        .clear();
                    self.chat.clear();
                    self.persist_session();
                    self.push_system_message("已清空会话。");
                } else {
                    self.push_system_message("使用 /new 或 /clear --confirm");
                }
            }
            SlashCommand::Compact => {
                let result = self
                    .runtime
                    .lock()
                    .map_err(|_| "lock")?
                    .compact(runtime::CompactionConfig::default());
                let removed = result.removed_message_count;
                let kept = result.compacted_session.messages.len();
                let session = result.compacted_session;
                let new_runtime = build_runtime(
                    session,
                    self.model.clone(),
                    self.system_prompt.clone(),
                    true,
                    false,
                    self.allowed_tools.clone(),
                    self.permission_mode,
                )?;
                *self.runtime.lock().map_err(|_| "lock")? = new_runtime;
                self.persist_session();
                self.push_system_message(&format!("压缩完成: 移除 {removed} 条, 保留 {kept} 条"));
            }
            SlashCommand::Session { action, target } => match action.as_deref() {
                None | Some("list") => {
                    if let Ok(sessions) = list_managed_sessions() {
                        self.session_picker =
                            Some(SessionPicker::new(sessions, self.session_handle.id.clone()));
                    }
                }
                Some("switch") => {
                    if let Some(t) = target {
                        self.switch_session(&t)?;
                    }
                }
                _ => {}
            },
            SlashCommand::Undo => {
                let report = self.runtime.lock().ok().and_then(|mut rt| {
                    crate::format_report::undo_last_interaction(rt.session_mut()).ok()
                });
                if let Some(report) = report {
                    self.persist_session();
                    self.push_system_message(&report);
                }
            }
            SlashCommand::ThinkingToggle => {
                // Toggle not wired in new arch yet
                self.push_system_message("推理过程显示: 暂未支持");
            }
            _ => {
                self.push_system_message("该命令尚未接入新架构，请使用 /help 查看可用命令。");
            }
        }
        Ok(true)
    }

    fn build_status_report(&self) -> String {
        let Ok(runtime) = self.runtime.lock() else {
            return "error".to_string();
        };
        let cumulative = runtime.usage().cumulative_usage();
        let latest = runtime.usage().current_turn_usage();
        let context = crate::format_report::status_context(Some(&self.session_handle.path))
            .unwrap_or_else(|_| crate::format_report::StatusContext {
                cwd: std::path::PathBuf::from("."),
                session_path: None,
                loaded_config_files: 0,
                discovered_config_files: 0,
                memory_file_count: 0,
                project_root: None,
                git_branch: None,
            });
        crate::format_report::format_status_report(
            &self.model,
            crate::format_report::StatusUsage {
                message_count: runtime.session().messages.len(),
                turns: runtime.usage().turns(),
                latest,
                cumulative,
                estimated_tokens: runtime.estimated_tokens(),
            },
            self.permission_mode.as_str(),
            &context,
        )
    }

    fn build_cost_report(&self) -> String {
        let Ok(runtime) = self.runtime.lock() else {
            return "error".to_string();
        };
        let cumulative = runtime.usage().cumulative_usage();
        crate::format_report::format_cost_report(cumulative)
    }

    fn switch_model(&mut self, model: &str) -> Result<(), Box<dyn std::error::Error>> {
        let model = crate::cli_action::resolve_model_alias(model).to_string();
        if model == self.model {
            return Ok(());
        }
        let session = self.runtime.lock().map_err(|_| "lock")?.session().clone();
        let new_runtime = build_runtime(
            session,
            model.clone(),
            self.system_prompt.clone(),
            true,
            false,
            self.allowed_tools.clone(),
            self.permission_mode,
        )?;
        *self.runtime.lock().map_err(|_| "lock")? = new_runtime;
        self.model = model;
        Ok(())
    }

    fn switch_permissions(&mut self, mode: &str) -> Result<(), Box<dyn std::error::Error>> {
        let normalized = crate::cli_action::normalize_permission_mode(mode)
            .ok_or_else(|| format!("unsupported mode '{mode}'"))?;
        let session = self.runtime.lock().map_err(|_| "lock")?.session().clone();
        self.permission_mode = crate::cli_action::permission_mode_from_label(normalized);
        let new_runtime = build_runtime(
            session,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            false,
            self.allowed_tools.clone(),
            self.permission_mode,
        )?;
        *self.runtime.lock().map_err(|_| "lock")? = new_runtime;
        self.push_system_message(&format!("权限模式已切换至: {normalized}"));
        Ok(())
    }

    fn switch_session(&mut self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        let handle = crate::session_mgr::resolve_session_reference(target)?;
        let session = runtime::Session::load_from_path(&handle.path)?;
        let new_runtime = build_runtime(
            session,
            self.model.clone(),
            self.system_prompt.clone(),
            true,
            false,
            self.allowed_tools.clone(),
            self.permission_mode,
        )?;
        *self.runtime.lock().map_err(|_| "lock")? = new_runtime;
        self.session_handle = handle;
        self.push_system_message(&format!("已切换至会话 {}", self.session_handle.id));
        Ok(())
    }

    // ── Helper methods ──

    fn wrap_human_intervention(&self, text: &str, interrupted: bool) -> String {
        let lead = if interrupted {
            "用户中断了进行中的轮次，并提交以下新指示。请暂停原任务路径，按新指示重新引导。"
        } else {
            "用户主动发起人机引导。请按以下指示调整当前阶段的方向、材料或检查点。"
        };
        format!("<yunxi_human_intervention>\n{lead}\n</yunxi_human_intervention>\n\n{text}")
    }

    fn open_human_guide(&mut self) {
        self.show_guide = true;
        if self.input.content().trim().is_empty() {
            self.input
                .set_content("【人机引导】请在此输入您的引导或调整指示：\n".to_string());
        }
    }

    fn close_human_guide(&mut self) {
        self.show_guide = false;
    }

    fn sync_theme_palette(&self) {
        crate::tui::ui_palette::active::apply(self.theme_manager.get_theme().clone());
    }

    pub fn cycle_theme(&mut self) -> String {
        self.theme_manager.toggle_theme();
        let name = self.theme_manager.current_name().to_string();
        self.global_state.theme.current_theme = name.clone();
        self.global_state.theme.is_dark = self.theme_manager.get_theme().is_dark;
        self.sync_theme_palette();
        name
    }

    pub fn set_theme_by_name(&mut self, name: &str) -> String {
        self.theme_manager.set_theme(name);
        let n = self.theme_manager.current_name().to_string();
        self.global_state.theme.current_theme = n.clone();
        self.global_state.theme.is_dark = self.theme_manager.get_theme().is_dark;
        self.sync_theme_palette();
        n
    }

    pub fn theme_name(&self) -> &str {
        self.theme_manager.current_name()
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn push_system_message(&mut self, text: &str) {
        self.chat.push(ChatEntry {
            role: ChatRole::System,
            text: text.to_string(),
            reasoning: None,
        });
        self.chat.scroll_to_bottom(10);
    }

    pub fn push_user_message(&mut self, text: &str) {
        self.chat.push(ChatEntry {
            role: ChatRole::User,
            text: text.to_string(),
            reasoning: None,
        });
    }

    pub fn push_assistant_message(&mut self, text: &str) {
        self.chat.push(ChatEntry {
            role: ChatRole::Assistant,
            text: text.to_string(),
            reasoning: None,
        });
        self.chat.scroll_to_bottom(10);
    }

    pub fn append_assistant_text(&mut self, delta: &str) {
        self.chat.append_to_last(delta);
    }

    pub fn set_last_assistant_text(&mut self, text: String) {
        self.chat.set_last_assistant_text(text);
        self.chat.scroll_to_bottom(10);
    }

    pub fn set_thinking(&mut self, thinking: bool) {
        self.thinking = thinking;
    }

    /// 输入区行数（含补全菜单高度）。
    pub fn layout_input_rows(&self) -> usize {
        let menu = self
            .slash_completion
            .as_ref()
            .map(|m| m.matches.len().min(6))
            .unwrap_or(0);
        let content = self.input.content().lines().count().max(1);
        (menu + content + 2).clamp(3, 10)
    }

    fn tick_spinner(&mut self) {
        self.spinner_frame = self.spinner_frame.wrapping_add(1);
    }

    pub fn spinner_glyph(&self) -> &'static str {
        const SPINNER: [&str; 4] = ["⠋", "⠙", "⠹", "⠸"];
        SPINNER[self.spinner_frame % SPINNER.len()]
    }

    fn refresh_slash_completion(&mut self) {
        self.slash_completion =
            crate::tui::slash_complete::SlashCompletion::refresh(self.input.content(), true);
    }

    fn apply_tab_completion(&mut self) -> bool {
        if let Some(menu) = &self.slash_completion {
            if let Some(replacement) = menu.selected_replacement() {
                self.input.set_content(replacement.to_string());
                self.refresh_slash_completion();
                return true;
            }
        }
        false
    }

    fn copy_visible_text_to_clipboard(&mut self) {
        use crate::tui::clipboard::{copy_text_to_clipboard, strip_ansi};
        let text = self.chat.export_plain_conversation();
        if text.trim().is_empty() {
            return;
        }
        let plain = strip_ansi(&text);
        if let Ok(()) = copy_text_to_clipboard(&plain) {
            self.push_system_message(&format!("已复制 {} 个字符到剪贴板", plain.chars().count()));
        }
    }

    fn push_output(&mut self, _title: &str, _body: &str, _width: u16, _height: u16) {
        self.push_system_message(_body);
    }

    pub fn persist_session(&self) {
        let runtime_guard = self.runtime.lock();
        if let Ok(runtime) = runtime_guard {
            let session = runtime.session().clone();
            let _ = crate::session_meta::merge_save_session(
                &self.session_handle.path,
                &session,
                &AthenaSessionMeta {
                    last_routing: None,
                    suspended_flows: vec![],
                },
            );
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.persist_session();
    }
}

// ── Key event types ──

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KeyEvent {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Ctrl(char),
    CtrlShift(char),
    ShiftEnter,
    Esc,
    F(u8),
    Tab,
    Home,
    End,
}

fn convert_key(key: &crossterm::event::KeyEvent) -> KeyEvent {
    use crossterm::event::{KeyCode, KeyModifiers};
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

fn app_key_to_event(key: &KeyEvent) -> Event {
    use crossterm::event::{KeyCode, KeyEvent as CKEvent, KeyModifiers};
    let ck = match key {
        KeyEvent::Char(c) => CKEvent::new(KeyCode::Char(*c), KeyModifiers::empty()),
        KeyEvent::Enter => CKEvent::new(KeyCode::Enter, KeyModifiers::empty()),
        KeyEvent::Esc => CKEvent::new(KeyCode::Esc, KeyModifiers::empty()),
        KeyEvent::Backspace => CKEvent::new(KeyCode::Backspace, KeyModifiers::empty()),
        KeyEvent::Delete => CKEvent::new(KeyCode::Delete, KeyModifiers::empty()),
        KeyEvent::Up => CKEvent::new(KeyCode::Up, KeyModifiers::empty()),
        KeyEvent::Down => CKEvent::new(KeyCode::Down, KeyModifiers::empty()),
        KeyEvent::Left => CKEvent::new(KeyCode::Left, KeyModifiers::empty()),
        KeyEvent::Right => CKEvent::new(KeyCode::Right, KeyModifiers::empty()),
        KeyEvent::Tab => CKEvent::new(KeyCode::Tab, KeyModifiers::empty()),
        KeyEvent::Home => CKEvent::new(KeyCode::Home, KeyModifiers::empty()),
        KeyEvent::End => CKEvent::new(KeyCode::End, KeyModifiers::empty()),
        KeyEvent::ShiftEnter => CKEvent::new(KeyCode::Enter, KeyModifiers::SHIFT),
        KeyEvent::F(n) => CKEvent::new(KeyCode::F(*n), KeyModifiers::empty()),
        KeyEvent::Ctrl(c) => CKEvent::new(KeyCode::Char(*c), KeyModifiers::CONTROL),
        KeyEvent::CtrlShift(c) => CKEvent::new(
            KeyCode::Char(*c),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ),
    };
    Event::Input(InputEvent::Key(ck))
}
