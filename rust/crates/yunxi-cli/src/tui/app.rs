#![allow(dead_code, clippy::struct_excessive_bools)]

use crate::tui::components::chat_view::{ChatEntry, ChatRole, ChatView};
use crate::tui::components::help_overlay::HelpOverlay;
use crate::tui::components::input_bar::InputBar;
use crate::tui::components::tool_panel::{ToolEntry, ToolPanel};
use crate::tui::layout::{Layout, Rect};

/// TUI 应用主状态。
pub(crate) struct TuiApp {
    /// 对话视图。
    chat: ChatView,
    /// 工具面板。
    tools: ToolPanel,
    /// 输入框。
    input: InputBar,
    /// 帮助覆盖层是否显示。
    show_help: bool,
    /// 工具面板是否显示。
    show_tool_panel: bool,
    /// 是否应该退出。
    should_quit: bool,
    /// 是否正在等待 AI 响应。
    thinking: bool,
    /// 当前模型名。
    pub(crate) model: String,
    /// 版本号。
    version: String,
}

impl TuiApp {
    pub(crate) fn new(model: String, version: String) -> Self {
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
        }
    }

    /// 处理按键事件，返回用户提交的输入（如有）。
    pub(crate) fn handle_key(&mut self, key: &KeyEvent) -> Option<TuiAction> {
        if self.show_help {
            self.show_help = false;
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
            KeyEvent::Enter if !self.input.is_empty() => {
                let text = self.input.take();
                self.chat.push(ChatEntry {
                    role: ChatRole::User,
                    text: text.clone(),
                });
                Some(TuiAction::Submit(text))
            }
            // 帮助
            KeyEvent::Ctrl('h') | KeyEvent::F(1) => {
                self.show_help = true;
                None
            }
            // 切换工具面板
            KeyEvent::F(2) => {
                self.show_tool_panel = !self.show_tool_panel;
                None
            }
            // 滚动
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
                None
            }
            KeyEvent::Backspace => {
                self.input.backspace();
                None
            }
            KeyEvent::Delete => {
                self.input.delete();
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
    }

    /// 追加助手回复的增量文本（流式）。
    pub(crate) fn append_assistant_text(&mut self, delta: &str) {
        self.chat.append_to_last(delta);
    }

    /// 追加工具调用记录。
    pub(crate) fn push_tool_entry(&mut self, entry: ToolEntry) {
        self.tools.push(entry);
    }

    /// 渲染完整界面到 ANSI 字符串。
    pub(crate) fn render(&self, width: u16, height: u16) -> String {
        let layout = if self.show_tool_panel {
            Layout::compute(width, height)
        } else {
            Layout::compute_no_panel(width, height)
        };

        let mut output = String::new();

        // 清屏
        output.push_str("\x1b[2J\x1b[H");

        // 标题栏
        output.push_str(&self.render_title_bar(&layout));
        output.push('\n');

        // 对话区域
        output.push_str(&self.chat.render(layout.chat_view));
        output.push('\n');

        // 工具面板（如果显示）
        if self.show_tool_panel && layout.tool_panel.is_valid() {
            output.push_str(&self.tools.render(layout.tool_panel));
        }

        // 输入框
        output.push('\n');
        output.push_str(&self.input.render(layout.input_bar));
        output.push('\n');

        // 状态栏
        output.push_str(&Self::render_status_bar(&layout));

        // 帮助覆盖层（如果显示）
        if self.show_help {
            let overlay_area = Rect::new(width / 4, height / 4, width / 2, height / 2);
            output.push('\n');
            output.push_str(&HelpOverlay::render(overlay_area));
        }

        output
    }

    /// 渲染标题栏。
    fn render_title_bar(&self, layout: &Layout) -> String {
        let thinking_indicator = if self.thinking {
            " \x1b[38;5;213m⠋ 思考中...\x1b[0m"
        } else {
            ""
        };
        let title = format!(
            " \x1b[1m\x1b[38;5;183m云熙智能体\x1b[0m \x1b[2mv{}\x1b[0m  \x1b[38;5;213m✿\x1b[0m  {}{thinking_indicator}",
            self.version, self.model
        );
        let width = layout.title_bar.width as usize;
        truncate_ansi_to_width(&title, width)
    }

    /// 渲染状态栏。
    fn render_status_bar(layout: &Layout) -> String {
        let width = layout.status_bar.width as usize;
        let shortcuts = "Ctrl+S:发送 Ctrl+C:中断 Ctrl+H:帮助 F2:面板";
        truncate_ansi_to_width(shortcuts, width)
    }

    /// 是否应该退出。
    pub(crate) fn should_quit(&self) -> bool {
        self.should_quit
    }
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
    Esc,
    F(u8),
}

/// TUI 动作（按键处理后返回的结果）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TuiAction {
    /// 用户提交了消息。
    Submit(String),
}

/// 粗略截断 ANSI 字符串到指定可见宽度。
fn truncate_ansi_to_width(s: &str, max_width: usize) -> String {
    let mut visible_width = 0;
    let mut result = String::new();
    let mut in_escape = false;

    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\x1b' {
            in_escape = true;
            result.push(c);
            i += 1;
            continue;
        }
        if in_escape {
            result.push(c);
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            i += 1;
            continue;
        }
        let cw = if c.is_ascii() { 1 } else { 2 };
        if visible_width + cw > max_width {
            break;
        }
        result.push(c);
        visible_width += cw;
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> TuiApp {
        TuiApp::new("claude-opus-4-6".to_string(), "0.1.0".to_string())
    }

    #[test]
    fn app_handles_enter_submit() {
        let mut app = test_app();
        app.input.insert('h');
        app.input.insert('i');
        let action = app.handle_key(&KeyEvent::Enter);
        assert_eq!(action, Some(TuiAction::Submit("hi".to_string())));
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
        assert!(rendered.contains("claude-opus-4-6"));
        assert!(rendered.contains("工具输出面板"));
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
}
