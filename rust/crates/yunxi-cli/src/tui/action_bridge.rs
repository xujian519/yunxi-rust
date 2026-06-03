//! 将新架构 `Action` 桥接到生产路径 `TuiApp`。

use crate::tui::app::{KeyEvent, TuiAction, TuiApp};
use crate::tui::components::base::Component;
use crate::tui::core::action::Action;
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crate::tui::hitl::open_human_guide;
use crossterm::event::{
    KeyCode, KeyEvent as CrosstermKey, KeyEventKind, KeyEventState, KeyModifiers,
};

/// 将 `TuiApp` 按键事件转为新架构 `Event`。
pub(crate) fn app_key_to_event(key: &KeyEvent) -> Event {
    let crossterm_key = match key {
        KeyEvent::Char(c) => CrosstermKey::new(KeyCode::Char(*c), KeyModifiers::empty()),
        KeyEvent::Enter => CrosstermKey::new(KeyCode::Enter, KeyModifiers::empty()),
        KeyEvent::Backspace => CrosstermKey::new(KeyCode::Backspace, KeyModifiers::empty()),
        KeyEvent::Delete => CrosstermKey::new(KeyCode::Delete, KeyModifiers::empty()),
        KeyEvent::Left => CrosstermKey::new(KeyCode::Left, KeyModifiers::empty()),
        KeyEvent::Right => CrosstermKey::new(KeyCode::Right, KeyModifiers::empty()),
        KeyEvent::Up => CrosstermKey::new(KeyCode::Up, KeyModifiers::empty()),
        KeyEvent::Down => CrosstermKey::new(KeyCode::Down, KeyModifiers::empty()),
        KeyEvent::Home => CrosstermKey::new(KeyCode::Home, KeyModifiers::empty()),
        KeyEvent::End => CrosstermKey::new(KeyCode::End, KeyModifiers::empty()),
        KeyEvent::Esc => CrosstermKey::new(KeyCode::Esc, KeyModifiers::empty()),
        KeyEvent::Tab => CrosstermKey::new(KeyCode::Tab, KeyModifiers::empty()),
        KeyEvent::ShiftEnter => CrosstermKey::new(KeyCode::Enter, KeyModifiers::SHIFT),
        KeyEvent::F(n) => CrosstermKey::new(KeyCode::F(*n), KeyModifiers::empty()),
        KeyEvent::Ctrl(c) => CrosstermKey::new(KeyCode::Char(*c), KeyModifiers::CONTROL),
        KeyEvent::CtrlShift(c) => CrosstermKey::new(
            KeyCode::Char(*c),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ),
    };
    let mut key_event = crossterm_key;
    key_event.kind = KeyEventKind::Press;
    key_event.state = KeyEventState::NONE;
    Event::Input(InputEvent::Key(key_event))
}

/// 命令面板打开时优先消费按键；返回 `Some(TuiAction)` 表示需 runner 继续处理。
pub(crate) fn handle_command_palette_key(app: &mut TuiApp, key: &KeyEvent) -> Option<TuiAction> {
    if !app.command_palette.is_visible() {
        return None;
    }

    let event = app_key_to_event(key);
    let result = Component::handle_event(&mut app.command_palette, &event);
    match result {
        ActionResult::Action(action) => dispatch_core_action(app, action),
        ActionResult::Handled | ActionResult::Ignored => None,
        ActionResult::Actions(actions) => {
            let mut pending = None;
            for action in actions {
                if let Some(next) = dispatch_core_action(app, action) {
                    pending = Some(next);
                }
            }
            pending
        }
    }
}

/// 将 `Action` 映射到 `TuiApp` 副作用，必要时返回 runner 级动作。
pub(crate) fn dispatch_core_action(app: &mut TuiApp, action: Action) -> Option<TuiAction> {
    match action {
        Action::Quit => {
            app.request_quit();
            None
        }
        Action::ShowCommandPalette => {
            app.open_command_palette();
            None
        }
        Action::HideCommandPalette => {
            app.close_command_palette();
            None
        }
        Action::ToggleSidebar => {
            app.toggle_tool_panel();
            None
        }
        Action::CopySelection => {
            app.copy_visible_text_to_clipboard();
            None
        }
        Action::SaveSession => Some(TuiAction::SaveSession),
        Action::NewSession => Some(TuiAction::NewSession),
        Action::Refresh => Some(TuiAction::RefreshStatus),
        Action::ExecuteCommand(cmd) => match cmd.as_str() {
            "help" => {
                app.open_help();
                None
            }
            "human_guide" => {
                open_human_guide(app);
                None
            }
            "interrupt" => Some(TuiAction::InterruptTurn),
            "sessions" => Some(TuiAction::OpenSessionPicker),
            _ => {
                app.push_system_message(&format!("\x1b[2m命令面板: 未接入的命令 '{cmd}'\x1b[0m"));
                None
            }
        },
        Action::ShowDialog(name) => match name.as_str() {
            "go_to_top" => {
                app.chat.scroll_to_top();
                None
            }
            "go_to_bottom" => {
                app.chat.scroll_to_bottom(10);
                None
            }
            "navigate_down" => {
                app.chat.scroll_down(10);
                None
            }
            "navigate_up" => {
                app.chat.scroll_up();
                None
            }
            _ => None,
        },
        Action::ToggleDarkMode => {
            let name = app.cycle_theme();
            app.push_system_message(&format!("\x1b[2m主题: {name}\x1b[0m"));
            None
        }
        Action::SwitchTheme(name) => {
            let applied = app.set_theme_by_name(&name);
            app.push_system_message(&format!("\x1b[2m主题: {applied}\x1b[0m"));
            None
        }
        Action::GoBack
        | Action::GoForward
        | Action::Navigate(_)
        | Action::Paste
        | Action::EditorCopy
        | Action::EditorPaste
        | Action::EditorCut
        | Action::EditorUndo
        | Action::EditorRedo
        | Action::SwitchTab(_)
        | Action::SwitchSession(_)
        | Action::DeleteSession(_)
        | Action::RenameSession(_, _)
        | Action::HideDialog
        | Action::Close
        | Action::ShowSubmenu(_, _)
        | Action::ShowParentMenu(_)
        | Action::Custom(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::core::action::Action;

    fn test_app() -> TuiApp {
        TuiApp::new("test-model".to_string(), "0.1.0".to_string())
    }

    #[test]
    fn ctrl_p_opens_command_palette() {
        let mut app = test_app();
        assert!(!app.command_palette.is_visible());
        let action = dispatch_core_action(&mut app, Action::ShowCommandPalette);
        assert!(action.is_none());
        assert!(app.command_palette.is_visible());
    }

    #[test]
    fn toggle_sidebar_toggles_tool_panel() {
        let mut app = test_app();
        assert!(app.show_tool_panel);
        dispatch_core_action(&mut app, Action::ToggleSidebar);
        assert!(!app.show_tool_panel);
    }

    #[test]
    fn command_palette_handles_esc() {
        let mut app = test_app();
        app.open_command_palette();
        let action = handle_command_palette_key(&mut app, &KeyEvent::Esc);
        assert!(action.is_none());
        assert!(!app.command_palette.is_visible());
    }
}
