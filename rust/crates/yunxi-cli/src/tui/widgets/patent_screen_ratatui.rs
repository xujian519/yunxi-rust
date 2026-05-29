use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget};

use crate::tui::color::detect_background;
use crate::tui::diff;
use crate::tui::markdown;
use crate::tui::patent::workspace::{PatentNav, PatentWorkspace};

pub(crate) struct PatentScreenWidget<'a> {
    pub(crate) patent: &'a PatentWorkspace,
}

impl Widget for PatentScreenWidget<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(48),
                Constraint::Percentage(32),
            ])
            .split(area);

        self.render_nav(horizontal[0], buf);
        self.render_main(horizontal[1], buf);
        self.render_evidence(horizontal[2], buf);
    }
}

impl PatentScreenWidget<'_> {
    fn render_nav(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let bg = detect_background();
        let dim_color = if bg.is_dark() {
            Color::Indexed(245)
        } else {
            Color::Indexed(240)
        };

        let items: Vec<ListItem> = PatentNav::ALL
            .iter()
            .map(|nav| {
                let label = nav.label();
                let shortcut = nav.shortcut_index();
                let desc = nav.description();
                let text = format!(" {}. {}", shortcut, label);
                if *nav == self.patent.nav {
                    ListItem::new(vec![
                        Line::from(Span::styled(
                            text,
                            Style::default()
                                .fg(Color::Indexed(183))
                                .add_modifier(Modifier::BOLD),
                        )),
                        Line::from(Span::styled(
                            format!("    {}", desc),
                            Style::default().fg(dim_color),
                        )),
                    ])
                } else {
                    ListItem::new(Line::from(Span::styled(
                        text,
                        Style::default().fg(Color::Indexed(252)),
                    )))
                }
            })
            .collect();

        let mut header_lines = Vec::new();
        header_lines.push(Line::from(Span::styled(
            format!(" {}", self.patent.case.case_title),
            Style::default()
                .fg(Color::Indexed(183))
                .add_modifier(Modifier::BOLD),
        )));
        header_lines.push(Line::from(Span::styled(
            format!(" {}", self.patent.case.case_id),
            Style::default().fg(dim_color),
        )));

        let header = Paragraph::new(Text::from(header_lines))
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(dim_color)));

        let nav_area = Rect::new(area.x, area.y, area.width, 3);
        header.render(nav_area, buf);

        let list_area = Rect::new(
            area.x,
            area.y + nav_area.height + 1,
            area.width,
            area.height.saturating_sub(nav_area.height + 1),
        );
        let list = List::new(items)
            .block(Block::default().borders(Borders::RIGHT).border_style(Style::default().fg(dim_color)));
        list.render(list_area, buf);
    }

    fn render_main(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let content = self.patent.main_content_text();

        let rendered = match self.patent.nav {
            PatentNav::Claims if !self.patent.prior_art_lines.is_empty() => {
                let prior = self.patent.prior_art_lines.join("\n");
                self.render_claims_vs_prior_art(&content, &prior)
            }
            _ => markdown::markdown_to_text(&content),
        };

        Paragraph::new(rendered)
            .block(Block::default().borders(Borders::NONE))
            .scroll((self.patent.main_scroll as u16, 0))
            .render(area, buf);
    }

    fn render_claims_vs_prior_art(&self, claims: &str, prior: &str) -> Text<'static> {
        let mut lines = Vec::new();

        lines.push(Line::from(Span::styled(
            "┌─ 权利要求 ───────────────────────┬─ 对比文件 ───────────────────────┐",
            Style::default().fg(Color::Indexed(183)),
        )));

        let claim_lines: Vec<&str> = claims.lines().collect();
        let prior_lines: Vec<&str> = prior.lines().collect();
        let max_lines = claim_lines.len().max(prior_lines.len());

        for i in 0..max_lines {
            let left = claim_lines.get(i).copied().unwrap_or("");
            let right = prior_lines.get(i).copied().unwrap_or("");
            let left_padded = format!("│ {:<36}", left);
            let right_padded = format!("│ {:<36} │", right);

            let left_span = Span::styled(
                truncate_for_table(&left_padded, 38),
                Style::default().fg(Color::Indexed(252)),
            );
            let right_span = Span::styled(
                truncate_for_table(&right_padded, 39),
                Style::default().fg(Color::Indexed(245)),
            );

            lines.push(Line::from(vec![left_span, right_span]));
        }

        lines.push(Line::from(Span::styled(
            "└──────────────────────────────────┴──────────────────────────────────┘",
            Style::default().fg(Color::Indexed(183)),
        )));

        Text::from(lines)
    }

    fn render_evidence(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if self.patent.evidence_collapsed {
            return;
        }

        let bg = detect_background();
        let dim_color = if bg.is_dark() {
            Color::Indexed(245)
        } else {
            Color::Indexed(240)
        };

        let content = if !self.patent.prior_art_lines.is_empty() {
            self.render_evidence_prior_art()
        } else if !self.patent.search_hits.is_empty() {
            self.render_evidence_search()
        } else if !self.patent.office_action_excerpt.is_empty() && !self.patent.office_action_excerpt.starts_with('（') {
            self.render_evidence_oa()
        } else {
            Text::from(Line::from(Span::styled(
                "尚无证据内容。\n在对话中使用检索、对比工具后，\n相关证据将自动显示于此面板。",
                Style::default().fg(dim_color),
            )))
        };

        Paragraph::new(content)
            .block(
                Block::default()
                    .title(" 证据 (F2) ")
                    .borders(Borders::LEFT)
                    .border_style(Style::default().fg(dim_color)),
            )
            .render(area, buf);
    }

    fn render_evidence_prior_art(&self) -> Text<'static> {
        let bg = detect_background();
        let is_dark = bg.is_dark();

        let mut lines = Vec::new();
        let added = Vec::new();
        let removed = Vec::new();

        lines.extend(diff::render_add_remove_lines(&added, &removed, is_dark));

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "对比文件:",
            Style::default()
                .fg(Color::Indexed(183))
                .add_modifier(Modifier::BOLD),
        )));

        for art in &self.patent.prior_art_lines {
            lines.push(Line::from(Span::styled(
                format!("  • {art}"),
                Style::default().fg(Color::Indexed(252)),
            )));
        }

        Text::from(lines)
    }

    fn render_evidence_search(&self) -> Text<'static> {
        let mut lines = Vec::new();
        lines.push(Line::from(Span::styled(
            "检索命中:",
            Style::default()
                .fg(Color::Indexed(183))
                .add_modifier(Modifier::BOLD),
        )));
        for hit in &self.patent.search_hits {
            lines.push(Line::from(""));
            lines.extend(markdown::markdown_to_text(hit).lines);
        }
        Text::from(lines)
    }

    fn render_evidence_oa(&self) -> Text<'static> {
        let oa_text = &self.patent.office_action_excerpt;
        let annotated = annotate_oa(oa_text);
        markdown::markdown_to_text(&annotated)
    }
}

fn annotate_oa(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    result.push_str("**审查意见标注**\n\n");

    for line in text.lines() {
        if line.contains("缺乏") || line.contains("不具备") {
            result.push_str(&format!("- 🔴 {line}\n"));
        } else if line.contains("权利要求") || line.contains("对比文件") {
            result.push_str(&format!("- 📎 {line}\n"));
        } else if line.contains("法") && line.contains("条") {
            result.push_str(&format!("- ⚖️ {line}\n"));
        } else {
            result.push_str(&format!("{line}\n"));
        }
    }

    result
}

fn truncate_for_table(s: &str, max_width: usize) -> String {
    let mut width = 0usize;
    let mut result = String::new();
    for ch in s.chars() {
        let cw = if ch.is_ascii() { 1 } else { 2 };
        if width + cw > max_width {
            break;
        }
        result.push(ch);
        width += cw;
    }
    let padding = max_width.saturating_sub(width);
    result.push_str(&" ".repeat(padding));
    result
}

impl PatentNav {
    fn description(self) -> &'static str {
        match self {
            Self::Claims => "权利要求矩阵",
            Self::PriorArt => "对比文件分析",
            Self::OfficeAction => "审查意见原文",
            Self::Search => "检索命中列表",
            Self::Draft => "答复草稿汇总",
            Self::Assistant => "完整对话记录",
        }
    }
}
