use super::renderer::{RichText, TextSpan};

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownElement {
    Heading {
        level: u8,
        content: String,
    },
    Paragraph {
        spans: Vec<TextSpan>,
    },
    CodeBlock {
        language: Option<String>,
        content: String,
    },
    InlineCode {
        content: String,
    },
    Bold {
        content: String,
    },
    Italic {
        content: String,
    },
    Strikethrough {
        content: String,
    },
    Link {
        text: String,
        url: String,
    },
    UnorderedList {
        items: Vec<String>,
    },
    OrderedList {
        items: Vec<String>,
    },
    Blockquote {
        content: String,
    },
    HorizontalRule,
    Text {
        content: String,
    },
    LineBreak,
}

pub struct MarkdownParser {
    elements: Vec<MarkdownElement>,
    current_line: String,
    in_code_block: bool,
    code_block_language: Option<String>,
    code_block_content: Vec<String>,
    list_level: usize,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            current_line: String::new(),
            in_code_block: false,
            code_block_language: None,
            code_block_content: Vec::new(),
            list_level: 0,
        }
    }

    pub fn parse(&mut self, markdown: &str) -> Vec<MarkdownElement> {
        self.elements.clear();
        self.current_line.clear();
        self.in_code_block = false;
        self.code_block_language = None;
        self.code_block_content.clear();
        self.list_level = 0;

        for line in markdown.lines() {
            self.parse_line(line);
        }

        self.flush_current_line();
        self.elements.clone()
    }

    pub fn parse_to_rich_text(&mut self, markdown: &str) -> RichText {
        let elements = self.parse(markdown);
        self.elements_to_rich_text(&elements)
    }

    fn parse_line(&mut self, line: &str) {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            self.handle_code_block_fence(trimmed);
            return;
        }

        if self.in_code_block {
            self.code_block_content.push(line.to_string());
            return;
        }

        if trimmed.starts_with("#") {
            self.flush_current_line();
            self.parse_heading(trimmed);
        } else if trimmed.starts_with(">") {
            self.flush_current_line();
            self.parse_blockquote(trimmed);
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            self.flush_current_line();
            self.parse_unordered_list(trimmed);
        } else if trimmed.starts_with(&format!("{}. ", 1))
            || trimmed.starts_with(&format!("{}. ", self.list_level + 1))
        {
            self.flush_current_line();
            self.parse_ordered_list(trimmed);
        } else if trimmed == "---" || trimmed == "***" {
            self.flush_current_line();
            self.elements.push(MarkdownElement::HorizontalRule);
        } else if trimmed.is_empty() {
            self.flush_current_line();
        } else {
            if self.current_line.is_empty() {
                self.current_line = line.to_string();
            } else {
                self.current_line.push(' ');
                self.current_line.push_str(line);
            }
        }
    }

    fn handle_code_block_fence(&mut self, line: &str) {
        if self.in_code_block {
            self.in_code_block = false;
            let content = self.code_block_content.join("\n");
            self.elements.push(MarkdownElement::CodeBlock {
                language: self.code_block_language.take(),
                content,
            });
            self.code_block_content.clear();
        } else {
            self.in_code_block = true;
            self.code_block_language = Some(line[3..].trim().to_string());
            if self
                .code_block_language
                .as_ref()
                .map_or(false, |s| s.is_empty())
            {
                self.code_block_language = None;
            }
        }
    }

    fn parse_heading(&mut self, line: &str) {
        let mut level = 0;
        for c in line.chars() {
            if c == '#' {
                level += 1;
            } else {
                break;
            }
        }

        if level > 0 && level <= 6 {
            let content = line[level..].trim().to_string();
            self.elements
                .push(MarkdownElement::Heading { level: level as u8, content });
        } else {
            self.elements.push(MarkdownElement::Text {
                content: line.to_string(),
            });
        }
    }

    fn parse_blockquote(&mut self, line: &str) {
        let content = line[1..].trim().to_string();
        self.elements.push(MarkdownElement::Blockquote { content });
    }

    fn parse_unordered_list(&mut self, line: &str) {
        let content = line[2..].trim().to_string();
        self.elements.push(MarkdownElement::UnorderedList {
            items: vec![content],
        });
    }

    fn parse_ordered_list(&mut self, line: &str) {
        if let Some(pos) = line.find(". ") {
            let content = line[pos + 2..].trim().to_string();
            self.elements.push(MarkdownElement::OrderedList {
                items: vec![content],
            });
        }
    }

    fn flush_current_line(&mut self) {
        if !self.current_line.is_empty() {
            let spans = self.parse_inline_elements(&self.current_line);
            self.elements.push(MarkdownElement::Paragraph { spans });
            self.current_line.clear();
        }
    }

    fn parse_inline_elements(&self, text: &str) -> Vec<TextSpan> {
        let mut spans = Vec::new();
        let mut current = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '*' => {
                    if !current.is_empty() {
                        spans.push(TextSpan::text(current.clone()));
                        current.clear();
                    }
                    if let Some(&next) = chars.peek() {
                        if next == '*' {
                            chars.next();
                            let bold_content = self.extract_until(&mut chars, "**");
                            spans.push(TextSpan::text(bold_content).bold());
                        } else {
                            let italic_content = self.extract_until(&mut chars, "*");
                            spans.push(TextSpan::text(italic_content).italic());
                        }
                    } else {
                        current.push(c);
                    }
                }
                '_' => {
                    if !current.is_empty() {
                        spans.push(TextSpan::text(current.clone()));
                        current.clear();
                    }
                    let italic_content = self.extract_until(&mut chars, "_");
                    spans.push(TextSpan::text(italic_content).italic());
                }
                '`' => {
                    if !current.is_empty() {
                        spans.push(TextSpan::text(current.clone()));
                        current.clear();
                    }
                    if let Some(&next) = chars.peek() {
                        if next == '`' {
                            chars.next();
                            let code_content = self.extract_until(&mut chars, "``");
                            spans.push(TextSpan::code_block(code_content));
                        } else {
                            let code_content = self.extract_until(&mut chars, "`");
                            spans.push(TextSpan::code(code_content));
                        }
                    }
                }
                '[' => {
                    if !current.is_empty() {
                        spans.push(TextSpan::text(current.clone()));
                        current.clear();
                    }
                    let link_text = self.extract_until(&mut chars, "]");
                    if chars.peek() == Some(&'(') {
                        chars.next();
                        let url = self.extract_until(&mut chars, ")");
                        spans.push(TextSpan::link(link_text, url));
                    } else {
                        spans.push(TextSpan::text(format!("[{}]", link_text)));
                    }
                }
                '~' => {
                    if let Some(&next) = chars.peek() {
                        if next == '~' {
                            if !current.is_empty() {
                                spans.push(TextSpan::text(current.clone()));
                                current.clear();
                            }
                            chars.next();
                            let strikethrough_content = self.extract_until(&mut chars, "~~");
                            spans.push(TextSpan::text(strikethrough_content).dim());
                        } else {
                            current.push(c);
                        }
                    } else {
                        current.push(c);
                    }
                }
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        current.push(escaped);
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            spans.push(TextSpan::text(current));
        }

        spans
    }

    fn extract_until<'a>(
        &self,
        chars: &mut std::iter::Peekable<std::str::Chars<'a>>,
        delimiter: &str,
    ) -> String {
        let mut result = String::new();
        let delimiter_chars: Vec<char> = delimiter.chars().collect();
        let mut match_pos = 0;

        while let Some(c) = chars.next() {
            if c == delimiter_chars[match_pos] {
                match_pos += 1;
                if match_pos == delimiter_chars.len() {
                    return result;
                }
            } else {
                if match_pos > 0 {
                    for i in 0..match_pos {
                        result.push(delimiter_chars[i]);
                    }
                    match_pos = 0;
                }
                result.push(c);
            }
        }

        result
    }

    fn elements_to_rich_text(&self, elements: &[MarkdownElement]) -> RichText {
        let mut rich_text = RichText::new();

        for element in elements {
            match element {
                MarkdownElement::Heading { level, content } => {
                    let heading_style = match level {
                        1 => TextSpan::text(content).bold(),
                        2 => TextSpan::text(content).bold(),
                        3 => TextSpan::text(content).bold().underline(),
                        _ => TextSpan::text(content).underline(),
                    };
                    rich_text.add_span(heading_style);
                    rich_text.add_span(TextSpan::text("\n\n"));
                }
                MarkdownElement::Paragraph { spans } => {
                    for span in spans {
                        rich_text.add_span(span.clone());
                    }
                    rich_text.add_span(TextSpan::text("\n\n"));
                }
                MarkdownElement::CodeBlock { content, .. } => {
                    rich_text.add_span(TextSpan::code_block(content));
                    rich_text.add_span(TextSpan::text("\n\n"));
                }
                MarkdownElement::InlineCode { content } => {
                    rich_text.add_span(TextSpan::code(content));
                }
                MarkdownElement::Bold { content } => {
                    rich_text.add_span(TextSpan::text(content).bold());
                }
                MarkdownElement::Italic { content } => {
                    rich_text.add_span(TextSpan::text(content).italic());
                }
                MarkdownElement::Strikethrough { content } => {
                    rich_text.add_span(TextSpan::text(content).dim());
                }
                MarkdownElement::Link { text, url } => {
                    rich_text.add_span(TextSpan::link(text, url));
                }
                MarkdownElement::UnorderedList { items } => {
                    for item in items {
                        rich_text.add_span(TextSpan::text(format!("• {}\n", item)));
                    }
                    rich_text.add_span(TextSpan::text("\n"));
                }
                MarkdownElement::OrderedList { items } => {
                    for (i, item) in items.iter().enumerate() {
                        rich_text.add_span(TextSpan::text(format!("{}. {}\n", i + 1, item)));
                    }
                    rich_text.add_span(TextSpan::text("\n"));
                }
                MarkdownElement::Blockquote { content } => {
                    rich_text.add_span(TextSpan::text(format!("> {}\n\n", content)));
                }
                MarkdownElement::HorizontalRule => {
                    rich_text.add_span(TextSpan::text("\n────────────────────\n\n"));
                }
                MarkdownElement::Text { content } => {
                    rich_text.add_span(TextSpan::text(content));
                }
                MarkdownElement::LineBreak => {
                    rich_text.add_span(TextSpan::text("\n"));
                }
            }
        }

        rich_text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_parser_creation() {
        let parser = MarkdownParser::new();
        assert!(parser.elements.is_empty());
    }

    #[test]
    fn test_parse_heading() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("# Heading 1");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Heading { level, content } => {
                assert_eq!(*level, 1);
                assert_eq!(content, "Heading 1");
            }
            _ => panic!("Expected Heading element"),
        }
    }

    #[test]
    fn test_parse_multiple_headings() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("# H1\n## H2\n### H3");

        assert_eq!(elements.len(), 3);
        assert!(matches!(
            &elements[0],
            MarkdownElement::Heading { level: 1, .. }
        ));
        assert!(matches!(
            &elements[1],
            MarkdownElement::Heading { level: 2, .. }
        ));
        assert!(matches!(
            &elements[2],
            MarkdownElement::Heading { level: 3, .. }
        ));
    }

    #[test]
    fn test_parse_bold_text() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("This is **bold** text");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Paragraph { spans } => {
                assert_eq!(spans.len(), 3);
                assert!(spans[1].is_bold());
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_italic_text() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("This is *italic* text");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Paragraph { spans } => {
                assert_eq!(spans.len(), 3);
                assert!(spans[1].is_italic());
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_inline_code() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("This is `code` text");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Paragraph { spans } => {
                assert_eq!(spans.len(), 3);
                assert_eq!(spans[1].as_code(), Some("code"));
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_code_block() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("```rust\nfn main() {}\n```");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::CodeBlock { language, content } => {
                assert_eq!(language.as_deref(), Some("rust"));
                assert_eq!(content, "fn main() {}");
            }
            _ => panic!("Expected CodeBlock element"),
        }
    }

    #[test]
    fn test_parse_link() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("[Click here](https://example.com)");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Paragraph { spans } => {
                assert_eq!(spans.len(), 1);
                assert_eq!(
                    spans[0].as_link(),
                    Some(("Click here", "https://example.com"))
                );
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_unordered_list() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("- Item 1\n- Item 2");

        assert_eq!(elements.len(), 2);
        match &elements[0] {
            MarkdownElement::UnorderedList { items } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0], "Item 1");
            }
            _ => panic!("Expected UnorderedList element"),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("1. Item 1\n2. Item 2");

        assert_eq!(elements.len(), 2);
        match &elements[0] {
            MarkdownElement::OrderedList { items } => {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0], "Item 1");
            }
            _ => panic!("Expected OrderedList element"),
        }
    }

    #[test]
    fn test_parse_blockquote() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("> This is a quote");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Blockquote { content } => {
                assert_eq!(content, "This is a quote");
            }
            _ => panic!("Expected Blockquote element"),
        }
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("---");

        assert_eq!(elements.len(), 1);
        assert!(matches!(&elements[0], MarkdownElement::HorizontalRule));
    }

    #[test]
    fn test_parse_paragraph() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("This is a paragraph.\nWith multiple lines.");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Paragraph { spans } => {
                assert!(!spans.is_empty());
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_parse_mixed_content() {
        let mut parser = MarkdownParser::new();
        let markdown =
            "# Title\n\nThis is **bold** and *italic*.\n\n```rust\ncode here\n```\n\n- List item";
        let elements = parser.parse(markdown);

        assert_eq!(elements.len(), 4);
        assert!(matches!(&elements[0], MarkdownElement::Heading { .. }));
        assert!(matches!(&elements[1], MarkdownElement::Paragraph { .. }));
        assert!(matches!(&elements[2], MarkdownElement::CodeBlock { .. }));
        assert!(matches!(
            &elements[3],
            MarkdownElement::UnorderedList { .. }
        ));
    }

    #[test]
    fn test_parse_to_rich_text() {
        let mut parser = MarkdownParser::new();
        let rich_text = parser.parse_to_rich_text("# Heading\n\nThis is **bold** text.");

        assert!(!rich_text.is_empty());
        let plain = rich_text.plain_text();
        assert!(plain.contains("Heading"));
        assert!(plain.contains("bold"));
    }

    #[test]
    fn test_strikethrough_text() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("This is ~~strikethrough~~ text");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Paragraph { spans } => {
                assert_eq!(spans.len(), 3);
                if let TextSpan::Text { dim, .. } = &spans[1] {
                    assert!(*dim);
                } else {
                    panic!("Expected Text variant with dim modifier");
                }
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_multiple_code_blocks() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("```rust\nfn main() {}\n```\n\n```python\nprint('hello')\n```");

        assert_eq!(elements.len(), 2);
        assert!(matches!(&elements[0], MarkdownElement::CodeBlock { .. }));
        assert!(matches!(&elements[1], MarkdownElement::CodeBlock { .. }));
    }

    #[test]
    fn test_escaped_characters() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse(r"This is \*not\* italic");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::Paragraph { spans } => {
                let plain = spans.iter().map(|s| s.content()).collect::<String>();
                assert!(plain.contains("*not*"));
            }
            _ => panic!("Expected Paragraph element"),
        }
    }

    #[test]
    fn test_code_block_without_language() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("```\ncode here\n```");

        assert_eq!(elements.len(), 1);
        match &elements[0] {
            MarkdownElement::CodeBlock { language, content } => {
                assert!(language.is_none());
                assert_eq!(content, "code here");
            }
            _ => panic!("Expected CodeBlock element"),
        }
    }

    #[test]
    fn test_empty_markdown() {
        let mut parser = MarkdownParser::new();
        let elements = parser.parse("");

        assert!(elements.is_empty());
    }

    #[test]
    fn test_markdown_parser_reuse() {
        let mut parser = MarkdownParser::new();
        let elements1 = parser.parse("# First");
        let elements2 = parser.parse("# Second");

        assert_eq!(elements1.len(), 1);
        assert_eq!(elements2.len(), 1);
        assert_ne!(elements1, elements2);
    }
}
