use super::base::{generate_component_id, Component, ComponentState};
use crate::tui::core::action::ActionResult;
use crate::tui::core::event::{Event, InputEvent};
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::collections::HashSet;

pub struct Editor {
    state: ComponentState,
    content: String,
    lines: Vec<String>,
    cursor: Cursor,
    selection: Option<Selection>,
    syntax: Syntax,
    line_numbers: bool,
    folded_lines: HashSet<usize>,
    scroll_offset: (u16, u16),
    tab_size: usize,
    on_change: Option<Box<dyn Fn(String) -> ActionResult + Send + Sync>>,
    on_save: Option<Box<dyn Fn(String) -> ActionResult + Send + Sync>>,
    style: EditorStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub start: Cursor,
    pub end: Cursor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Syntax {
    Plain,
    Rust,
    Python,
    JavaScript,
    Json,
    Markdown,
}

#[derive(Debug, Clone)]
pub struct EditorStyle {
    pub bg: Color,
    pub fg: Color,
    pub line_number_bg: Color,
    pub line_number_fg: Color,
    pub cursor_bg: Color,
    pub cursor_fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub keyword_color: Color,
    pub string_color: Color,
    pub comment_color: Color,
    pub number_color: Color,
    pub function_color: Color,
    pub border: bool,
}

impl Default for EditorStyle {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(26, 35, 50),
            fg: Color::Rgb(232, 232, 237),
            line_number_bg: Color::Rgb(22, 22, 30),
            line_number_fg: Color::Rgb(106, 106, 128),
            cursor_bg: Color::Rgb(139, 176, 240),
            cursor_fg: Color::Rgb(13, 13, 18),
            selection_bg: Color::Rgb(68, 138, 255),
            selection_fg: Color::Rgb(13, 13, 18),
            keyword_color: Color::Rgb(207, 135, 255),
            string_color: Color::Rgb(209, 154, 102),
            comment_color: Color::Rgb(106, 153, 85),
            number_color: Color::Rgb(209, 154, 102),
            function_color: Color::Rgb(97, 175, 239),
            border: true,
        }
    }
}

impl Default for Syntax {
    fn default() -> Self {
        Syntax::Plain
    }
}

impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl Selection {
    pub fn new(start: Cursor, end: Cursor) -> Self {
        let (start, end) = if start.row < end.row || (start.row == end.row && start.col <= end.col)
        {
            (start, end)
        } else {
            (end, start)
        };
        Self { start, end }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl Editor {
    pub fn new(content: impl Into<String>) -> Self {
        let content = content.into();
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        Self {
            state: ComponentState::new(generate_component_id("editor")),
            content,
            lines,
            cursor: Cursor::new(0, 0),
            selection: None,
            syntax: Syntax::Plain,
            line_numbers: true,
            folded_lines: HashSet::new(),
            scroll_offset: (0, 0),
            tab_size: 4,
            on_change: None,
            on_save: None,
            style: EditorStyle::default(),
        }
    }

    pub fn with_syntax(mut self, syntax: Syntax) -> Self {
        self.syntax = syntax;
        self
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    pub fn with_tab_size(mut self, tab_size: usize) -> Self {
        self.tab_size = tab_size;
        self
    }

    pub fn with_style(mut self, style: EditorStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) -> ActionResult + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    pub fn with_on_save<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) -> ActionResult + Send + Sync + 'static,
    {
        self.on_save = Some(Box::new(callback));
        self
    }

    pub fn with_id(mut self, id: String) -> Self {
        self.state.id = id;
        self
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }

    pub fn get_cursor(&self) -> Cursor {
        self.cursor
    }

    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor.row >= self.lines.len() {
            self.lines.push(String::new());
        }

        let line = &mut self.lines[self.cursor.row];
        line.insert(self.cursor.col, c);
        self.cursor.col += 1;
        self.update_content();
    }

    pub fn insert_text(&mut self, text: &str) {
        for c in text.chars() {
            self.insert_char(c);
        }
    }

    pub fn delete_char(&mut self) {
        if self.cursor.row >= self.lines.len() {
            return;
        }

        let line = &mut self.lines[self.cursor.row];
        if self.cursor.col > 0 {
            line.remove(self.cursor.col - 1);
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            let prev_line_len = self.lines[self.cursor.row - 1].len();
            let current_line = self.lines.remove(self.cursor.row);
            self.lines[self.cursor.row - 1].push_str(&current_line);
            self.cursor.row -= 1;
            self.cursor.col = prev_line_len;
        }
        self.update_content();
    }

    pub fn delete_next_char(&mut self) {
        if self.cursor.row >= self.lines.len() {
            return;
        }

        let col = self.cursor.col;
        let row = self.cursor.row;
        let next_row = row + 1;

        if col < self.lines[row].len() {
            self.lines[row].remove(col);
        } else if next_row < self.lines.len() {
            let next_line = self.lines.remove(next_row);
            self.lines[row].push_str(&next_line);
        }
        self.update_content();
    }

    pub fn new_line(&mut self) {
        if self.cursor.row >= self.lines.len() {
            self.lines.push(String::new());
        } else {
            let line = &mut self.lines[self.cursor.row];
            let after_cursor = line.split_off(self.cursor.col);
            self.lines.insert(self.cursor.row + 1, after_cursor);
        }
        self.cursor.row += 1;
        self.cursor.col = 0;
        self.update_content();
    }

    pub fn insert_tab(&mut self) {
        let spaces = " ".repeat(self.tab_size);
        self.insert_text(&spaces);
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        let max_row = self.lines.len().saturating_sub(1);
        let max_col = self
            .lines
            .get(self.cursor.row)
            .map(|l| l.len())
            .unwrap_or(0);

        match direction {
            Direction::Up => {
                self.cursor.row = self.cursor.row.saturating_sub(1);
                self.cursor.col = self.cursor.col.min(
                    self.lines
                        .get(self.cursor.row)
                        .map(|l| l.len())
                        .unwrap_or(0),
                );
            }
            Direction::Down => {
                self.cursor.row = (self.cursor.row + 1).min(max_row);
                self.cursor.col = self.cursor.col.min(
                    self.lines
                        .get(self.cursor.row)
                        .map(|l| l.len())
                        .unwrap_or(0),
                );
            }
            Direction::Left => {
                self.cursor.col = self.cursor.col.saturating_sub(1);
            }
            Direction::Right => {
                self.cursor.col = (self.cursor.col + 1).min(max_col);
            }
            Direction::Home => {
                self.cursor.col = 0;
            }
            Direction::End => {
                self.cursor.col = max_col;
            }
            Direction::PageUp => {
                let page = 10;
                self.cursor.row = self.cursor.row.saturating_sub(page);
            }
            Direction::PageDown => {
                let page = 10;
                self.cursor.row = (self.cursor.row + page).min(max_row);
            }
            Direction::Top => {
                self.cursor.row = 0;
                self.cursor.col = 0;
            }
            Direction::Bottom => {
                self.cursor.row = max_row;
                self.cursor.col = self.lines.get(max_row).map(|l| l.len()).unwrap_or(0);
            }
        }
    }

    pub fn toggle_fold(&mut self, line_number: usize) {
        if self.folded_lines.contains(&line_number) {
            self.folded_lines.remove(&line_number);
        } else {
            self.folded_lines.insert(line_number);
        }
    }

    pub fn unfold_all(&mut self) {
        self.folded_lines.clear();
    }

    pub fn fold_all(&mut self) {
        self.folded_lines = (0..self.lines.len()).collect();
    }

    fn update_content(&mut self) {
        self.content = self.lines.join("\n");
        if let Some(ref callback) = self.on_change {
            callback(self.content.clone());
        }
    }

    fn highlight_syntax(&self, line: &str) -> Line {
        let mut spans = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match self.syntax {
                Syntax::Rust => self.highlight_rust(&chars, &mut i, &mut spans),
                Syntax::Python => self.highlight_python(&chars, &mut i, &mut spans),
                Syntax::JavaScript => self.highlight_javascript(&chars, &mut i, &mut spans),
                Syntax::Json => self.highlight_json(&chars, &mut i, &mut spans),
                Syntax::Markdown => self.highlight_markdown(&chars, &mut i, &mut spans),
                Syntax::Plain => {
                    spans.push(Span::raw(chars[i].to_string()));
                    i += 1;
                }
            }
        }

        Line::from(spans)
    }

    fn highlight_rust(&self, chars: &[char], i: &mut usize, spans: &mut Vec<Span>) {
        let keywords = [
            "fn", "let", "mut", "pub", "struct", "enum", "impl", "use", "mod", "trait",
        ];
        let rest: String = chars[*i..].iter().collect();

        if rest.starts_with("//") {
            *i += 2;
            while *i < chars.len() {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.comment_color),
                ));
                *i += 1;
            }
        } else if rest.starts_with('"') {
            spans.push(Span::styled(
                '"'.to_string(),
                Style::default().fg(self.style.string_color),
            ));
            *i += 1;
            while *i < chars.len() && chars[*i] != '"' {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
            if *i < chars.len() {
                spans.push(Span::styled(
                    '"'.to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
        } else if rest.starts_with(|c: char| c.is_ascii_digit()) {
            while *i < chars.len() && chars[*i].is_ascii_digit() {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.number_color),
                ));
                *i += 1;
            }
        } else if keywords.iter().any(|kw| {
            rest.starts_with(kw)
                && (*i + kw.len() == chars.len() || !chars[*i + kw.len()].is_alphanumeric())
        }) {
            for kw in keywords {
                if rest.starts_with(kw)
                    && (*i + kw.len() == chars.len() || !chars[*i + kw.len()].is_alphanumeric())
                {
                    for c in kw.chars() {
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().fg(self.style.keyword_color),
                        ));
                        *i += 1;
                    }
                    return;
                }
            }
        } else {
            spans.push(Span::raw(chars[*i].to_string()));
            *i += 1;
        }
    }

    fn highlight_python(&self, chars: &[char], i: &mut usize, spans: &mut Vec<Span>) {
        let keywords = [
            "def", "class", "if", "else", "for", "while", "import", "from", "return",
        ];
        let rest: String = chars[*i..].iter().collect();

        if rest.starts_with('#') {
            while *i < chars.len() {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.comment_color),
                ));
                *i += 1;
            }
        } else if rest.starts_with('"') || rest.starts_with('\'') {
            let quote = chars[*i];
            spans.push(Span::styled(
                quote.to_string(),
                Style::default().fg(self.style.string_color),
            ));
            *i += 1;
            while *i < chars.len() && chars[*i] != quote {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
            if *i < chars.len() {
                spans.push(Span::styled(
                    quote.to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
        } else {
            for kw in keywords {
                if rest.starts_with(kw)
                    && (*i + kw.len() == chars.len() || !chars[*i + kw.len()].is_alphanumeric())
                {
                    for c in kw.chars() {
                        spans.push(Span::styled(
                            c.to_string(),
                            Style::default().fg(self.style.keyword_color),
                        ));
                        *i += 1;
                    }
                    return;
                }
            }
            spans.push(Span::raw(chars[*i].to_string()));
            *i += 1;
        }
    }

    fn highlight_javascript(&self, chars: &[char], i: &mut usize, spans: &mut Vec<Span>) {
        self.highlight_rust(chars, i, spans);
    }

    fn highlight_json(&self, chars: &[char], i: &mut usize, spans: &mut Vec<Span>) {
        let rest: String = chars[*i..].iter().collect();

        if rest.starts_with('"') {
            spans.push(Span::styled(
                '"'.to_string(),
                Style::default().fg(self.style.string_color),
            ));
            *i += 1;
            while *i < chars.len() && chars[*i] != '"' {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
            if *i < chars.len() {
                spans.push(Span::styled(
                    '"'.to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
        } else if chars[*i].is_ascii_digit() {
            while *i < chars.len() && (chars[*i].is_ascii_digit() || chars[*i] == '.') {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.number_color),
                ));
                *i += 1;
            }
        } else if chars[*i] == 't' || chars[*i] == 'f' {
            let boolean = if chars[*i] == 't' { "true" } else { "false" };
            if rest.starts_with(boolean) {
                for c in boolean.chars() {
                    spans.push(Span::styled(
                        c.to_string(),
                        Style::default().fg(self.style.keyword_color),
                    ));
                    *i += 1;
                }
                return;
            }
        } else {
            spans.push(Span::raw(chars[*i].to_string()));
            *i += 1;
        }
    }

    fn highlight_markdown(&self, chars: &[char], i: &mut usize, spans: &mut Vec<Span>) {
        let rest: String = chars[*i..].iter().collect();

        if rest.starts_with('#') {
            while *i < chars.len() && chars[*i] == '#' {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default()
                        .fg(self.style.keyword_color)
                        .add_modifier(Modifier::BOLD),
                ));
                *i += 1;
            }
        } else if rest.starts_with("**") {
            spans.push(Span::styled(
                "**".to_string(),
                Style::default().fg(self.style.keyword_color),
            ));
            *i += 2;
        } else if rest.starts_with('`') {
            while *i < chars.len() && chars[*i] != '`' {
                spans.push(Span::styled(
                    chars[*i].to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
            if *i < chars.len() {
                spans.push(Span::styled(
                    '`'.to_string(),
                    Style::default().fg(self.style.string_color),
                ));
                *i += 1;
            }
        } else {
            spans.push(Span::raw(chars[*i].to_string()));
            *i += 1;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Top,
    Bottom,
}

impl Component for Editor {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.state.visible {
            return;
        }

        let max_line_number = self.lines.len();
        let line_number_width = if self.line_numbers {
            max_line_number.to_string().len() as u16 + 1
        } else {
            0
        };

        let lines: Vec<Line> = self
            .lines
            .iter()
            .enumerate()
            .map(|(idx, line)| {
                let highlighted = self.highlight_syntax(line);
                let line_number = if self.line_numbers {
                    let num = (idx + 1).to_string();
                    let style = if idx == self.cursor.row && self.state.focused {
                        Style::default()
                            .bg(self.style.line_number_bg)
                            .fg(self.style.cursor_fg)
                    } else {
                        Style::default()
                            .bg(self.style.line_number_bg)
                            .fg(self.style.line_number_fg)
                    };
                    Line::from(vec![
                        Span::styled(
                            format!("{:>width$}", num, width = line_number_width as usize - 1),
                            style,
                        ),
                        Span::raw(" "),
                    ])
                } else {
                    Line::default()
                };

                let mut all_spans = line_number.spans;
                all_spans.extend(highlighted.spans);
                Line::from(all_spans)
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(if self.style.border {
                        Borders::ALL
                    } else {
                        Borders::NONE
                    })
                    .style(Style::default()),
            )
            .scroll(self.scroll_offset);

        paragraph.render(area, buf);

        let cursor_area = Rect {
            x: area.x + line_number_width + self.cursor.col as u16,
            y: area.y + 1 + self.cursor.row as u16 - self.scroll_offset.1,
            width: 1,
            height: 1,
        };

        if cursor_area.x < area.right() && cursor_area.y < area.bottom() {
            if cursor_area.x < buf.area.width && cursor_area.y < buf.area.height {
                let cell = buf.get_mut(cursor_area.x, cursor_area.y);
                cell.set_style(
                    Style::default()
                        .bg(if self.state.focused {
                            self.style.cursor_bg
                        } else {
                            self.style.fg
                        })
                        .fg(if self.state.focused {
                            self.style.cursor_fg
                        } else {
                            self.style.bg
                        }),
                );
            }
        }
    }

    fn handle_event(&mut self, event: &Event) -> ActionResult {
        if self.state.disabled || !self.state.visible {
            return ActionResult::Ignored;
        }

        match event {
            Event::Input(InputEvent::Key(key)) => match key.code {
                KeyCode::Up => {
                    self.move_cursor(Direction::Up);
                    ActionResult::Handled
                }
                KeyCode::Down => {
                    self.move_cursor(Direction::Down);
                    ActionResult::Handled
                }
                KeyCode::Left => {
                    self.move_cursor(Direction::Left);
                    ActionResult::Handled
                }
                KeyCode::Right => {
                    self.move_cursor(Direction::Right);
                    ActionResult::Handled
                }
                KeyCode::Enter => {
                    self.new_line();
                    ActionResult::Handled
                }
                KeyCode::Backspace => {
                    self.delete_char();
                    ActionResult::Handled
                }
                KeyCode::Delete => {
                    self.delete_next_char();
                    ActionResult::Handled
                }
                KeyCode::Tab => {
                    self.insert_tab();
                    ActionResult::Handled
                }
                KeyCode::Home => {
                    self.move_cursor(Direction::Home);
                    ActionResult::Handled
                }
                KeyCode::End => {
                    self.move_cursor(Direction::End);
                    ActionResult::Handled
                }
                KeyCode::PageUp => {
                    self.move_cursor(Direction::PageUp);
                    ActionResult::Handled
                }
                KeyCode::PageDown => {
                    self.move_cursor(Direction::PageDown);
                    ActionResult::Handled
                }
                KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.move_cursor(Direction::Top);
                    ActionResult::Handled
                }
                KeyCode::Char('G') => {
                    self.move_cursor(Direction::Bottom);
                    ActionResult::Handled
                }
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(ref callback) = self.on_save {
                        return callback(self.content.clone());
                    }
                    ActionResult::Ignored
                }
                KeyCode::Char(c) => {
                    self.insert_char(c);
                    ActionResult::Handled
                }
                _ => ActionResult::Ignored,
            },
            Event::Input(InputEvent::Paste(text)) => {
                self.insert_text(text);
                ActionResult::Handled
            }
            _ => ActionResult::Ignored,
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.clone()
    }

    fn on_focus(&mut self, focused: bool) {
        self.state.focused = focused;
    }

    fn on_resize(&mut self, area: Rect) {
        self.state.bounds = area;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = Editor::new("Hello, World!".to_string());
        assert_eq!(editor.get_content(), "Hello, World!");
        assert_eq!(editor.get_lines().len(), 1);
    }

    #[test]
    fn test_editor_insert_char() {
        let mut editor = Editor::new("Hello".to_string());
        editor.move_cursor(Direction::End);
        editor.insert_char('!');
        assert_eq!(editor.get_content(), "Hello!");
    }

    #[test]
    fn test_editor_delete_char() {
        let mut editor = Editor::new("Hello".to_string());
        editor.move_cursor(Direction::End);
        editor.delete_char();
        assert_eq!(editor.get_content(), "Hell");
    }

    #[test]
    fn test_editor_new_line() {
        let mut editor = Editor::new("Hello".to_string());
        editor.move_cursor(Direction::End);
        editor.new_line();
        editor.insert_text("World");
        assert_eq!(editor.get_lines(), vec!["Hello", "World"]);
    }

    #[test]
    fn test_editor_move_cursor() {
        let mut editor = Editor::new("Hello, World!".to_string());
        editor.move_cursor(Direction::End);
        assert_eq!(editor.get_cursor().col, 13);
        editor.move_cursor(Direction::Home);
        assert_eq!(editor.get_cursor().col, 0);
    }

    #[test]
    fn test_editor_syntax_highlighting() {
        let editor =
            Editor::new("fn main() { println!(\"Hello\"); }".to_string()).with_syntax(Syntax::Rust);
        assert_eq!(editor.get_lines().len(), 1);
    }

    #[test]
    fn test_editor_fold() {
        let mut editor = Editor::new("Line 1\nLine 2\nLine 3".to_string());
        editor.toggle_fold(0);
        assert!(editor.folded_lines.contains(&0));
        editor.unfold_all();
        assert!(editor.folded_lines.is_empty());
    }
}
