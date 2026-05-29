use std::io::{self, Stdout, Write};

use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub(crate) fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub(crate) fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<()> {
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}