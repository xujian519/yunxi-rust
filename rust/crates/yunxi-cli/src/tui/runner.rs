use std::io::{self, Write};

use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use crate::cli_action::AllowedToolSet;
use crate::permission_ui::CliPermissionPrompter;
use crate::runtime_bridge::{build_runtime, build_system_prompt};
use crate::session_mgr::create_managed_session_handle;
use crate::tui::app::{KeyEvent, TuiApp};
use crate::tui::components::tool_panel::ToolEntry;
use crate::VERSION;

use crate::runtime_bridge::CliToolExecutor;
use runtime::{ContentBlock, ConversationMessage, ConversationRuntime, PermissionMode, Session};

/// 启动 TUI 全屏 REPL。
pub(crate) fn run_tui_repl(
    model: String,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
) -> Result<(), Box<dyn std::error::Error>> {
    // 构建运行时（emit_output: false，TUI 模式下不直接打印到 stdout）
    let system_prompt = build_system_prompt()?;
    let session_handle = create_managed_session_handle()?;
    let runtime = build_runtime(
        Session::new(),
        model.clone(),
        system_prompt,
        true,
        false, // emit_output: false
        allowed_tools,
        permission_mode,
    )?;
    let session_id = session_handle.id.clone();

    // 包装 runtime 和 session 在可变状态中
    let mut state = TuiState {
        runtime,
        session_handle,
        permission_mode,
    };

    let mut app = TuiApp::new(model, VERSION.to_string());

    // 显示欢迎横幅作为首条系统消息
    let cwd = std::env::current_dir()
        .map_or_else(|_| "<unknown>".to_string(), |p| p.display().to_string());
    let banner = crate::tui::banner::render_banner(
        &app.model,
        state.permission_mode.as_str(),
        &cwd,
        &session_id,
    );
    app.push_assistant_message(&banner);

    // 进入 raw mode + alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // 主事件循环
    let result = run_event_loop(&mut app, &mut state, &mut stdout);

    // 清理终端
    execute!(stdout, LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;

    // 持久化会话
    state.persist_session()?;

    result
}

/// TUI 运行时状态（持有 runtime 和 session）。
struct TuiState {
    runtime: ConversationRuntime<llm::LlmClient, CliToolExecutor>,
    session_handle: crate::session_mgr::SessionHandle,
    permission_mode: PermissionMode,
}

impl TuiState {
    fn persist_session(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let session = self.runtime.session();
        session.save_to_path(&self.session_handle.path)?;
        Ok(())
    }
}

/// 主事件循环。
fn run_event_loop(
    app: &mut TuiApp,
    state: &mut TuiState,
    stdout: &mut io::Stdout,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // 渲染
        let (width, height) = crossterm::terminal::size()?;
        let rendered = app.render(width, height);
        execute!(stdout, Print(&rendered))?;
        stdout.flush()?;

        // 读取按键
        let event = event::read()?;
        let key = match event {
            Event::Key(key_event) => convert_key(key_event),
            Event::Resize(_, _)
            | Event::Mouse(_)
            | Event::FocusGained
            | Event::FocusLost
            | Event::Paste(_) => continue,
        };

        let action = app.handle_key(&key);

        if app.should_quit() {
            break;
        }

        if let Some(crate::tui::app::TuiAction::Submit(text)) = action {
            handle_submit(app, state, stdout, &text)?;
        }
    }

    Ok(())
}

/// 处理用户提交消息。
fn handle_submit(
    app: &mut TuiApp,
    state: &mut TuiState,
    stdout: &mut io::Stdout,
    input: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 显示思考状态
    app.set_thinking(true);
    let (width, height) = crossterm::terminal::size()?;
    let rendered = app.render(width, height);
    execute!(stdout, Print(&rendered))?;
    stdout.flush()?;

    // 暂时退出 raw mode 以便 runtime 正常工作
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;

    // 调用 runtime
    let mut permission_prompter = CliPermissionPrompter::new(state.permission_mode);
    let result = state
        .runtime
        .run_turn(input, Some(&mut permission_prompter));

    // 重新进入 raw mode
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    app.set_thinking(false);

    match result {
        Ok(summary) => {
            // 从 assistant_messages 提取文本
            for msg in &summary.assistant_messages {
                let text = extract_message_text(msg);
                if !text.is_empty() {
                    app.push_assistant_message(&text);
                }
                // 提取工具调用
                for block in &msg.blocks {
                    if let ContentBlock::ToolUse { name, .. } = block {
                        app.push_tool_entry(ToolEntry {
                            name: name.clone(),
                            detail: String::new(),
                            is_error: false,
                            collapsed: false,
                        });
                    }
                }
            }

            // 从 tool_results 提取
            for msg in &summary.tool_results {
                for block in &msg.blocks {
                    if let ContentBlock::ToolResult {
                        tool_name,
                        output,
                        is_error,
                        ..
                    } = block
                    {
                        app.push_tool_entry(ToolEntry {
                            name: tool_name.clone(),
                            detail: output.clone(),
                            is_error: *is_error,
                            collapsed: output.lines().count() > 10,
                        });
                    }
                }
            }
        }
        Err(error) => {
            app.push_assistant_message(&format!("\x1b[31m请求失败: {error}\x1b[0m"));
        }
    }

    Ok(())
}

/// 从 `ConversationMessage` 提取文本内容。
fn extract_message_text(msg: &ConversationMessage) -> String {
    msg.blocks
        .iter()
        .filter_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

/// 转换 crossterm 按键为 TUI `KeyEvent`。
fn convert_key(key: crossterm::event::KeyEvent) -> KeyEvent {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            return KeyEvent::Ctrl(c);
        }
    }

    match key.code {
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

#[cfg(test)]
mod tests {
    use super::*;
    use runtime::MessageRole;

    #[test]
    fn convert_key_maps_enter() {
        let ck = crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(convert_key(ck), KeyEvent::Enter);
    }

    #[test]
    fn convert_key_maps_ctrl_c() {
        let ck = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(convert_key(ck), KeyEvent::Ctrl('c'));
    }

    #[test]
    fn convert_key_maps_char() {
        let ck = crossterm::event::KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(convert_key(ck), KeyEvent::Char('a'));
    }

    #[test]
    fn extract_text_from_message() {
        let msg = ConversationMessage {
            role: MessageRole::Assistant,
            blocks: vec![
                ContentBlock::Text {
                    text: "hello ".to_string(),
                },
                ContentBlock::Text {
                    text: "world".to_string(),
                },
            ],
            usage: None,
        };
        assert_eq!(extract_message_text(&msg), "hello world");
    }

    #[test]
    fn extract_text_skips_tool_blocks() {
        let msg = ConversationMessage {
            role: MessageRole::Assistant,
            blocks: vec![
                ContentBlock::Text {
                    text: "result".to_string(),
                },
                ContentBlock::ToolUse {
                    id: "1".to_string(),
                    name: "bash".to_string(),
                    input: "{}".to_string(),
                },
            ],
            usage: None,
        };
        assert_eq!(extract_message_text(&msg), "result");
    }
}
