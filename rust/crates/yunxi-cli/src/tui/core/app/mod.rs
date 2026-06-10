//! New-architecture TUI application: combines event loop, state, rendering, and LLM integration.
//! Replaces the old TuiApp + runner.rs + app_ratatui.rs architecture.

mod chat;
mod session;
mod theme;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use runtime::PermissionMode;

use crate::cli_action::AllowedToolSet;
use crate::format_report::{
    render_config_report, render_connect_report, render_conversation_search, render_diff_report,
    render_export_text, render_last_tool_debug_report, render_memory_report,
    render_teleport_report, resolve_export_path,
};
use crate::init::initialize_repo;
use crate::runtime_bridge::{build_runtime, build_system_prompt};
use crate::session_meta::{execute_flow_resume, AthenaSessionMeta};
use crate::session_mgr::{
    create_managed_session_handle, list_managed_sessions, resolve_session_reference, SessionHandle,
};
use crate::slash_sync::{bughunter_prompt, run_commit, run_issue, run_pr, ultraplan_prompt};
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
use crate::tui::turn::{ActiveTurn, SharedRuntime, TurnEvent};
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
    pub router: crate::tui::router::Router,

    // ── Stream state ──
    pub(crate) stream_state_active: bool,
    pub(crate) stream_state_saw_text: bool,
    pub(crate) spinner_frame: usize,

    // ── Pending modal state ──
    pub(crate) pending_permission: Option<runtime::PermissionRequest>,
    pub(crate) pending_flow_hitl: Option<crate::session_meta::SuspendedFlowRecord>,

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
            router: crate::tui::router::Router::new(),

            stream_state_active: false,
            stream_state_saw_text: false,
            spinner_frame: 0,

            pending_permission: None,
            pending_flow_hitl: None,

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

    // ── Public wrappers for runner ──
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
        // ── Reducer 委托链：按优先级尝试各 Reducer ──
        if self.dispatch_theme_action(&action) {
            return Ok(());
        }
        if self.dispatch_session_action(&action)? {
            return Ok(());
        }
        if self.dispatch_chat_action(&action)? {
            return Ok(());
        }

        // ── 剩余 UI 开关 / 导航 / 剪贴板类 Action ──
        match action {
            Action::ShowHelp => {
                self.show_help = true;
            }
            Action::ShowGuide => {
                self.open_human_guide();
            }
            Action::ToggleSidebar => {
                self.show_sidebar = !self.show_sidebar;
            }
            Action::CopySelection => {
                self.copy_visible_text_to_clipboard();
            }
            Action::ShowDialog(name) => match name.as_str() {
                "go_to_top" => self.chat.scroll_to_top(),
                "go_to_bottom" => self.chat.scroll_to_bottom(10),
                "navigate_down" => self.chat.scroll_down(10),
                "navigate_up" => self.chat.scroll_up(),
                _ => {}
            },
            Action::Navigate(route) => {
                self.router.navigate(route);
            }
            Action::GoBack => {
                self.router.go_back();
            }
            Action::GoForward => {
                self.router.go_forward();
            }
            Action::Collapse => {
                self.push_system_message("折叠");
            }
            Action::Expand => {
                self.push_system_message("展开");
            }
            Action::HideDialog | Action::Close => {
                self.show_help = false;
                self.show_guide = false;
                self.command_palette.hide();
                self.session_picker = None;
            }
            Action::SwitchTab(_idx) => {}
            Action::StartSearch => {
                self.push_system_message("搜索模式（开发中）");
            }
            Action::Paste | Action::EditorPaste => {
                match arboard::Clipboard::new().and_then(|mut cb| cb.get_text()) {
                    Ok(text) if !self.has_blocking_modal() => {
                        self.input.set_content(text);
                    }
                    _ => {}
                }
            }
            Action::EditorCopy => {
                let text = self.input.content().to_string();
                if !text.is_empty() {
                    if let Ok(()) = crate::tui::clipboard::copy_text_to_clipboard(&text) {
                        self.push_system_message("已复制输入内容");
                    }
                }
            }
            Action::EditorCut => {
                let text = self.input.content().to_string();
                if !text.is_empty() {
                    let _ = crate::tui::clipboard::copy_text_to_clipboard(&text);
                    self.input = InputBar::new();
                    self.push_system_message("已剪切输入内容");
                }
            }
            Action::EditorUndo => {
                if self.input.undo() {
                    self.refresh_slash_completion();
                }
            }
            Action::EditorRedo => {
                if self.input.redo() {
                    self.refresh_slash_completion();
                }
            }
            Action::Refresh => {
                self.needs_render = true;
            }
            Action::ShowSubmenu(_id, _idx) => {}
            Action::ShowParentMenu(_id) => {}
            Action::Custom(cmd) => {
                self.push_system_message(&format!("自定义动作: {cmd}"));
            }
            Action::Quit => {
                self.should_quit = true;
            }
            _ => {}
        }
        Ok(())
    }

    // ── Key handling ──

    pub fn handle_key(&mut self, key: &KeyEvent) -> Option<Action> {
        // Modal overlays take priority
        if let Some(action) = self.handle_modal_keys(key) {
            return Some(action);
        }

        // Help overlay — any key dismisses
        if self.show_help {
            self.show_help = false;
            return None;
        }

        // Command palette
        if self.command_palette.is_visible() {
            return self.handle_command_palette_key(key);
        }

        // Session picker
        if self.session_picker.is_some() {
            return self.handle_session_picker_key(key);
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

    fn handle_modal_keys(&mut self, key: &KeyEvent) -> Option<Action> {
        if let Some(record) = self.pending_flow_hitl() {
            return match key {
                KeyEvent::Char('y') | KeyEvent::Char('Y') => {
                    let flow_id = record.flow_id.clone();
                    let run_id = record.run_id.clone();
                    self.set_pending_flow_hitl(None);
                    Some(Action::FlowResume(flow_id, run_id))
                }
                KeyEvent::Char('n') | KeyEvent::Char('N') | KeyEvent::Esc => {
                    self.set_pending_flow_hitl(None);
                    None
                }
                _ => None,
            };
        }
        if self.pending_permission.is_some() {
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

    fn handle_session_picker_key(&mut self, key: &KeyEvent) -> Option<Action> {
        match key {
            KeyEvent::Esc | KeyEvent::Ctrl('c') => {
                self.session_picker = None;
            }
            KeyEvent::Enter => {
                if let Some(id) = self
                    .session_picker
                    .as_ref()
                    .and_then(|p| p.selected_session())
                    .map(|s| s.id.clone())
                {
                    self.session_picker = None;
                    return Some(Action::SwitchSession(id));
                }
            }
            KeyEvent::Up | KeyEvent::Char('k') => {
                if let Some(ref mut picker) = self.session_picker {
                    picker.move_up();
                }
            }
            KeyEvent::Down | KeyEvent::Char('j') => {
                if let Some(ref mut picker) = self.session_picker {
                    picker.move_down();
                }
            }
            KeyEvent::Backspace => {
                if let Some(ref mut picker) = self.session_picker {
                    picker.pop_filter_char();
                }
            }
            KeyEvent::Char(c) => {
                if let Some(ref mut picker) = self.session_picker {
                    picker.push_filter_char(*c);
                }
            }
            _ => {}
        }
        None
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
        self.pending_flow_hitl.as_ref()
    }

    pub fn set_pending_flow_hitl(
        &mut self,
        record: Option<crate::session_meta::SuspendedFlowRecord>,
    ) {
        self.pending_flow_hitl = record;
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
        self.pending_permission.as_ref()
    }

    pub fn set_pending_permission(&mut self, request: Option<runtime::PermissionRequest>) {
        self.pending_permission = request;
    }

    // ── Stream handling ──

    fn begin_turn_stream(&mut self) {
        self.stream_state_active = true;
        self.stream_state_saw_text = false;
        // 显示打字指示器（趣味动词轮转）
        let verb = Self::thinking_verb(self.spinner_frame);
        self.push_assistant_message(&format!("● {verb}..."));
    }

    /// 根据帧计数轮转思考动词
    fn thinking_verb(frame: usize) -> &'static str {
        const VERBS: &[&str] = &[
            "思考中",
            "分析中",
            "推理中",
            "检索中",
            "理解中",
            "整理中",
            "生成中",
            "校验中",
        ];
        VERBS[frame % VERBS.len()]
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

            // ── A 类：简单展示命令 ──

            SlashCommand::Config { section } => {
                match render_config_report(section.as_deref()) {
                    Ok(report) => self.push_output("Config", &report, 80, 24),
                    Err(e) => self.push_system_message(&format!("配置加载失败: {e}")),
                }
            }
            SlashCommand::Memory => match render_memory_report() {
                Ok(report) => self.push_output("Memory", &report, 80, 24),
                Err(e) => self.push_system_message(&format!("记忆加载失败: {e}")),
            },
            SlashCommand::Diff => match render_diff_report() {
                Ok(report) => self.push_output("Diff", &report, 80, 24),
                Err(e) => self.push_system_message(&format!("Diff 失败: {e}")),
            },
            SlashCommand::Teleport { target } => {
                let Some(target) = target.as_deref().map(str::trim).filter(|v| !v.is_empty())
                else {
                    self.push_system_message("用法：/teleport <symbol-or-path>");
                    return Ok(true);
                };
                match render_teleport_report(target) {
                    Ok(report) => self.push_output("Teleport", &report, 80, 24),
                    Err(e) => self.push_system_message(&format!("搜索失败: {e}")),
                }
            }
            SlashCommand::DebugToolCall => {
                let report = {
                    let Ok(runtime) = self.runtime.lock() else {
                        self.push_system_message("runtime 锁失败");
                        return Ok(true);
                    };
                    render_last_tool_debug_report(runtime.session())
                };
                match report {
                    Ok(report) => self.push_output("Debug", &report, 80, 24),
                    Err(e) => self.push_system_message(&format!("调试报告失败: {e}")),
                }
            }
            SlashCommand::Connect => match render_connect_report() {
                Ok(report) => self.push_output("Connect", &report, 80, 24),
                Err(e) => self.push_system_message(&format!("连接报告失败: {e}")),
            },
            SlashCommand::Search { query } => {
                let Some(query) = query.as_deref().map(str::trim).filter(|v| !v.is_empty()) else {
                    self.push_system_message("用法：/search <关键词>");
                    return Ok(true);
                };
                let report = {
                    let Ok(runtime) = self.runtime.lock() else {
                        self.push_system_message("runtime 锁失败");
                        return Ok(true);
                    };
                    render_conversation_search(runtime.session(), query)
                };
                self.push_output("Search", &report, 80, 24);
            }

            // ── B 类：文件操作命令 ──

            SlashCommand::Export { path } => {
                let result = {
                    let Ok(runtime) = self.runtime.lock() else {
                        self.push_system_message("runtime 锁失败");
                        return Ok(true);
                    };
                    let session = runtime.session();
                    match resolve_export_path(path.as_deref(), session) {
                        Ok(export_path) => {
                            let text = render_export_text(session);
                            let count = session.messages.len();
                            Some((export_path, text, count))
                        }
                        Err(_) => None,
                    }
                };
                match result {
                    Some((export_path, text, count)) => {
                        match std::fs::write(&export_path, &text) {
                            Ok(()) => self.push_system_message(&format!(
                                "已导出到 {}（{} 条消息）",
                                export_path.display(),
                                count
                            )),
                            Err(e) => self.push_system_message(&format!("写入失败: {e}")),
                        }
                    }
                    None => self.push_system_message("导出路径错误"),
                }
            }
            SlashCommand::Init => {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                match initialize_repo(&cwd) {
                    Ok(report) => self.push_system_message(&report.render()),
                    Err(e) => self.push_system_message(&format!("初始化失败: {e}")),
                }
            }
            SlashCommand::Resume { session_path } => {
                let Some(ref_path) = session_path else {
                    self.push_system_message("用法：/resume <session-id>");
                    return Ok(true);
                };
                match resolve_session_reference(&ref_path) {
                    Ok(handle) => match runtime::Session::load_from_path(&handle.path) {
                        Ok(loaded) => {
                            let count = loaded.messages.len();
                            let new_runtime = build_runtime(
                                loaded,
                                self.model.clone(),
                                self.system_prompt.clone(),
                                true,
                                false,
                                self.allowed_tools.clone(),
                                self.permission_mode,
                            )?;
                            *self.runtime.lock().map_err(|_| "lock")? = new_runtime;
                            self.chat.clear();
                            self.persist_session();
                            self.push_system_message(&format!(
                                "已恢复会话 {}（{} 条消息）",
                                handle.id, count
                            ));
                        }
                        Err(e) => self.push_system_message(&format!("加载会话失败: {e}")),
                    },
                    Err(e) => self.push_system_message(&format!("找不到会话: {e}")),
                }
            }

            // ── C 类：LLM 轮次命令 ──

            SlashCommand::Bughunter { scope } => {
                let prompt = bughunter_prompt(scope.as_deref());
                self.push_user_message(input);
                self.start_llm_turn(&prompt);
            }
            SlashCommand::Ultraplan { task } => {
                let prompt = ultraplan_prompt(task.as_deref());
                self.push_user_message(input);
                self.start_llm_turn(&prompt);
            }
            SlashCommand::Custom { name, arguments } => {
                if let Some(prompt) =
                    commands::resolve_custom_prompt(&name, arguments.as_deref())
                {
                    self.push_user_message(input);
                    self.start_llm_turn(&prompt);
                } else {
                    self.push_system_message(&format!("自定义命令 /{name} 未找到。"));
                }
            }
            SlashCommand::Commit => {
                self.push_system_message("正在生成提交...");
                let result = run_commit(
                    Arc::clone(&self.runtime),
                    self.model.clone(),
                    self.system_prompt.clone(),
                    self.allowed_tools.clone(),
                    self.permission_mode,
                );
                match result {
                    Ok(report) => self.push_output("Commit", &report, 80, 24),
                    Err(e) => self.push_system_message(&format!("提交失败: {e}")),
                }
            }
            SlashCommand::Pr { context } => {
                self.push_system_message("正在生成 PR...");
                let result = run_pr(
                    Arc::clone(&self.runtime),
                    self.model.clone(),
                    self.system_prompt.clone(),
                    self.allowed_tools.clone(),
                    self.permission_mode,
                    context.as_deref(),
                );
                match result {
                    Ok(report) => self.push_output("PR", &report, 80, 24),
                    Err(e) => self.push_system_message(&format!("PR 失败: {e}")),
                }
            }
            SlashCommand::Issue { context } => {
                self.push_system_message("正在生成 Issue...");
                let result = run_issue(
                    Arc::clone(&self.runtime),
                    self.model.clone(),
                    self.system_prompt.clone(),
                    self.allowed_tools.clone(),
                    self.permission_mode,
                    context.as_deref(),
                );
                match result {
                    Ok(report) => self.push_output("Issue", &report, 80, 24),
                    Err(e) => self.push_system_message(&format!("Issue 失败: {e}")),
                }
            }
            SlashCommand::Unknown(name) => {
                self.push_system_message(&format!("未知命令 /{name}，输入 /help 查看帮助。"));
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
                    suspended_flows: self
                        .pending_flow_hitl
                        .as_ref()
                        .map(|r| vec![r.clone()])
                        .unwrap_or_default(),
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

pub(crate) fn convert_key(key: &crossterm::event::KeyEvent) -> KeyEvent {
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
