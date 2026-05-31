use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

/// 是否启用鼠标捕获（滚轮/点击）。默认关闭以允许终端原生拖选复制。
fn mouse_capture_enabled() -> bool {
    match std::env::var("YUNXI_TUI_MOUSE").as_deref() {
        Ok("1" | "true" | "TRUE" | "yes" | "YES") => true,
        Ok("0" | "false" | "FALSE" | "no" | "NO") => false,
        _ => false,
    }
}

pub(crate) struct TuiTerminal {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    mouse_capture: bool,
}

impl TuiTerminal {
    pub(crate) fn setup() -> io::Result<Self> {
        enable_raw_mode()?;
        let mouse_capture = mouse_capture_enabled();
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        if mouse_capture {
            stdout.execute(EnableMouseCapture)?;
        }
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            mouse_capture,
        })
    }

    pub(crate) fn restore(mut self) -> io::Result<()> {
        if self.mouse_capture {
            self.terminal.backend_mut().execute(DisableMouseCapture)?;
        }
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}
