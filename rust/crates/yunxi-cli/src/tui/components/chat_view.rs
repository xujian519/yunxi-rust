#![allow(dead_code)]

use crate::render::TerminalRenderer;
use crate::tui::components::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::Event;
use crate::tui::frame::{truncate_ansi_to_width, visible_width, wrap_ansi_to_width};
use crate::tui::layout::Rect;

fn text_contains_ansi(text: &str) -> bool {
    text.contains("\x1b[")
}

/// 将条目正文转为可显示的 ANSI 文本（专利 plain 模式或已含 ANSI 时保持原样）。
fn display_body(role: ChatRole, text: &str, plain: bool, renderer: &TerminalRenderer) -> String {
    if plain || text_contains_ansi(text) {
        return text.to_string();
    }
    match role {
        ChatRole::Assistant | ChatRole::System => renderer.markdown_to_ansi(text),
        ChatRole::User => text.to_string(),
    }
}

/// 对话消息条目。
#[derive(Debug, Clone)]
pub(crate) struct ChatEntry {
    pub role: ChatRole,
    pub text: String,
    /// AI 推理过程（reasoning_delta），与最终回答分离存储。
    pub reasoning: Option<String>,
}

/// 消息角色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChatRole {
    User,
    Assistant,
    System,
}

impl ChatRole {
    /// 显示标签。
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::User => "你",
            Self::Assistant => "云熙",
            Self::System => "系统",
        }
    }
}

/// 对话历史视图（可滚动）。
pub(crate) struct ChatView {
    entries: Vec<ChatEntry>,
    scroll_offset: usize,
    /// 用户是否手动向上滚动（暂停自动跟随）
    user_scrolled: bool,
    /// 用户手动滚动时，未读的新消息数量
    unread_count: usize,
    state: ComponentState,
}

impl ChatView {
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            scroll_offset: 0,
            user_scrolled: false,
            unread_count: 0,
            state: ComponentState::new(generate_component_id("chat_view")),
        }
    }

    /// 追加一条消息。
    pub(crate) fn push(&mut self, entry: ChatEntry) {
        self.entries.push(entry);
    }

    /// 追加纯文本到最新 assistant 条目末尾（用于流式增量）。
    pub(crate) fn append_to_last(&mut self, text: &str) {
        if let Some(last) = self.entries.last_mut() {
            if last.role == ChatRole::Assistant {
                last.text.push_str(text);
            }
        }
    }

    /// 追加 reasoning 文本到最新 assistant 条目（用于流式推理增量）。
    pub(crate) fn append_reasoning_to_last(&mut self, text: &str) {
        if let Some(last) = self.entries.last_mut() {
            if last.role == ChatRole::Assistant {
                match &mut last.reasoning {
                    Some(r) => r.push_str(text),
                    None => last.reasoning = Some(text.to_string()),
                }
            }
        }
    }

    #[must_use]
    pub(crate) fn last_assistant_has_content(&self) -> bool {
        self.entries
            .last()
            .filter(|entry| entry.role == ChatRole::Assistant)
            .is_some_and(|entry| {
                !entry.text.is_empty() || entry.reasoning.as_ref().is_some_and(|r| !r.is_empty())
            })
    }

    pub(crate) fn last_entry(&self) -> Option<&ChatEntry> {
        self.entries.last()
    }

    pub(crate) fn set_last_assistant_text(&mut self, text: String) {
        if let Some(last) = self.entries.last_mut() {
            if last.role == ChatRole::Assistant {
                last.text = text;
            }
        }
    }

    /// 向下滚动一行。
    pub(crate) fn scroll_down(&mut self, visible_lines: usize) {
        self.scroll_down_by(1, visible_lines);
        // 检查是否滚回底部
        self.check_scrolled_to_bottom(visible_lines);
    }

    /// 向上滚动一行。
    pub(crate) fn scroll_up(&mut self) {
        self.scroll_up_by(1);
        self.user_scrolled = true;
    }

    /// 向下滚动多行。
    pub(crate) fn scroll_down_by(&mut self, amount: usize, visible_lines: usize) {
        let max = self.total_lines().saturating_sub(visible_lines);
        self.scroll_offset = (self.scroll_offset + amount).min(max);
    }

    /// 向上滚动多行。
    pub(crate) fn scroll_up_by(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// 滚动到顶部。
    pub(crate) fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// 滚动到最新内容。
    pub(crate) fn scroll_to_bottom(&mut self, visible_lines: usize) {
        let max = self.total_lines().saturating_sub(visible_lines);
        self.scroll_offset = max;
        self.user_scrolled = false;
        self.unread_count = 0;
    }

    /// 检查是否已滚回底部，如果是则恢复自动跟随。
    fn check_scrolled_to_bottom(&mut self, visible_lines: usize) {
        let max = self.total_lines().saturating_sub(visible_lines);
        if self.scroll_offset >= max {
            self.user_scrolled = false;
            self.unread_count = 0;
        }
    }

    /// 流式内容追加时的自动滚动策略。
    /// 返回 true 表示应滚动到底部，false 表示保持当前位置。
    pub(crate) fn should_auto_scroll(&mut self, visible_lines: usize) -> bool {
        if self.user_scrolled {
            self.unread_count += 1;
            return false;
        }
        self.scroll_to_bottom(visible_lines);
        true
    }

    /// 用户是否处于手动滚动状态（暂停自动跟随）。
    pub(crate) fn is_user_scrolled(&self) -> bool {
        self.user_scrolled
    }

    /// 未读新消息数量（仅当 user_scrolled 为 true 时有意义）。
    pub(crate) fn unread_count(&self) -> usize {
        self.unread_count
    }

    /// 生成未读新消息提示文本。
    pub(crate) fn unread_hint(&self) -> Option<String> {
        if self.user_scrolled && self.unread_count > 0 {
            Some(format!("↓ {} 条新消息", self.unread_count))
        } else {
            None
        }
    }

    /// 估算总行数（简单按换行符计数）。
    fn total_lines(&self) -> usize {
        self.entries
            .iter()
            .map(|e| {
                let line_count = e.text.lines().count().max(1);
                // +1 给角色标签行
                line_count + 1
            })
            .sum()
    }

    /// 渲染为 ANSI 字符串。
    pub(crate) fn render(&self, area: Rect) -> String {
        self.render_styled(area, false)
    }

    /// 专利专屏等场景：纯文本，不套品牌色。
    pub(crate) fn render_plain(&self, area: Rect) -> String {
        self.render_styled(area, true)
    }

    fn render_styled(&self, area: Rect, plain: bool) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let mut lines: Vec<String> = Vec::new();
        let renderer = TerminalRenderer::new();

        for entry in &self.entries {
            let body = display_body(entry.role, &entry.text, plain, &renderer);
            let prefix = if plain {
                format!("{}: ", entry.role.label())
            } else {
                let label_color = match entry.role {
                    ChatRole::User => "\x1b[38;5;183m",
                    ChatRole::Assistant => "\x1b[38;5;213m",
                    ChatRole::System => "\x1b[38;5;246m",
                };
                format!("{label_color}{}\x1b[0m: ", entry.role.label())
            };
            let prefix_w = usize::from(visible_width(&prefix));
            let first = body.lines().next().unwrap_or("");
            let first_clipped = truncate_ansi_to_width(first, width.saturating_sub(prefix_w));
            lines.push(format!("{prefix}{first_clipped}"));
            for line in body.lines().skip(1) {
                let indent = "  ";
                let wrapped = wrap_ansi_to_width(line, width.saturating_sub(indent.len()));
                for wl in wrapped {
                    lines.push(format!("{indent}{wl}"));
                }
            }
        }

        // 应用滚动偏移
        let visible_height = area.height as usize;
        let start = self
            .scroll_offset
            .min(lines.len().saturating_sub(visible_height));
        let end = std::cmp::min(start + visible_height, lines.len());

        lines[start..end].join("\n")
    }

    /// 条目数量。
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 格式化为可读对话全文（用于 /panel 与视图 6）。
    pub(crate) fn transcript_text(&self) -> String {
        if self.entries.is_empty() {
            return "（尚无对话，请在底栏输入消息。）".to_string();
        }
        self.entries
            .iter()
            .map(|entry| format!("{}: {}", entry.role.label(), entry.text))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    #[must_use]
    pub(crate) fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// 清空所有条目。
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.scroll_offset = 0;
    }

    #[must_use]
    pub(crate) fn entries(&self) -> &[ChatEntry] {
        &self.entries
    }

    /// 导出对话纯文本（去 ANSI），供剪贴板使用。
    pub(crate) fn export_plain_conversation(&self) -> String {
        use crate::tui::clipboard::strip_ansi;

        self.entries
            .iter()
            .filter(|entry| !entry.text.is_empty())
            .map(|entry| format!("{}: {}", entry.role.label(), strip_ansi(&entry.text)))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl Component for ChatView {
    fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.state.visible {
            return;
        }
        use crate::tui::widgets::chat_view_ratatui::ChatViewWidget;
        use ratatui::prelude::Widget;
        ChatViewWidget {
            chat: self,
            thinking: false,
            spinner_frame: 0,
        }
        .render(area, buf);
    }

    fn handle_event(&mut self, _event: &Event) -> ActionResult {
        // ChatView 是只读显示，不直接处理输入事件
        ActionResult::Ignored
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    fn on_resize(&mut self, area: ratatui::layout::Rect) {
        self.state.bounds = area;
    }
}

/// 简易文本折行。
pub(crate) fn wrap_line(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }
    let mut result = Vec::new();
    let mut current = String::new();
    // 按字符宽度粗略计算（ASCII 字符宽度 1，CJK 宽度 2）
    let mut width: usize = 0;
    for ch in text.chars() {
        let cw = if ch.is_ascii() { 1 } else { 2 };
        if width + cw > max_width && !current.is_empty() {
            result.push(current.clone());
            current.clear();
            width = 0;
        }
        current.push(ch);
        width += cw;
    }
    if !current.is_empty() {
        result.push(current);
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_view_push_and_render() {
        let mut view = ChatView::new();
        view.push(ChatEntry {
            role: ChatRole::User,
            text: "你好".to_string(),
            reasoning: None,
        });
        view.push(ChatEntry {
            role: ChatRole::Assistant,
            text: "你好！有什么可以帮你的？".to_string(),
            reasoning: None,
        });
        assert_eq!(view.len(), 2);

        let rendered = view.render(Rect::new(0, 0, 40, 10));
        assert!(rendered.contains("你"));
        assert!(rendered.contains("云熙"));
    }

    #[test]
    fn chat_view_transcript_text() {
        let mut view = ChatView::new();
        view.push(ChatEntry {
            role: ChatRole::User,
            text: "你好".to_string(),
            reasoning: None,
        });
        view.push(ChatEntry {
            role: ChatRole::Assistant,
            text: "你好！".to_string(),
            reasoning: None,
        });
        let text = view.transcript_text();
        assert!(text.contains("你: 你好"));
        assert!(text.contains("云熙: 你好！"));
    }

    #[test]
    fn chat_view_scroll() {
        let mut view = ChatView::new();
        for i in 0..50 {
            view.push(ChatEntry {
                role: ChatRole::User,
                text: format!("消息 {i}"),
                reasoning: None,
            });
        }
        view.scroll_down(5);
        assert_eq!(view.scroll_offset, 1);
        view.scroll_up();
        assert_eq!(view.scroll_offset, 0);
        view.scroll_to_bottom(5);
        assert!(view.scroll_offset > 0);
    }

    #[test]
    fn chat_view_append_to_last() {
        let mut view = ChatView::new();
        view.push(ChatEntry {
            role: ChatRole::Assistant,
            text: "你好".to_string(),
            reasoning: None,
        });
        view.append_to_last("世界");
        assert_eq!(view.entries[0].text, "你好世界");
    }

    #[test]
    fn wrap_line_splits_long_text() {
        let wrapped = wrap_line("abcdefghij", 5);
        assert_eq!(wrapped, vec!["abcde", "fghij"]);
    }

    #[test]
    fn wrap_line_handles_empty() {
        let wrapped = wrap_line("", 10);
        assert_eq!(wrapped, vec![""]);
    }

    #[test]
    fn chat_view_renders_markdown_for_assistant() {
        let mut view = ChatView::new();
        view.push(ChatEntry {
            role: ChatRole::Assistant,
            text: "# Title\n\n**bold** text".to_string(),
            reasoning: None,
        });
        let rendered = view.render(Rect::new(0, 0, 80, 20));
        assert!(rendered.contains('\u{1b}'), "expected ANSI styling");
        assert!(
            !rendered.contains("**bold**"),
            "raw markdown should be rendered"
        );
        assert!(rendered.contains("bold"));
    }

    #[test]
    fn chat_view_wrap_preserves_ansi_on_banner_like_text() {
        let mut view = ChatView::new();
        let banner = format!(
            "{}\n{}",
            "\x1b[38;5;183m  │  \x1b[1m云\x1b[0m \x1b[1m熙\x1b[0m │\x1b[0m",
            "\x1b[2m模型 test\x1b[0m"
        );
        view.push(ChatEntry {
            role: ChatRole::Assistant,
            text: banner,
            reasoning: None,
        });
        let rendered = view.render(Rect::new(0, 0, 20, 10));
        assert!(
            !rendered.contains("[0m 熙"),
            "broken ansi leak: {rendered:?}"
        );
    }
}
