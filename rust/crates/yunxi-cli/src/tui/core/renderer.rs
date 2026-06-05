//! Ratatui-based renderer for the new-architecture App.

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::Widget;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use ratatui::Terminal;
use std::io;

use crate::tui::core::app::App;
use crate::tui::ui_palette;
use crate::tui::widgets::chat_view_ratatui::ChatViewWidget;
use crate::tui::widgets::command_palette_ratatui::CommandPaletteWidget;
use crate::tui::widgets::input_bar_ratatui::InputBarWidget;
use crate::tui::widgets::status_bar_ratatui::StatusBarWidget;
use crate::tui::widgets::title_bar::TitleBar;
use crate::tui::widgets::tool_panel_ratatui::ToolPanelWidget;

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
                Constraint::Length(input_rows.min(10).max(3)),
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
        frame.render_widget(
            StatusBarWidget {
                model: &app.model,
                permission_mode: app.permission_mode.as_str(),
                input_tokens: 0,
                output_tokens: 0,
                active_tool: app.active_tool.as_deref(),
            },
            vertical[3],
        );

        // Overlays
        Self::render_overlays(frame, area, app);
    }

    fn render_main(frame: &mut Frame, area: Rect, app: &App) {
        let show_panel = app.show_tool_panel;
        let show_sidebar = app.show_sidebar;

        let mut constraints = Vec::new();
        if show_sidebar {
            constraints.push(Constraint::Length(16));
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
            Self::render_sidebar(frame, horizontal[idx]);
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

    fn render_sidebar(frame: &mut Frame, area: Rect) {
        let c = ui_palette::active::bg_secondary();
        let bg = Color::Rgb(c.0, c.1, c.2);
        let tc = ui_palette::active::text_primary();
        let text = Color::Rgb(tc.0, tc.1, tc.2);
        let bc = ui_palette::active::border();
        let border_c = Color::Rgb(bc.0, bc.1, bc.2);

        let items = vec!["💬 Chat", "🔧 Tools", "⚙️ Settings"];
        let lines: Vec<Line> = items
            .iter()
            .map(|label| Line::from(Span::styled(*label, Style::default().fg(text))))
            .collect();
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_c))
            .title(" 导航 ")
            .style(Style::default().bg(bg));
        let paragraph = Paragraph::new(ratatui::text::Text::from(lines)).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_overlays(frame: &mut Frame, area: Rect, app: &App) {
        if app.command_palette.is_visible() {
            CommandPaletteWidget {
                palette: &app.command_palette,
            }
            .render(area, frame.buffer_mut());
            return;
        }

        if app.show_help {
            Self::render_help_overlay(frame, area);
            return;
        }

        if app.show_guide {
            Self::render_guide_overlay(frame, area);
        }
    }

    fn theme_bg_secondary() -> Color {
        let c = ui_palette::active::bg_secondary();
        Color::Rgb(c.0, c.1, c.2)
    }

    fn theme_border() -> Color {
        let c = ui_palette::active::border();
        Color::Rgb(c.0, c.1, c.2)
    }

    fn theme_text_primary() -> Color {
        let c = ui_palette::active::text_primary();
        Color::Rgb(c.0, c.1, c.2)
    }

    fn overlay_block(title: &str) -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Self::theme_border()))
            .title(format!(" {title} "))
            .style(Style::default().bg(Self::theme_bg_secondary()))
    }

    fn render_help_overlay(frame: &mut Frame, area: Rect) {
        let popup = Self::centered_rect(60, 20, area);
        frame.render_widget(Clear, popup);

        let help_lines = vec![
            Line::from(Span::styled(
                "快捷键帮助",
                Style::default().fg(Self::theme_text_primary()),
            )),
            Line::from(""),
            Line::from("  Ctrl+H / F1  打开帮助"),
            Line::from("  Ctrl+P / F3  命令面板"),
            Line::from("  Ctrl+B       侧边栏"),
            Line::from("  Ctrl+D       切换主题"),
            Line::from("  Ctrl+G       人机引导"),
            Line::from("  Ctrl+I       中断轮次"),
            Line::from("  Ctrl+Shift+C 复制"),
            Line::from("  Esc/Ctrl+C   退出/清空输入"),
            Line::from("  Tab          补全"),
            Line::from(""),
            Line::from("按任意键关闭"),
        ];

        let block = Self::overlay_block("帮助");
        let paragraph = Paragraph::new(ratatui::text::Text::from(help_lines)).block(block);
        frame.render_widget(paragraph, popup);
    }

    fn render_guide_overlay(frame: &mut Frame, area: Rect) {
        let popup = Self::centered_rect(60, 8, area);
        frame.render_widget(Clear, popup);

        let guide_text = vec![Line::from(
            "人机引导模式已开启 — 输入您的引导后按 Enter 发送",
        )];

        let block = Self::overlay_block("引导");
        let paragraph = Paragraph::new(ratatui::text::Text::from(guide_text)).block(block);
        frame.render_widget(paragraph, popup);
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
