//! Ratatui-based renderer for the new-architecture App.

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Clear;
use ratatui::Frame;
use ratatui::Terminal;
use std::io;

use crate::tui::core::app::App;
use crate::tui::layout::breakpoint::Viewport;
use crate::tui::widgets::chat_view_ratatui::ChatViewWidget;
use crate::tui::widgets::command_palette_ratatui::CommandPaletteWidget;
use crate::tui::widgets::flow_hitl_overlay_ratatui::FlowHitlOverlayWidget;
use crate::tui::widgets::guide_overlay_ratatui::GuideOverlayWidget;
use crate::tui::widgets::help_overlay_ratatui::HelpOverlay;
use crate::tui::widgets::input_bar_ratatui::InputBarWidget;
use crate::tui::widgets::permission_overlay_ratatui::PermissionOverlayWidget;
use crate::tui::widgets::session_picker_ratatui::SessionPickerWidget;
use crate::tui::widgets::sidebar_ratatui::SidebarWidget;
use crate::tui::widgets::status_bar_ratatui::StatusBarWidget;
use crate::tui::widgets::title_bar::TitleBar;
use crate::tui::widgets::toast_ratatui::ToastWidget;
use crate::tui::widgets::tool_panel_ratatui::ToolPanelWidget;
use ratatui::prelude::Widget;

pub(crate) struct Renderer {
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { terminal: None }
    }

    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        self.terminal = Some(terminal);
        Ok(())
    }

    pub fn render(&mut self, app: &App) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(terminal) = &mut self.terminal {
            terminal.draw(|frame| {
                Self::render_frame(frame, app);
            })?;
        }
        Ok(())
    }

    pub fn restore(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(terminal) = &mut self.terminal {
            crossterm::execute!(
                terminal.backend_mut(),
                crossterm::terminal::LeaveAlternateScreen
            )?;
        }
        crossterm::terminal::disable_raw_mode()?;
        Ok(())
    }

    fn render_frame(frame: &mut Frame, app: &App) {
        let area = frame.area();
        let input_rows = app.layout_input_rows() as u16;

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(input_rows.clamp(3, 10)),
                Constraint::Length(1),
            ])
            .split(area);

        // Title bar
        frame.render_widget(TitleBar::new(&app.model, &app.version), vertical[0]);

        // Main content area
        Self::render_main(frame, vertical[1], app);

        // Input bar
        frame.render_widget(
            InputBarWidget {
                content: app.input.content(),
                slash_completion_count: app
                    .slash_completion
                    .as_ref()
                    .map(|s| s.matches.len())
                    .unwrap_or(0),
                slash_completion: app.slash_completion.as_ref(),
            },
            vertical[2],
        );

        // Status bar
        let progress_msg = app
            .progress_manager
            .get(&crate::tui::progress::manager::ProgressId::named(
                "llm_turn",
            ))
            .and_then(|p| p.message.clone());
        frame.render_widget(
            StatusBarWidget {
                model: &app.model,
                permission_mode: app.permission_mode.as_str(),
                input_tokens: 0,
                output_tokens: 0,
                active_tool: app.active_tool.as_deref(),
                progress_message: progress_msg.as_deref(),
            },
            vertical[3],
        );

        // Overlays
        Self::render_overlays(frame, area, app);
    }

    fn render_main(frame: &mut Frame, area: Rect, app: &App) {
        let show_panel = app.show_tool_panel;
        let show_sidebar = app.show_sidebar && Viewport::from_size(area.width).sidebar_visible();

        let mut constraints = Vec::new();
        if show_sidebar {
            constraints.push(Constraint::Length(20));
        }
        constraints.push(Constraint::Percentage(if show_panel { 65 } else { 100 }));
        if show_panel {
            constraints.push(Constraint::Percentage(35));
        }

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let mut idx = 0;

        if show_sidebar {
            SidebarWidget {
                current_route: app.router.current_route(),
                focus_index: app.sidebar_focus,
            }
            .render(frame, horizontal[idx]);
            idx += 1;
        }

        // Chat view
        frame.render_widget(
            ChatViewWidget {
                chat: &app.chat,
                thinking: app.thinking,
                spinner_frame: app.spinner_frame,
            },
            horizontal[idx],
        );
        idx += 1;

        if show_panel {
            frame.render_widget(
                ToolPanelWidget {
                    tools: &app.tools,
                    focus_index: 0,
                },
                horizontal[idx],
            );
        }
    }

    fn render_overlays(frame: &mut Frame, area: Rect, app: &App) {
        if app.command_palette.is_visible() {
            CommandPaletteWidget {
                palette: &app.command_palette,
            }
            .render(area, frame.buffer_mut());
            return;
        }

        if let Some(ref picker) = app.session_picker {
            SessionPickerWidget { picker }.render(area, frame.buffer_mut());
            return;
        }

        if let Some(ref request) = app.pending_permission {
            let popup = Self::centered_rect(60, 10, area);
            frame.render_widget(Clear, popup);
            PermissionOverlayWidget { request }.render(popup, frame.buffer_mut());
            return;
        }

        if let Some(ref record) = app.pending_flow_hitl {
            let popup = Self::centered_rect(60, 10, area);
            frame.render_widget(Clear, popup);
            FlowHitlOverlayWidget { record }.render(popup, frame.buffer_mut());
            return;
        }

        if app.show_help {
            Self::render_help_overlay(frame, area);
            return;
        }

        if app.show_guide {
            Self::render_guide_overlay(frame, area, app);
        }

        // Toast layer — always on top (lowest overlay priority)
        if let Some(ref toast) = app.toast {
            ToastWidget { toast }.render(frame, area);
        }
    }

    fn render_help_overlay(frame: &mut Frame, area: Rect) {
        frame.render_widget(HelpOverlay, area);
    }

    fn render_guide_overlay(frame: &mut Frame, area: Rect, app: &App) {
        let popup = Self::centered_rect(60, 14, area);
        frame.render_widget(Clear, popup);
        frame.render_widget(
            GuideOverlayWidget {
                thinking: app.thinking,
            },
            popup,
        );
    }

    fn centered_rect(w: u16, h: u16, area: Rect) -> Rect {
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((area.height.saturating_sub(h)) / 2),
                Constraint::Length(h),
                Constraint::Length((area.height.saturating_sub(h)) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width.saturating_sub(w)) / 2),
                Constraint::Length(w),
                Constraint::Length((area.width.saturating_sub(w)) / 2),
            ])
            .split(v[1])[1]
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
