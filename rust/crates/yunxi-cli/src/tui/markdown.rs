use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

const HEADING_COLOR: Color = Color::Indexed(183);
const STRONG_COLOR: Color = Color::Indexed(214);
const CODE_BG: Color = Color::Indexed(236);
const LINK_COLOR: Color = Color::Indexed(39);
const QUOTE_COLOR: Color = Color::Indexed(245);
const TABLE_BORDER_COLOR: Color = Color::Indexed(240);

pub(crate) fn markdown_to_text(input: &str) -> Text<'static> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let mut expanded = String::with_capacity(input.len());
    let body = unwrap_markdown_fence(input, &mut expanded);

    let parser = Parser::new_ext(body, options);
    let events: Vec<_> = parser.collect();

    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut current_style = Style::default();

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    flush_line(&mut lines, &mut current_line);
                    current_style = Style::default()
                        .fg(HEADING_COLOR)
                        .add_modifier(Modifier::BOLD);
                    let prefix = format!("{} ", "#".repeat(*level as usize));
                    current_line.push(Span::styled(prefix, current_style));
                }
                Tag::Paragraph => {}
                Tag::BlockQuote(_) => {
                    flush_line(&mut lines, &mut current_line);
                    current_line.push(Span::styled("│ ", Style::default().fg(QUOTE_COLOR)));
                }
                Tag::CodeBlock(kind) => {
                    flush_line(&mut lines, &mut current_line);
                    let lang = match kind {
                        CodeBlockKind::Fenced(lang) if !lang.is_empty() => lang.as_ref(),
                        _ => "",
                    };
                    let code = collect_text(&events, i + 1);
                    lines.extend(code_block_to_lines(&code, lang));
                    i = skip_to_end(&events, i, &TagEnd::CodeBlock);
                }
                Tag::Table(_) => {
                    flush_line(&mut lines, &mut current_line);
                    let (header, rows) = collect_table(&events, i + 1);
                    lines.extend(render_table(header, rows));
                    i = skip_to_end(&events, i, &TagEnd::Table);
                }
                Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                    i += 1;
                    continue;
                }
                Tag::List(_) => {}
                Tag::Item => {
                    flush_line(&mut lines, &mut current_line);
                    current_line.push(Span::styled("  • ", Style::default().fg(DIM_COLOR)));
                }
                Tag::Emphasis => {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                Tag::Strong => {
                    current_style = current_style
                        .add_modifier(Modifier::BOLD)
                        .fg(STRONG_COLOR);
                }
                Tag::Strikethrough => {
                    current_style = current_style.add_modifier(Modifier::CROSSED_OUT);
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Heading(..) => {
                    flush_line(&mut lines, &mut current_line);
                    current_style = Style::default();
                }
                TagEnd::Paragraph | TagEnd::BlockQuote(_) => {
                    flush_line(&mut lines, &mut current_line);
                }
                TagEnd::Item => {
                    flush_line(&mut lines, &mut current_line);
                }
                TagEnd::Emphasis => {
                    current_style = current_style.remove_modifier(Modifier::ITALIC);
                }
                TagEnd::Strong => {
                    current_style = Style::default();
                }
                TagEnd::Strikethrough => {
                    current_style = current_style.remove_modifier(Modifier::CROSSED_OUT);
                }
                _ => {}
            },
            Event::Text(text) | Event::Code(text) => {
                let is_code = matches!(events[i], Event::Code(_));
                let style = if is_code {
                    Style::default()
                        .fg(Color::Indexed(214))
                        .bg(CODE_BG)
                } else {
                    current_style
                };
                let prefix = if is_code { "`" } else { "" };
                let suffix = if is_code { "`" } else { "" };
                current_line.push(Span::styled(
                    format!("{prefix}{text}{suffix}"),
                    style,
                ));
            }
            Event::InlineMath(_) => {}
            Event::InlineHtml(_) => {}
            Event::SoftBreak => {
                current_line.push(Span::styled(" ", Style::default()));
            }
            Event::HardBreak => {
                flush_line(&mut lines, &mut current_line);
            }
            Event::Rule => {
                flush_line(&mut lines, &mut current_line);
                let w = 40;
                lines.push(Line::from(Span::styled(
                    "─".repeat(w),
                    Style::default().fg(DIM_COLOR),
                )));
            }
            Event::TaskListMarker(checked) => {
                let mark = if *checked { "[x]" } else { "[ ]" };
                current_line.push(Span::styled(mark, Style::default().fg(DIM_COLOR)));
            }
            _ => {}
        }
        i += 1;
    }

    flush_line(&mut lines, &mut current_line);

    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    Text::from(lines)
}

const DIM_COLOR: Color = Color::Indexed(245);

fn flush_line(lines: &mut Vec<Line<'static>>, current: &mut Vec<Span<'static>>) {
    if !current.is_empty() || !lines.is_empty() {
        lines.push(Line::from(std::mem::take(current)));
    }
}

fn unwrap_markdown_fence<'a>(input: &'a str, buf: &'a mut String) -> &'a str {
    let trimmed = input.trim();
    if let Some(inner) = trimmed
        .strip_prefix("```markdown\n")
        .or_else(|| trimmed.strip_prefix("```md\n"))
        .or_else(|| trimmed.strip_prefix("```\n"))
    {
        if let Some(body) = inner.strip_suffix("\n```") {
            *buf = body.to_string();
            return buf.as_str();
        }
    }
    input
}

fn collect_text(events: &[Event], start: usize) -> String {
    let mut text = String::new();
    for event in events.iter().skip(start) {
        match event {
            Event::Text(t) | Event::Code(t) => text.push_str(t),
            Event::SoftBreak => text.push('\n'),
            Event::HardBreak => text.push('\n'),
            Event::End(_) => break,
            _ => {}
        }
    }
    text
}

fn skip_to_end(events: &[Event], start: usize, end_tag: &TagEnd) -> usize {
    let mut depth = 1;
    for (i, event) in events.iter().enumerate().skip(start) {
        match event {
            Event::Start(_) => depth += 1,
            Event::End(t) if *t == *end_tag => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    start
}

fn code_block_to_lines(code: &str, lang: &str) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let top = format!("┌─ {} ─────────────", if lang.is_empty() { "code" } else { lang });
    lines.push(Line::from(Span::styled(top, Style::default().fg(Color::Indexed(240)))));
    for line in code.lines() {
        lines.push(Line::from(Span::styled(
            format!("│ {line}"),
            Style::default().fg(Color::Indexed(250)),
        )));
    }
    lines.push(Line::from(Span::styled(
        "└──────────────────",
        Style::default().fg(Color::Indexed(240)),
    )));
    lines
}

fn collect_table(
    events: &[Event],
    start: usize,
) -> (Vec<String>, Vec<Vec<String>>) {
    let mut header = Vec::new();
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut in_head = true;

    let mut i = start;
    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::TableHead) => {
                in_head = true;
            }
            Event::End(TagEnd::TableHead) => {
                in_head = false;
            }
            Event::Start(Tag::TableRow) => {
                current_row.clear();
            }
            Event::Start(Tag::TableCell) => {}
            Event::Text(text) => {
                current_row.push(text.to_string());
            }
            Event::End(TagEnd::TableCell) => {}
            Event::End(TagEnd::TableRow) => {
                if !current_row.is_empty() {
                    let row = std::mem::take(&mut current_row);
                    if in_head {
                        header = row;
                    } else {
                        rows.push(row);
                    }
                }
            }
            Event::End(TagEnd::Table) => break,
            _ => {}
        }
        i += 1;
    }

    (header, rows)
}

fn render_table(header: Vec<String>, rows: Vec<Vec<String>>) -> Vec<Line<'static>> {
    let col_count = header.len().max(
        rows.first().map_or(0, |r| r.len()),
    );

    let mut col_widths = vec![0usize; col_count];
    for (i, cell) in header.iter().enumerate() {
        col_widths[i] = col_widths[i].max(cell.chars().count());
    }
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.chars().count());
            }
        }
    }

    let mut lines = Vec::new();

    if !header.is_empty() {
        lines.push(table_row(&header, &col_widths, true));
        let sep: Vec<String> = col_widths
            .iter()
            .map(|w| "─".repeat(*w))
            .collect();
        lines.push(table_separator(&sep, &col_widths));
    }

    for row in &rows {
        lines.push(table_row(row, &col_widths, false));
    }

    lines
}

fn table_row(cells: &[String], widths: &[usize], is_header: bool) -> Line<'static> {
    let mut spans = Vec::new();
    let border_style = Style::default().fg(TABLE_BORDER_COLOR);
    spans.push(Span::styled("│ ", border_style));

    for (i, cell) in cells.iter().enumerate() {
        let w = widths.get(i).copied().unwrap_or(0);
        let text = format!("{:<w$}", cell, w = w);
        if is_header {
            spans.push(Span::styled(text, Style::default().fg(HEADING_COLOR).add_modifier(Modifier::BOLD)));
        } else {
            spans.push(Span::styled(text, Style::default()));
        }
        if i < cells.len() - 1 {
            spans.push(Span::styled(" │ ", border_style));
        }
    }
    spans.push(Span::styled(" │", border_style));
    Line::from(spans)
}

fn table_separator(cells: &[String], widths: &[usize]) -> Line<'static> {
    let mut spans = Vec::new();
    let style = Style::default().fg(TABLE_BORDER_COLOR);
    spans.push(Span::styled("├─", style));

    for (i, cell) in cells.iter().enumerate() {
        let w = widths.get(i).copied().unwrap_or(0);
        spans.push(Span::styled(format!("{:<w$}", cell, w = w), style));
        if i < cells.len() - 1 {
            spans.push(Span::styled("─┼─", style));
        }
    }
    spans.push(Span::styled("─┤", style));
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading() {
        let text = markdown_to_text("# Hello");
        let line = text.lines.first().unwrap();
        assert!(line.to_string().contains("Hello"));
    }

    #[test]
    fn bold_text() {
        let text = markdown_to_text("**bold** text");
        let line = text.lines.first().unwrap();
        assert!(line.to_string().contains("bold"));
    }

    #[test]
    fn code_block() {
        let md = "```\nfn main() {}\n```";
        let text = markdown_to_text(md);
        assert!(text.to_string().contains("fn main()"));
    }

    #[test]
    fn table_rendering() {
        let md = "| A | B |\n| --- | --- |\n| 1 | 2 |";
        let text = markdown_to_text(md);
        assert!(text.to_string().contains("A"));
        assert!(text.to_string().contains("1"));
    }

    #[test]
    fn unwrap_fence() {
        let md = "```markdown\n# Title\n```";
        let mut buf = String::new();
        let result = unwrap_markdown_fence(md, &mut buf);
        assert_eq!(result.trim(), "# Title");
    }
}
