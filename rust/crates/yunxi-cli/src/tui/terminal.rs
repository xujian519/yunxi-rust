use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub(crate) struct TuiTerminal {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TuiTerminal {
    pub(crate) fn setup() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub(crate) fn restore(mut self) -> io::Result<()> {
        self.terminal.backend_mut().execute(DisableMouseCapture)?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}
