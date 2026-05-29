#![allow(dead_code, clippy::struct_excessive_bools)]

use runtime::{PermissionRequest, Session};

use crate::session_meta::SuspendedFlowRecord;
use crate::tui::components::chat_view::{ChatEntry, ChatRole, ChatView};
use crate::tui::components::input_bar::{InputBar, INPUT_PROMPT_WIDTH};
use crate::tui::components::session_picker::SessionPicker;
use crate::tui::components::tool_panel::{ToolEntry, ToolPanel};
use crate::tui::frame::{compose_row, fit_lines, truncate_ansi_to_width, Frame};
use crate::tui::hitl::{defer_flow_hitl_overlay, open_human_guide, open_import_prefill};
use crate::tui::layout::{Layout, Rect};
use crate::tui::overlays::ModalOverlays;
use crate::tui::pager::Pager;
use crate::tui::patent::layout::{
    PatentLayout, PATENT_EVIDENCE_HEADER_LINES, PATENT_INPUT_LAYOUT_ROWS,
};
use crate::tui::patent::render::{render_patent_screen, PatentRenderContext};
use crate::tui::patent::session_store::hydrate_workspace;
use crate::tui::patent::workspace::{PatentNav, PatentWorkspace};
use crate::tui::slash_complete::SlashCompletion;
use crate::tui::status_bar::{StatusBar, StatusBarSnapshot};
use crate::tui::ui_mode::UiMode;
use crate::tui::ui_palette::{bold_fg256, fg256, ACCENT, BRAND, BRAND_MARK};

/// TUI 应用主状态。
pub(crate) struct TuiApp {
    /// 对话视图。
    pub(crate) chat: ChatView,
    /// 工具面板。
    pub(crate) tools: ToolPanel,
    /// 输入框。
    pub(crate) input: InputBar,
    /// 帮助覆盖层是否显示。
    pub(crate) show_help: bool,
    /// 工具面板是否显示。
    pub(crate) show_tool_panel: bool,
    /// 是否应该退出。
    should_quit: bool,
    /// 是否正在等待 AI 响应。
    pub(crate) thinking: bool,
    /// 当前模型名。
    pub(crate) model: String,
    /// 版本号。
    pub(crate) version: String,
    ui_mode: UiMode,
    pub patent: PatentWorkspace,
    pub(crate) status: StatusBarSnapshot,
    pub(crate) pager: Option<Pager>,
    pub(crate) session_picker: Option<SessionPicker>,
    pub active_tool: Option<String>,
    pub(crate) turn_output_tokens: u32,
    turn_output_max: u32,
    pub(crate) pending_flow_hitl: Option<SuspendedFlowRecord>,
    /// 人机引导迷你面板（Ctrl+G）。
    pub(crate) show_guide: bool,
    /// 等待用户确认的工具权限。
    pub(crate) pending_permission: Option<PermissionRequest>,
    pub(crate) spinner_frame: usize,
    pub(crate) slash_completion: Option<SlashCompletion>,
    /// 当前轮次是否正在向对话区流式写入。
    turn_streaming: bool,
    stream_saw_text: bool,
    /// 是否在对话区显示模型推理（Reasoning）增量。
    show_reasoning: bool,
}

impl TuiApp {
    pub(crate) fn new(model: String, version: String, ui_mode: UiMode) -> Self {
        Self {
            chat: ChatView::new(),
            tools: ToolPanel::new(),
            input: InputBar::new(),
            show_help: false,
            show_tool_panel: true,
            should_quit: false,
            thinking: false,
            model,
            version,
            ui_mode,
            patent: PatentWorkspace::default(),
            status: StatusBarSnapshot::default(),
            pager: None,
            session_picker: None,
            active_tool: None,
            turn_output_tokens: 0,
            turn_output_max: 0,
            pending_flow_hitl: None,
            show_guide: false,
            pending_permission: None,
            spinner_frame: 0,
            slash_completion: None,
            turn_streaming: false,
            stream_saw_text: false,
            show_reasoning: true,
        }
    }

    /// 清空对话区（/new、/clear --confirm）。
    pub(crate) fn clear_chat(&mut self) {
        self.chat.clear();
    }

    /// 切换推理块显示；返回切换后的状态。
    pub(crate) fn toggle_show_reasoning(&mut self) -> bool {
        self.show_reasoning = !self.show_reasoning;
        self.show_reasoning
    }

    #[must_use]
    pub(crate) fn show_reasoning(&self) -> bool {
        self.show_reasoning
    }

    fn refresh_slash_completion(&mut self) {
        let content = self.input.content();
        self.slash_completion = SlashCompletion::refresh(content, true, self.is_patent_mode());
    }

    fn input_content_rows(&self) -> u16 {
        let mut rows = self.input.content().lines().count().max(1);
        if let Some(menu) = &self.slash_completion {
            rows += menu.matches.len().min(6);
        }
        rows += 1;
        u16::try_from(rows).unwrap_or(2)
    }

    /// 布局用输入区行数（专利专屏固定高度，通用模式动态计算）。
    fn layout_input_rows(&self) -> u16 {
        let menu = self
            .slash_completion
            .as_ref()
            .map(|m| m.matches.len().min(6))
            .unwrap_or(0);
        let content = self.input.content().lines().count().max(1);
        let dynamic = u16::try_from(menu + content + 2).unwrap_or(4).clamp(3, 10);
        if self.is_patent_mode() {
            return dynamic.max(PATENT_INPUT_LAYOUT_ROWS);
        }
        dynamic
    }

    fn render_input_block(&self, width: u16, plain: bool) -> String {
        let rows = self.layout_input_rows();
        let area = Rect::new(0, 0, width, crate::tui::layout::input_block_height(rows));
        if plain {
            self.input
                .render_plain(area, self.slash_completion.as_ref())
        } else {
            self.input
                .render_with_completion(area, self.slash_completion.as_ref())
        }
    }

    fn apply_tab_completion(&mut self) -> bool {
        if let Some(menu) = &self.slash_completion {
            if let Some(replacement) = menu.selected_replacement() {
                self.input.set_content(replacement.to_string());
                self.refresh_slash_completion();
                return true;
            }
        }
        if SlashCompletion::refresh(self.input.content(), true, self.is_patent_mode()).is_some() {
            self.refresh_slash_completion();
            return true;
        }
        false
    }

    fn handle_input_navigation(&mut self, key: &KeyEvent) -> bool {
        let Some(menu) = self.slash_completion.as_mut() else {
            return false;
        };
        match key {
            KeyEvent::Up => menu.move_up(),
            KeyEvent::Down => menu.move_down(),
            _ => return false,
        }
        true
    }

    #[must_use]
    pub(crate) fn is_patent_mode(&self) -> bool {
        self.ui_mode == UiMode::Patent
    }

    #[must_use]
    pub(crate) fn model(&self) -> &str {
        &self.model
    }

    pub(crate) fn set_model(&mut self, model: String) {
        self.model = model;
    }

    pub(crate) fn set_input_content(&mut self, text: String) {
        self.input.set_content(text);
        self.refresh_slash_completion();
    }

    pub(crate) fn reset_turn_progress(&mut self) {
        self.turn_output_tokens = 0;
        self.turn_output_max = 4096;
    }

    #[must_use]
    pub(crate) fn is_thinking(&self) -> bool {
        self.thinking
    }

    pub(crate) fn turn_progress(&self) -> (u32, u32) {
        (self.turn_output_tokens, self.turn_output_max)
    }

    pub(crate) fn update_status(&mut self, snapshot: StatusBarSnapshot) {
        self.status = snapshot;
    }

    pub(crate) fn push_system_message(&mut self, text: &str) {
        self.chat.push(ChatEntry {
            role: ChatRole::Assistant,
            text: text.to_string(),
        });
        self.scroll_patent_chat_to_latest();
    }

    pub(crate) fn push_output(&mut self, title: &str, body: &str, width: u16, height: u16) {
        let visible = height.saturating_sub(4).max(8) as usize;
        self.pager = Some(Pager::new(title, body, width.max(40) as usize));
        let _ = visible;
    }

    pub(crate) fn open_help(&mut self) {
        self.show_help = true;
    }

    pub(crate) fn open_session_picker(
        &mut self,
        sessions: Vec<crate::session_mgr::ManagedSessionSummary>,
        active_session_id: String,
    ) {
        self.session_picker = Some(SessionPicker::new(sessions, active_session_id));
    }

    pub(crate) fn set_pending_flow_hitl(&mut self, record: Option<SuspendedFlowRecord>) {
        self.pending_flow_hitl = record;
    }

    #[must_use]
    pub(crate) fn pending_flow_hitl(&self) -> Option<&SuspendedFlowRecord> {
        self.pending_flow_hitl.as_ref()
    }

    pub(crate) fn set_show_guide(&mut self, show: bool) {
        self.show_guide = show;
    }

    #[must_use]
    pub(crate) fn show_guide(&self) -> bool {
        self.show_guide
    }

    pub(crate) fn set_pending_permission(&mut self, request: Option<PermissionRequest>) {
        self.pending_permission = request;
    }

    #[must_use]
    pub(crate) fn pending_permission(&self) -> Option<&PermissionRequest> {
        self.pending_permission.as_ref()
    }

    #[must_use]
    pub(crate) fn input_content(&self) -> &str {
        self.input.content()
    }

    #[must_use]
    pub(crate) fn chat_scroll_offset(&self) -> usize {
        self.chat.scroll_offset()
    }

    #[must_use]
    pub(crate) fn chat_transcript(&self) -> String {
        self.chat.transcript_text()
    }

    pub(crate) fn has_blocking_modal(&self) -> bool {
        self.pending_flow_hitl.is_some() || self.pending_permission.is_some()
    }

    pub(crate) fn reload_patent_from_session(
        &mut self,
        _session_id: &str,
        session_path: &std::path::Path,
        session: &Session,
    ) {
        hydrate_workspace(&mut self.patent, session_path, session);
    }

    pub(crate) fn tick_spinner(&mut self) {
        self.spinner_frame = self.spinner_frame.wrapping_add(1);
    }

    const SPINNER: [&str; 4] = ["⠋", "⠙", "⠹", "⠸"];

    fn spinner_glyph(&self) -> &'static str {
        Self::SPINNER[self.spinner_frame % Self::SPINNER.len()]
    }

    fn patent_input_rendered(&self, width: u16) -> String {
        self.render_input_block(width, true)
    }

    /// 处理按键事件，返回用户提交的输入（如有）。
    pub(crate) fn handle_key(&mut self, key: &KeyEvent) -> Option<TuiAction> {
        if let Some(action) = self.handle_modal_keys(key) {
            return Some(action);
        }

        if self.show_guide && matches!(key, KeyEvent::Esc) {
            self.show_guide = false;
            return None;
        }

        if self.pager.is_some() {
            match key {
                KeyEvent::Esc | KeyEvent::Char('q') => {
                    self.pager = None;
                }
                KeyEvent::Char('j') | KeyEvent::Down => {
                    if let Some(p) = &mut self.pager {
                        p.scroll_down(1, 20);
                    }
                }
                KeyEvent::Char('k') | KeyEvent::Up => {
                    if let Some(p) = &mut self.pager {
                        p.scroll_up(1);
                    }
                }
                _ => {}
            }
            return None;
        }

        if self.show_help {
            self.show_help = false;
            return None;
        }

        if self.is_patent_mode() && self.handle_patent_key(key) {
            return None;
        }

        match key {
            // 退出
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
            // 发送
            KeyEvent::Enter if !self.input.is_empty() && !self.has_blocking_modal() => {
                let text = self.input.take();
                let as_intervention =
                    crate::tui::hitl::is_human_intervention_message(&text) || self.show_guide;
                if self.show_guide {
                    self.show_guide = false;
                }
                self.chat.push(ChatEntry {
                    role: ChatRole::User,
                    text: text.clone(),
                });
                self.scroll_patent_chat_to_latest();
                Some(TuiAction::Submit {
                    text,
                    as_intervention,
                    interrupted_turn: false,
                })
            }
            // 帮助
            KeyEvent::Ctrl('h') | KeyEvent::F(1) => {
                self.open_help();
                None
            }
            KeyEvent::Ctrl('g') => {
                open_human_guide(self);
                None
            }
            KeyEvent::Ctrl('i') => Some(TuiAction::InterruptTurn),
            KeyEvent::Ctrl('u') => {
                open_import_prefill(self);
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
            // 切换工具面板
            KeyEvent::F(2) => {
                self.show_tool_panel = !self.show_tool_panel;
                None
            }
            // 滚动（补全菜单打开时 ↑↓ 用于选择候选）
            KeyEvent::Up | KeyEvent::Down if self.slash_completion.is_some() => {
                let _ = self.handle_input_navigation(key);
                None
            }
            KeyEvent::Char('j') | KeyEvent::Down if self.input.is_empty() => {
                self.chat.scroll_down(10);
                None
            }
            KeyEvent::Char('k') | KeyEvent::Up if self.input.is_empty() => {
                self.chat.scroll_up();
                None
            }
            KeyEvent::Char('g') if self.input.is_empty() => {
                self.chat.scroll_to_top();
                None
            }
            KeyEvent::Char('G') if self.input.is_empty() => {
                self.chat.scroll_to_bottom(10);
                None
            }
            // 输入框操作
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
            _ => None,
        }
    }

    fn handle_modal_keys(&mut self, key: &KeyEvent) -> Option<TuiAction> {
        if self.pending_flow_hitl.is_some() {
            return match key {
                KeyEvent::Char('y') | KeyEvent::Char('Y') => {
                    let (flow_id, run_id) = (
                        self.pending_flow_hitl.as_ref()?.flow_id.clone(),
                        self.pending_flow_hitl.as_ref()?.run_id.clone(),
                    );
                    Some(TuiAction::FlowResume { flow_id, run_id })
                }
                KeyEvent::Char('n') | KeyEvent::Char('N') | KeyEvent::Esc => {
                    defer_flow_hitl_overlay(self);
                    None
                }
                _ => None,
            };
        }

        if self.pending_permission.is_some() {
            return match key {
                KeyEvent::Char('y') | KeyEvent::Char('Y') => {
                    Some(TuiAction::PermissionDecision(true))
                }
                KeyEvent::Char('n') | KeyEvent::Char('N') | KeyEvent::Esc | KeyEvent::Ctrl('c') => {
                    Some(TuiAction::PermissionDecision(false))
                }
                _ => None,
            };
        }

        None
    }

    fn paint_modal_overlays(&self, frame: &mut Frame, width: u16, height: u16, patent_help: bool) {
        ModalOverlays {
            width,
            height,
            show_help: self.show_help,
            patent_help,
            show_guide: self.show_guide,
            thinking: self.thinking,
            pending_flow_hitl: self.pending_flow_hitl.as_ref(),
            pending_permission: self.pending_permission.as_ref(),
        }
        .paint(frame);
    }

    /// 设置思考状态。
    pub(crate) fn set_thinking(&mut self, thinking: bool) {
        self.thinking = thinking;
    }

    /// 添加用户消息（不触发 submit）。
    pub(crate) fn push_user_message(&mut self, text: &str) {
        self.chat.push(ChatEntry {
            role: ChatRole::User,
            text: text.to_string(),
        });
    }

    /// 追加助手回复。
    pub(crate) fn push_assistant_message(&mut self, text: &str) {
        self.chat.push(ChatEntry {
            role: ChatRole::Assistant,
            text: text.to_string(),
        });
        self.scroll_patent_chat_to_latest();
    }

    /// 开始流式回复：预置一条空的 assistant 消息。
    pub(crate) fn begin_turn_stream(&mut self) {
        self.turn_streaming = true;
        self.stream_saw_text = false;
        self.push_assistant_message("");
        if self.is_patent_mode() {
            self.patent.select_nav(PatentNav::Assistant);
            self.chat.scroll_to_bottom(24);
        }
    }

    #[must_use]
    pub(crate) fn turn_was_streamed(&self) -> bool {
        self.turn_streaming
    }

    pub(crate) fn end_turn_stream(&mut self) {
        self.turn_streaming = false;
        self.stream_saw_text = false;
    }

    pub(crate) fn append_stream_event(&mut self, event: &runtime::AssistantEvent) {
        use runtime::AssistantEvent;

        match event {
            AssistantEvent::ReasoningDelta(delta) => {
                if self.show_reasoning {
                    self.append_assistant_text(delta);
                }
            }
            AssistantEvent::TextDelta(delta) => {
                if !self.stream_saw_text && self.chat.last_assistant_has_content() {
                    self.append_assistant_text("\n\n");
                }
                self.stream_saw_text = true;
                self.append_assistant_text(delta);
            }
            AssistantEvent::ToolUse { name, .. } => {
                self.append_assistant_text(&format!("\n\n▸ 调用 {name}…\n"));
                self.active_tool = Some(name.clone());
            }
            AssistantEvent::Usage(_) | AssistantEvent::MessageStop => {}
        }
    }

    /// 追加助手回复的增量文本（流式）。
    pub(crate) fn append_assistant_text(&mut self, delta: &str) {
        self.chat.append_to_last(delta);
        if self.is_patent_mode() {
            self.chat.scroll_to_bottom(24);
        }
    }

    pub(crate) fn set_last_assistant_text(&mut self, text: String) {
        self.chat.set_last_assistant_text(text);
        self.scroll_patent_chat_to_latest();
    }

    #[must_use]
    pub(crate) fn last_assistant_text_is_empty(&self) -> bool {
        !self.chat.last_assistant_has_content()
    }

    /// 追加工具调用记录。
    pub(crate) fn push_tool_entry(&mut self, entry: ToolEntry) {
        self.tools.push(entry);
    }

    fn handle_patent_key(&mut self, key: &KeyEvent) -> bool {
        match key {
            KeyEvent::Char(c) if ('1'..='6').contains(c) => {
                if let Some(nav) = PatentNav::from_digit(*c) {
                    self.patent.select_nav(nav);
                    if nav == PatentNav::Assistant {
                        self.chat.scroll_to_bottom(24);
                    }
                }
                true
            }
            KeyEvent::F(3) => {
                let nav = self.patent.nav.next();
                self.patent.select_nav(nav);
                if nav == PatentNav::Assistant {
                    self.chat.scroll_to_bottom(24);
                }
                true
            }
            KeyEvent::F(2) | KeyEvent::Char('o') => {
                self.patent.evidence_collapsed = !self.patent.evidence_collapsed;
                true
            }
            KeyEvent::Char('[') if self.input.is_empty() && !self.patent.evidence_collapsed => {
                self.tools.scroll_up();
                true
            }
            KeyEvent::Char(']') if self.input.is_empty() && !self.patent.evidence_collapsed => {
                self.tools.scroll_down(1, 40);
                true
            }
            KeyEvent::Char('j') | KeyEvent::Down if self.input.is_empty() => {
                if self.patent.nav == PatentNav::Assistant {
                    self.chat.scroll_down(12);
                } else {
                    self.patent.main_scroll = self.patent.main_scroll.saturating_add(1);
                }
                true
            }
            KeyEvent::Char('k') | KeyEvent::Up if self.input.is_empty() => {
                if self.patent.nav == PatentNav::Assistant {
                    self.chat.scroll_up();
                } else {
                    self.patent.main_scroll = self.patent.main_scroll.saturating_sub(1);
                }
                true
            }
            KeyEvent::Char('g') if self.input.is_empty() => {
                if self.patent.nav == PatentNav::Assistant {
                    self.chat.scroll_to_top();
                } else {
                    self.patent.main_scroll = 0;
                }
                true
            }
            KeyEvent::Char('G') if self.input.is_empty() => {
                if self.patent.nav == PatentNav::Assistant {
                    self.chat.scroll_to_bottom(12);
                } else {
                    self.patent.main_scroll = usize::MAX;
                }
                true
            }
            _ => false,
        }
    }

    /// 渲染完整界面到 ANSI 字符串。
    pub(crate) fn render(&self, width: u16, height: u16) -> String {
        self.render_with_cursor(width, height)
    }

    /// 渲染界面并将终端光标置于输入行，便于 IME 候选窗定位。
    pub(crate) fn render_with_cursor(&self, width: u16, height: u16) -> String {
        let mut rendered = if self.is_patent_mode() {
            self.render_patent(width, height)
        } else {
            self.render_general(width, height)
        };
        if let Some((row, col)) = self.input_cursor_pos(width, height) {
            rendered.push_str(&format!("\x1b[{row};{col}H"));
        }
        rendered
    }

    fn input_cursor_pos(&self, width: u16, height: u16) -> Option<(u16, u16)> {
        if self.has_blocking_modal() || self.pager.is_some() || self.show_help {
            return None;
        }
        let input_bar = if self.is_patent_mode() {
            PatentLayout::compute(
                width,
                height,
                self.layout_input_rows(),
                self.patent.evidence_collapsed,
            )
            .input_bar
        } else {
            self.general_layout(width, height).input_bar
        };
        if !input_bar.is_valid() {
            return None;
        }
        let line_in_block = InputBar::input_line_index(self.slash_completion.as_ref());
        let row = input_bar.y.saturating_add(line_in_block).saturating_add(1);
        let col = input_bar
            .x
            .saturating_add(INPUT_PROMPT_WIDTH)
            .saturating_add(self.input.cursor_visible_col())
            .saturating_add(1);
        Some((row.min(height), col.min(width)))
    }

    /// 通用对话布局：使用帧缓冲按 (x,y) 合成，避免换行导致全屏错乱。
    fn render_general(&self, width: u16, height: u16) -> String {
        let input_rows = self.layout_input_rows();
        let layout = if self.show_tool_panel {
            Layout::compute_with_input_rows(width, height, input_rows, true)
        } else {
            Layout::compute_with_input_rows(width, height, input_rows, false)
        };

        let mut frame = Frame::new(width, height);

        frame.set_row(layout.title_bar.y, &self.render_title_bar(&layout));

        let chat_body = self.chat.render(layout.chat_view);
        let chat_lines = fit_lines(
            &chat_body,
            layout.chat_view.width as usize,
            layout.chat_view.height as usize,
        );
        let tool_lines = if layout.tool_panel.is_valid() {
            fit_lines(
                &self.tools.render(layout.tool_panel),
                layout.tool_panel.width as usize,
                layout.tool_panel.height as usize,
            )
        } else {
            Vec::new()
        };

        for i in 0..layout.chat_view.height as usize {
            let y = layout.chat_view.y.saturating_add(i as u16);
            let left = chat_lines.get(i).map(String::as_str).unwrap_or("");
            let row = if layout.tool_panel.is_valid() {
                let right = tool_lines.get(i).map(String::as_str).unwrap_or("");
                compose_row(left, layout.chat_view.width, right, layout.tool_panel.width)
            } else {
                crate::tui::frame::pad_ansi_line(left, width)
            };
            frame.set_row(y, &row);
        }

        frame.paint_area(layout.input_bar, &self.render_input_block(width, false));

        let status_line = StatusBar::with_width(layout.status_bar.width).render(&self.status);
        frame.set_row(
            layout.status_bar.y,
            &truncate_ansi_to_width(&status_line, layout.status_bar.width as usize),
        );

        self.paint_modal_overlays(&mut frame, width, height, false);

        frame.as_ansi()
    }

    fn render_patent(&self, width: u16, height: u16) -> String {
        let layout = PatentLayout::compute(
            width,
            height,
            self.layout_input_rows(),
            self.patent.evidence_collapsed,
        );
        let ctx = PatentRenderContext {
            workspace: &self.patent,
            chat: &self.chat,
            tools: &self.tools,
            status: &self.status,
            version: &self.version,
            thinking: self.thinking,
            active_tool: self.active_tool.as_deref(),
            spinner_frame: self.spinner_glyph(),
            input_rendered: self.patent_input_rendered(width),
            show_help: self.show_help,
            show_guide: self.show_guide,
            pending_flow_hitl: self.pending_flow_hitl.as_ref(),
            pending_permission: self.pending_permission.as_ref(),
        };
        let mut frame = render_patent_screen(&layout, &ctx, width, height);
        if let Some(pager) = &self.pager {
            let overlay = Rect::new(width / 8, height / 6, width * 3 / 4, height * 2 / 3);
            let lines: Vec<String> = pager
                .render(overlay.height as usize)
                .lines()
                .map(String::from)
                .collect();
            // pager overlay via second pass would need Frame API — append hint in title bar for v1
            let _ = overlay;
            let _ = lines;
            frame.push_str("\n\x1b[2m[分页器打开 — Esc 关闭]\x1b[0m");
        }
        frame
    }

    /// 渲染标题栏。
    fn render_title_bar(&self, layout: &Layout) -> String {
        let thinking_indicator = if self.thinking {
            format!(" {}", fg256(ACCENT, "⠋ 思考中…"))
        } else {
            String::new()
        };
        let title = format!(
            " {} {}  {}  {}{thinking_indicator}",
            bold_fg256(BRAND, "云熙智能体"),
            crate::tui::ui_palette::dim(&format!("v{}", self.version)),
            fg256(ACCENT, BRAND_MARK),
            fg256(ACCENT, &self.model),
        );
        let width = layout.title_bar.width as usize;
        truncate_ansi_to_width(&title, width)
    }

    /// 是否应该退出。
    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }

    fn general_layout(&self, width: u16, height: u16) -> Layout {
        let input_rows = self.layout_input_rows();
        if self.show_tool_panel {
            Layout::compute_with_input_rows(width, height, input_rows, true)
        } else {
            Layout::compute_with_input_rows(width, height, input_rows, false)
        }
    }

    fn scroll_patent_chat_to_latest(&mut self) {
        if self.is_patent_mode() && self.patent.nav == PatentNav::Assistant {
            self.chat.scroll_to_bottom(24);
        }
    }

    fn patent_layout(&self, width: u16, height: u16) -> PatentLayout {
        PatentLayout::compute(
            width,
            height,
            self.layout_input_rows(),
            self.patent.evidence_collapsed,
        )
    }

    /// 处理鼠标事件（滚轮滚动、工具面板左键折叠）。
    pub(crate) fn handle_mouse(
        &mut self,
        col: u16,
        row: u16,
        action: MouseAction,
        width: u16,
        height: u16,
    ) {
        if self.has_blocking_modal()
            || self.pager.is_some()
            || self.show_help
            || self.session_picker.is_some()
        {
            return;
        }

        if self.is_patent_mode() {
            self.handle_patent_mouse(col, row, action, width, height);
        } else {
            self.handle_general_mouse(col, row, action, width, height);
        }
    }

    fn handle_general_mouse(
        &mut self,
        col: u16,
        row: u16,
        action: MouseAction,
        width: u16,
        height: u16,
    ) {
        let layout = self.general_layout(width, height);
        let chat_h = layout.chat_view.height as usize;

        match action {
            MouseAction::ScrollUp => {
                if layout.chat_view.contains(col, row) {
                    self.chat.scroll_up_by(MOUSE_WHEEL_LINES);
                } else if layout.tool_panel.is_valid() && layout.tool_panel.contains(col, row) {
                    self.tools.scroll_up_by(MOUSE_WHEEL_LINES);
                }
            }
            MouseAction::ScrollDown => {
                if layout.chat_view.contains(col, row) {
                    self.chat.scroll_down_by(MOUSE_WHEEL_LINES, chat_h);
                } else if layout.tool_panel.is_valid() && layout.tool_panel.contains(col, row) {
                    let panel_w = layout.tool_panel.width as usize;
                    let panel_h = layout.tool_panel.height as usize;
                    self.tools
                        .scroll_down_by(MOUSE_WHEEL_LINES, panel_h, panel_w);
                }
            }
            MouseAction::LeftClick => {
                if layout.tool_panel.is_valid() && layout.tool_panel.contains(col, row) {
                    self.toggle_tool_entry_at(col, row, &layout.tool_panel);
                }
            }
        }
    }

    fn handle_patent_mouse(
        &mut self,
        col: u16,
        row: u16,
        action: MouseAction,
        width: u16,
        height: u16,
    ) {
        let layout = self.patent_layout(width, height);

        match action {
            MouseAction::ScrollUp => {
                if layout.main_panel.contains(col, row) {
                    if self.patent.nav == PatentNav::Assistant {
                        self.chat.scroll_up_by(MOUSE_WHEEL_LINES);
                    } else {
                        self.patent.main_scroll =
                            self.patent.main_scroll.saturating_sub(MOUSE_WHEEL_LINES);
                    }
                } else if layout.evidence_panel.is_valid()
                    && layout.evidence_panel.contains(col, row)
                {
                    self.tools.scroll_up_by(MOUSE_WHEEL_LINES);
                }
            }
            MouseAction::ScrollDown => {
                if layout.main_panel.contains(col, row) {
                    if self.patent.nav == PatentNav::Assistant {
                        self.chat.scroll_down_by(MOUSE_WHEEL_LINES, 12);
                    } else {
                        self.patent.main_scroll =
                            self.patent.main_scroll.saturating_add(MOUSE_WHEEL_LINES);
                    }
                } else if layout.evidence_panel.is_valid()
                    && layout.evidence_panel.contains(col, row)
                {
                    let panel_w = layout.evidence_panel.width as usize;
                    let panel_h = layout
                        .evidence_panel
                        .height
                        .saturating_sub(PATENT_EVIDENCE_HEADER_LINES)
                        as usize;
                    self.tools
                        .scroll_down_by(MOUSE_WHEEL_LINES, panel_h.max(1), panel_w);
                }
            }
            MouseAction::LeftClick => {
                if layout.evidence_panel.is_valid() && layout.evidence_panel.contains(col, row) {
                    self.toggle_tool_entry_at(col, row, &layout.evidence_panel);
                }
            }
        }
    }

    fn toggle_tool_entry_at(&mut self, col: u16, row: u16, panel: &Rect) {
        let rel_row = row.saturating_sub(panel.y) as usize;
        let header_skip = if self.is_patent_mode() {
            usize::from(PATENT_EVIDENCE_HEADER_LINES)
        } else {
            0
        };
        if rel_row < header_skip {
            return;
        }
        let line = self.tools.scroll_offset() + rel_row.saturating_sub(header_skip);
        let wrap_width = panel.width as usize;
        if let Some(idx) = self.tools.entry_at_rendered_line(line, wrap_width) {
            self.tools.toggle_collapse(idx);
        }
        let _ = col;
    }
}

const MOUSE_WHEEL_LINES: usize = 3;

/// 鼠标动作（与 crossterm 解耦，方便测试）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MouseAction {
    ScrollUp,
    ScrollDown,
    LeftClick,
}

/// 简易按键事件（与 crossterm 解耦，方便测试）。
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
    ShiftEnter,
    Esc,
    F(u8),
    Tab,
}

/// TUI 动作（按键处理后返回的结果）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TuiAction {
    /// 用户提交了消息。
    Submit {
        text: String,
        /// 是否包装为主动干预（引导/中断后重定向）。
        as_intervention: bool,
        /// 发送前是否中断了进行中的轮次。
        interrupted_turn: bool,
    },
    /// 恢复挂起的工作流（Flow HITL 覆盖层按 y）。
    FlowResume { flow_id: String, run_id: String },
    /// 工具权限：允许或拒绝。
    PermissionDecision(bool),
    /// 中断当前轮次（Ctrl+I）；完成后由 runner 打开引导面板。
    InterruptTurn,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> TuiApp {
        TuiApp::new(
            "deepseek-v4-pro".to_string(),
            "0.1.0".to_string(),
            UiMode::General,
        )
    }

    #[test]
    fn app_handles_enter_submit() {
        let mut app = test_app();
        app.input.insert('h');
        app.input.insert('i');
        let action = app.handle_key(&KeyEvent::Enter);
        assert_eq!(
            action,
            Some(TuiAction::Submit {
                text: "hi".to_string(),
                as_intervention: false,
                interrupted_turn: false,
            })
        );
        assert!(app.input.is_empty());
    }

    #[test]
    fn app_handles_quit() {
        let mut app = test_app();
        app.handle_key(&KeyEvent::Char('q'));
        assert!(app.should_quit());
    }

    #[test]
    fn app_handles_help_toggle() {
        let mut app = test_app();
        app.handle_key(&KeyEvent::Ctrl('h'));
        assert!(app.show_help);
        // 任意键关闭
        app.handle_key(&KeyEvent::Char('a'));
        assert!(!app.show_help);
    }

    #[test]
    fn app_ctrl_g_opens_guide_with_template() {
        let mut app = test_app();
        app.handle_key(&KeyEvent::Ctrl('g'));
        assert!(app.show_guide());
        assert!(app.input_content().contains("【人机引导】"));
    }

    #[test]
    fn app_ctrl_u_prefills_import() {
        let mut app = test_app();
        app.handle_key(&KeyEvent::Ctrl('u'));
        assert_eq!(app.input_content(), "/import ");
    }

    #[test]
    fn app_flow_hitl_y_resumes() {
        let mut app = test_app();
        app.set_pending_flow_hitl(Some(SuspendedFlowRecord {
            flow_id: "f1".into(),
            run_id: "r1".into(),
            noted_at: String::new(),
            flow_name: None,
            current_step: None,
            step_title: Some("确认".into()),
            step_description: None,
        }));
        let action = app.handle_key(&KeyEvent::Char('y'));
        assert_eq!(
            action,
            Some(TuiAction::FlowResume {
                flow_id: "f1".into(),
                run_id: "r1".into(),
            })
        );
    }

    #[test]
    fn app_flow_hitl_n_defers_overlay() {
        let mut app = test_app();
        app.set_pending_flow_hitl(Some(SuspendedFlowRecord {
            flow_id: "f1".into(),
            run_id: "r1".into(),
            noted_at: String::new(),
            flow_name: None,
            current_step: None,
            step_title: None,
            step_description: None,
        }));
        assert!(app.handle_key(&KeyEvent::Char('n')).is_none());
        assert!(app.pending_flow_hitl().is_none());
    }

    #[test]
    fn app_handles_tool_panel_toggle() {
        let mut app = test_app();
        let initial = app.show_tool_panel;
        app.handle_key(&KeyEvent::F(2));
        assert_eq!(app.show_tool_panel, !initial);
    }

    #[test]
    fn app_push_assistant_and_tool() {
        let mut app = test_app();
        app.push_assistant_message("你好！");
        app.push_tool_entry(ToolEntry {
            name: "bash".to_string(),
            detail: "$ echo hi\nhi".to_string(),
            is_error: false,
            collapsed: false,
        });
        assert_eq!(app.chat.len(), 1);
        assert_eq!(app.tools.len(), 1);
    }

    #[test]
    fn app_render_produces_output() {
        let app = test_app();
        let rendered = app.render(80, 24);
        assert!(rendered.contains("云熙智能体"));
        assert!(rendered.contains("deepseek-v4-pro"));
        assert!(rendered.contains("\x1b[1;1H"));
        assert!(rendered.contains("工具输出面板") || rendered.contains("Shift+Enter"));
    }

    #[test]
    fn app_backspace_in_input() {
        let mut app = test_app();
        app.input.insert('a');
        app.input.insert('b');
        app.handle_key(&KeyEvent::Backspace);
        assert_eq!(app.input.content(), "a");
    }

    #[test]
    fn truncate_ansi_preserves_escapes() {
        let s = "\x1b[1mhello\x1b[0m world";
        let truncated = truncate_ansi_to_width(s, 8);
        assert!(truncated.contains("\x1b[1m"));
        assert!(truncated.contains("hello"));
    }

    #[test]
    fn truncate_ansi_truncates_long_text() {
        let truncated = truncate_ansi_to_width("abcdefghij", 5);
        assert_eq!(truncated, "abcde");
    }

    #[test]
    fn app_ctrl_f_prefills_search_in_general_mode() {
        let mut app = test_app();
        app.handle_key(&KeyEvent::Ctrl('f'));
        assert_eq!(app.input_content(), "/search ");
    }

    #[test]
    fn app_mouse_scrolls_chat() {
        let mut app = test_app();
        for i in 0..50 {
            app.push_user_message(&format!("消息 {i}"));
        }
        let layout = Layout::compute_with_input_rows(80, 24, app.layout_input_rows(), true);
        assert_eq!(app.chat_scroll_offset(), 0);
        app.handle_mouse(
            layout.chat_view.x + 1,
            layout.chat_view.y + 1,
            MouseAction::ScrollDown,
            80,
            24,
        );
        assert!(app.chat_scroll_offset() > 0);
    }

    #[test]
    fn app_patent_bracket_scrolls_evidence() {
        let mut app = TuiApp::new(
            "deepseek-v4-pro".to_string(),
            "0.1.0".to_string(),
            UiMode::Patent,
        );
        app.push_tool_entry(ToolEntry {
            name: "read_file".to_string(),
            detail: (0..30)
                .map(|i| format!("line {i}"))
                .collect::<Vec<_>>()
                .join("\n"),
            is_error: false,
            collapsed: false,
        });
        assert_eq!(app.tools.scroll_offset(), 0);
        app.handle_key(&KeyEvent::Char(']'));
        assert!(app.tools.scroll_offset() > 0);
        app.handle_key(&KeyEvent::Char('['));
        assert_eq!(app.tools.scroll_offset(), 0);
    }
}
