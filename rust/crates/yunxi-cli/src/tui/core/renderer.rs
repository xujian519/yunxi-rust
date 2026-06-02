use crate::tui::state::global::GlobalState;
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Renderer {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    rerender_requested: std::sync::Arc<AtomicBool>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            terminal: None,
            rerender_requested: std::sync::Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        self.terminal = Some(terminal);
        Ok(())
    }

    pub fn render(&mut self, state: &GlobalState) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(terminal) = &mut self.terminal {
            terminal.draw(|frame| {
                Self::render_frame(frame, state);
            })?;
        }
        self.rerender_requested.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn render_frame(frame: &mut Frame, state: &GlobalState) {
        use ratatui::text::{Line, Span};
        use ratatui::widgets::Paragraph;

        let text = vec![
            Line::from("YunXi TUI - Opencode Redesign"),
            Line::from(""),
            Line::from(vec![
                Span::raw("Theme: "),
                Span::raw(&state.theme.current_theme),
            ]),
            Line::from(vec![
                Span::raw("Dark Mode: "),
                Span::raw(if state.theme.is_dark { "Yes" } else { "No" }),
            ]),
        ];

        let paragraph = Paragraph::new(text);
        frame.render_widget(paragraph, frame.area());
    }

    pub fn request_rerender(&self) {
        self.rerender_requested.store(true, Ordering::SeqCst);
    }

    pub fn is_rerender_requested(&self) -> bool {
        self.rerender_requested.load(Ordering::SeqCst)
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
