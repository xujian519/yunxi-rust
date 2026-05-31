use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

use crate::tui::app::TuiApp;

use crate::tui::widgets::chat_view_ratatui::ChatViewWidget;
use crate::tui::widgets::flow_hitl_overlay_ratatui::FlowHitlOverlayWidget;
use crate::tui::widgets::guide_overlay_ratatui::GuideOverlayWidget;
use crate::tui::widgets::help_overlay_ratatui::HelpOverlay;
use crate::tui::widgets::input_bar_ratatui::InputBarWidget;
use crate::tui::widgets::permission_overlay_ratatui::PermissionOverlayWidget;
use crate::tui::widgets::session_picker_ratatui::SessionPickerWidget;
use crate::tui::widgets::status_bar_ratatui::StatusBarWidget;
use crate::tui::widgets::title_bar::TitleBar;
use crate::tui::widgets::tool_panel_ratatui::ToolPanelWidget;

impl TuiApp {
    pub(crate) fn render_frame(&self, frame: &mut Frame) {
        let area = frame.area();

        if self.pager.is_some() {
            self.render_pager_overlay(frame, area);
            return;
        }

        let input_rows = self.layout_input_rows();

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(input_rows),
                Constraint::Length(1),
            ])
            .split(area);

        TitleBar::new(&self.model, &self.version).render(vertical[0], frame.buffer_mut());

        self.render_general_main(frame, vertical[1]);

        let content = self.input.content();
        InputBarWidget {
            content,
            slash_completion_count: self
                .slash_completion
                .as_ref()
                .map(|s| s.matches.len())
                .unwrap_or(0),
            slash_completion: self.slash_completion.as_ref(),
        }
        .render(vertical[2], frame.buffer_mut());

        StatusBarWidget {
            model: &self.status.model,
            permission_mode: &self.status.permission_mode,
            session_id: &self.status.session_id,
            input_tokens: self.status.cumulative_input_tokens as u32,
            output_tokens: self.turn_output_tokens,
            cost_usd: self.status.estimated_cost_usd,
            active_tool: self.active_tool.as_deref(),
        }
        .render(vertical[3], frame.buffer_mut());

        self.render_overlays(frame, area);
    }

    fn render_general_main(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let show_panel = self.show_tool_panel;
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(if show_panel {
                vec![Constraint::Percentage(65), Constraint::Percentage(35)]
            } else {
                vec![Constraint::Percentage(100)]
            })
            .split(area);

        ChatViewWidget {
            chat: &self.chat,
            thinking: self.thinking,
            spinner_frame: self.spinner_frame,
        }
        .render(horizontal[0], frame.buffer_mut());

        if show_panel {
            ToolPanelWidget { tools: &self.tools }.render(horizontal[1], frame.buffer_mut());
        }
    }

    fn render_overlays(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        if self.show_help {
            HelpOverlay.render(area, frame.buffer_mut());
            return;
        }

        if let Some(ref req) = self.pending_permission {
            let popup = centered_popup(area, 10);
            PermissionOverlayWidget { request: req }.render(popup, frame.buffer_mut());
            return;
        }

        if let Some(ref record) = self.pending_flow_hitl {
            let popup = centered_popup(area, 12);
            FlowHitlOverlayWidget { record }.render(popup, frame.buffer_mut());
            return;
        }

        if self.show_guide {
            let popup = centered_popup(area, 10);
            GuideOverlayWidget {
                thinking: self.thinking,
            }
            .render(popup, frame.buffer_mut());
        }

        if self.session_picker.is_some() {
            SessionPickerWidget {
                picker: self.session_picker.as_ref().unwrap(),
            }
            .render(area, frame.buffer_mut());
        }
    }

    fn render_pager_overlay(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let pager = self.pager.as_ref().expect("pager must exist");
        let end = (pager.scroll_offset() + area.height as usize - 6).min(pager.line_count());
        let visible: Vec<Line> = pager
            .lines()
            .iter()
            .skip(pager.scroll_offset())
            .take(end.saturating_sub(pager.scroll_offset()))
            .map(|l: &String| {
                Line::from(Span::styled(
                    l.as_str(),
                    Style::default().fg(Color::Indexed(
                        if crate::tui::ui_palette::terminal_light_background() {
                            235
                        } else {
                            252
                        },
                    )),
                ))
            })
            .collect();

        let popup_width = (area.width * 3 / 4).min(100);
        let popup_height = (area.height * 2 / 3).min(30);
        let popup = centered_rect(popup_width, popup_height, area);

        Clear.render(popup, frame.buffer_mut());
        Paragraph::new(Text::from(visible)).render(popup, frame.buffer_mut());
    }
}

fn centered_popup(area: ratatui::layout::Rect, height: u16) -> ratatui::layout::Rect {
    let popup_width = std::cmp::min(area.width, 60);
    centered_rect(popup_width, height, area)
}

fn centered_rect(w: u16, h: u16, area: ratatui::layout::Rect) -> ratatui::layout::Rect {
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
