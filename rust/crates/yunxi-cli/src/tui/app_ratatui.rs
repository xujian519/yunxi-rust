use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Widget;
use ratatui::Terminal;

use crate::tui::app::TuiApp;
use crate::tui::widgets::chat_view_ratatui::ChatViewWidget;
use crate::tui::widgets::help_overlay_ratatui::HelpOverlay;
use crate::tui::widgets::input_bar_ratatui::InputBarWidget;
use crate::tui::widgets::status_bar_ratatui::StatusBarWidget;
use crate::tui::widgets::title_bar::TitleBar;
use crate::tui::widgets::tool_panel_ratatui::ToolPanelWidget;

impl TuiApp {
    pub(crate) fn render_ratatui(&self, terminal: &mut Terminal<impl ratatui::backend::Backend>) {
        let _ = terminal.draw(|frame| {
            let area = frame.area();

            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Min(0),
                    Constraint::Length(4),
                    Constraint::Length(1),
                ])
                .split(area);

            TitleBar::new(&self.model, &self.version, self.is_patent_mode())
                .render(vertical[0], frame.buffer_mut());

            let show_panel = self.show_tool_panel && !self.is_patent_mode();
            let main_horizontal = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(if show_panel {
                    vec![Constraint::Percentage(65), Constraint::Percentage(35)]
                } else {
                    vec![Constraint::Percentage(100)]
                })
                .split(vertical[1]);

            ChatViewWidget {
                chat: &self.chat,
                thinking: self.thinking,
                spinner_frame: self.spinner_frame,
            }
            .render(main_horizontal[0], frame.buffer_mut());

            if show_panel {
                ToolPanelWidget { tools: &self.tools }
                    .render(main_horizontal[1], frame.buffer_mut());
            }

            let content = self.input.content();
            InputBarWidget {
                content,
                slash_completion_count: self
                    .slash_completion
                    .as_ref()
                    .map(|s| s.matches.len())
                    .unwrap_or(0),
            }
            .render(vertical[2], frame.buffer_mut());

            StatusBarWidget {
                model: &self.model,
                permission_mode: "default",
                session_id: "",
                input_tokens: 0,
                output_tokens: self.turn_output_tokens,
                cost_usd: 0.0,
                active_tool: self.active_tool.as_deref(),
            }
            .render(vertical[3], frame.buffer_mut());

            if self.show_help {
                HelpOverlay.render(area, frame.buffer_mut());
            }
        });
    }
}