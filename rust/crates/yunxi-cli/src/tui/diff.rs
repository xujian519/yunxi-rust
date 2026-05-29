use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

const DARK_ADDED_BG: Color = Color::Rgb(33, 58, 43);
const DARK_REMOVED_BG: Color = Color::Rgb(74, 34, 29);
const LIGHT_ADDED_BG: Color = Color::Rgb(218, 251, 225);
const LIGHT_REMOVED_BG: Color = Color::Rgb(255, 235, 233);

pub(crate) fn render_unified_diff(diff_text: &str, is_dark: bool) -> Vec<Line<'static>> {
    let added_bg = if is_dark { DARK_ADDED_BG } else { LIGHT_ADDED_BG };
    let removed_bg = if is_dark { DARK_REMOVED_BG } else { LIGHT_REMOVED_BG };

    let mut lines = Vec::new();

    for line_text in diff_text.lines() {
        let line = if line_text.starts_with('+') {
            Line::from(Span::styled(
                line_text.to_string(),
                Style::default().bg(added_bg).fg(Color::Indexed(250)),
            ))
        } else if line_text.starts_with('-') {
            Line::from(Span::styled(
                line_text.to_string(),
                Style::default().bg(removed_bg).fg(Color::Indexed(250)),
            ))
        } else if line_text.starts_with("@@") {
            Line::from(Span::styled(
                line_text.to_string(),
                Style::default().fg(Color::Indexed(183)).add_modifier(ratatui::style::Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                line_text.to_string(),
                Style::default().fg(Color::Indexed(245)),
            ))
        };
        lines.push(line);
    }

    lines
}

pub(crate) fn render_add_remove_lines(
    added: &[String],
    removed: &[String],
    is_dark: bool,
) -> Vec<Line<'static>> {
    let added_bg = if is_dark { DARK_ADDED_BG } else { LIGHT_ADDED_BG };
    let removed_bg = if is_dark { DARK_REMOVED_BG } else { LIGHT_REMOVED_BG };

    let mut lines = Vec::new();

    for text in added {
        lines.push(Line::from(Span::styled(
            format!("+ {text}"),
            Style::default().bg(added_bg).fg(Color::Indexed(250)),
        )));
    }

    for text in removed {
        lines.push(Line::from(Span::styled(
            format!("- {text}"),
            Style::default().bg(removed_bg).fg(Color::Indexed(250)),
        )));
    }

    if added.is_empty() && removed.is_empty() {
        lines.push(Line::from(Span::styled(
            "（无差异）",
            Style::default().fg(Color::Indexed(245)),
        )));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_diff_colors_additions() {
        let diff = "+added line\n normal line\n-removed line";
        let lines = render_unified_diff(diff, true);
        assert_eq!(lines.len(), 3);

        let added = &lines[0];
        let style = added.spans[0].style;
        assert_eq!(style.bg, Some(DARK_ADDED_BG));

        let removed = &lines[2];
        let style = removed.spans[0].style;
        assert_eq!(style.bg, Some(DARK_REMOVED_BG));
    }

    #[test]
    fn add_remove_empty_shows_placeholder() {
        let lines = render_add_remove_lines(&[], &[], true);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].to_string().contains("无差异"));
    }

    #[test]
    fn add_remove_renders_content() {
        let added = vec!["新增的特征".to_string()];
        let removed = vec!["删除的特征".to_string()];
        let lines = render_add_remove_lines(&added, &removed, false);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].to_string().contains("新增的特征"));
        assert!(lines[1].to_string().contains("删除的特征"));
    }
}
