//! TUI entry point using the new architecture (core::App + core::Renderer).

use crate::tui::core::app::{App, KeyEvent};
use crate::tui::core::renderer::Renderer;

/// Launch the TUI application.
pub(crate) fn run_tui_repl(
    model: String,
    _allowed_tools: Option<crate::cli_action::AllowedToolSet>,
    _permission_mode: runtime::PermissionMode,
    _resume_session: Option<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new(model)?;
    let mut renderer = Renderer::new();
    renderer.initialize()?;

    // Show welcome banner
    {
        let cwd = std::env::current_dir()
            .map_or_else(|_| "<unknown>".to_string(), |p| p.display().to_string());
        let banner = crate::tui::banner::render_banner(
            &app.model,
            app.permission_mode.as_str(),
            &cwd,
            &app.session_handle.id,
        );
        app.push_system_message(&banner);
        app.push_system_message(
            "\x1b[2mF3/Ctrl+P 命令 · Ctrl+B 面板 · Ctrl+D 主题 · Ctrl+G 引导 · Ctrl+I 中断\x1b[0m",
        );
    }
    renderer.render(&app)?;

    // Event loop
    loop {
        // Poll active turn
        let turn_events: Vec<crate::tui::turn::TurnEvent> = app
            .active_turn
            .as_mut()
            .map(|t| t.poll())
            .unwrap_or_default();
        let turn_finished = app
            .active_turn
            .as_ref()
            .map(|t| t.is_finished())
            .unwrap_or(false);

        for event in turn_events {
            app.needs_render = true;
            app.handle_turn_event_wrapped(event);
        }
        if turn_finished {
            let finished = app.active_turn.take().expect("turn");
            finished.join();
        }

        if app.should_quit() {
            break;
        }

        // Tick spinner when thinking
        if app.thinking {
            app.spinner_frame = app.spinner_frame.wrapping_add(1);
            app.needs_render = true;
        }

        // Render if needed
        if app.needs_render() {
            renderer.render(&app)?;
            app.clear_render_flag();
        }

        // Non-blocking input poll
        if !crossterm::event::poll(std::time::Duration::from_millis(50))? {
            continue;
        }

        match crossterm::event::read()? {
            crossterm::event::Event::Key(key_event) => {
                let key = convert_crossterm_key(&key_event);
                let action = app.handle_key(&key);
                if let Some(action) = action {
                    app.dispatch_action_wrapped(action)?;
                }
            }
            crossterm::event::Event::Mouse(mouse_event) => {
                app.handle_mouse_wrapped(mouse_event);
            }
            crossterm::event::Event::Resize(_, _) => {
                app.needs_render = true;
            }
            crossterm::event::Event::Paste(text) if !app.has_blocking_modal() => {
                app.input.set_content(text);
            }
            _ => {}
        }
    }

    renderer.restore()?;
    app.persist_session();
    Ok(())
}

/// Convert a crossterm KeyEvent to our internal KeyEvent.
fn convert_crossterm_key(key: &crossterm::event::KeyEvent) -> KeyEvent {
    super::core::app::convert_key(key)
}
