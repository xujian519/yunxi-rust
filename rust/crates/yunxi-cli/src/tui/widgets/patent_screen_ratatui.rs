use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget};

use crate::tui::patent::workspace::{PatentNav, PatentWorkspace};

pub(crate) struct PatentScreenWidget<'a> {
    pub(crate) patent: &'a PatentWorkspace,
}

impl Widget for PatentScreenWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // 导航
                Constraint::Percentage(48), // 主区
                Constraint::Percentage(32), // 证据
            ])
            .split(area);

        self.render_nav(horizontal[0], buf);
        self.render_main(horizontal[1], buf);
        self.render_evidence(horizontal[2], buf);
    }
}

impl PatentScreenWidget<'_> {
    fn render_nav(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let items: Vec<ListItem> = PatentNav::ALL
            .iter()
            .map(|nav| {
                let label = nav.label();
                let shortcut = nav.shortcut_index();
                if *nav == self.patent.nav {
                    ListItem::new(Span::styled(
                        format!(" {}. {}", shortcut, label),
                        Style::default()
                            .fg(Color::Indexed(183))
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    ListItem::new(Span::styled(
                        format!(" {}. {}", shortcut, label),
                        Style::default().fg(Color::Indexed(252)),
                    ))
                }
            })
            .collect();

        let block = Block::default()
            .title(" 导航 ")
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(Color::Indexed(245)));

        List::new(items).block(block).render(area, buf);
    }

    fn render_main(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let content = self.patent.main_content_text();

        Paragraph::new(Line::from(Span::styled(
            content,
            Style::default().fg(Color::Indexed(252)),
        )))
        .block(Block::default().borders(Borders::NONE))
        .scroll((self.patent.main_scroll as u16, 0))
        .render(area, buf);
    }

    fn render_evidence(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if self.patent.evidence_collapsed {
            return;
        }

        let block = Block::default()
            .title(" 证据 (F2) ")
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(Color::Indexed(245)));

        let content = if !self.patent.prior_art_lines.is_empty() {
            self.patent.prior_art_lines.join("\n")
        } else if !self.patent.search_hits.is_empty() {
            self.patent.search_hits.join("\n")
        } else {
            "尚无证据内容。\n在对话中使用检索、对比工具后，\n相关证据将自动显示于此面板。".to_string()
        };

        Paragraph::new(Line::from(Span::styled(
            content,
            Style::default().fg(Color::Indexed(252)),
        )))
        .block(block)
        .render(area, buf);
    }
}