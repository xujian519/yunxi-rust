#[derive(Debug, Clone, PartialEq)]
pub enum TextSpan {
    Text {
        content: String,
        bold: bool,
        italic: bool,
        underline: bool,
        dim: bool,
        blink: bool,
        reverse: bool,
        hidden: bool,
    },
    Link {
        text: String,
        url: String,
    },
    Code {
        content: String,
        inline: bool,
    },
}

impl TextSpan {
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            content: content.into(),
            bold: false,
            italic: false,
            underline: false,
            dim: false,
            blink: false,
            reverse: false,
            hidden: false,
        }
    }

    pub fn bold(mut self) -> Self {
        if let Self::Text { ref mut bold, .. } = self {
            *bold = true;
        }
        self
    }

    pub fn italic(mut self) -> Self {
        if let Self::Text { ref mut italic, .. } = self {
            *italic = true;
        }
        self
    }

    pub fn underline(mut self) -> Self {
        if let Self::Text {
            ref mut underline, ..
        } = self
        {
            *underline = true;
        }
        self
    }

    pub fn dim(mut self) -> Self {
        if let Self::Text { ref mut dim, .. } = self {
            *dim = true;
        }
        self
    }

    pub fn blink(mut self) -> Self {
        if let Self::Text { ref mut blink, .. } = self {
            *blink = true;
        }
        self
    }

    pub fn reverse(mut self) -> Self {
        if let Self::Text {
            ref mut reverse, ..
        } = self
        {
            *reverse = true;
        }
        self
    }

    pub fn hidden(mut self) -> Self {
        if let Self::Text { ref mut hidden, .. } = self {
            *hidden = true;
        }
        self
    }

    pub fn link(text: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Link {
            text: text.into(),
            url: url.into(),
        }
    }

    pub fn code(content: impl Into<String>) -> Self {
        Self::Code {
            content: content.into(),
            inline: true,
        }
    }

    pub fn code_block(content: impl Into<String>) -> Self {
        Self::Code {
            content: content.into(),
            inline: false,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { content, .. } => Some(content),
            _ => None,
        }
    }

    pub fn as_link(&self) -> Option<(&str, &str)> {
        match self {
            Self::Link { text, url } => Some((text, url)),
            _ => None,
        }
    }

    pub fn as_code(&self) -> Option<&str> {
        match self {
            Self::Code { content, .. } => Some(content),
            _ => None,
        }
    }

    pub fn content(&self) -> String {
        match self {
            Self::Text { content, .. } => content.clone(),
            Self::Link { text, .. } => text.clone(),
            Self::Code { content, .. } => content.clone(),
        }
    }

    pub fn is_bold(&self) -> bool {
        matches!(self, Self::Text { bold: true, .. })
    }

    pub fn is_italic(&self) -> bool {
        matches!(self, Self::Text { italic: true, .. })
    }

    pub fn is_underline(&self) -> bool {
        matches!(
            self,
            Self::Text {
                underline: true,
                ..
            }
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct RichText {
    spans: Vec<TextSpan>,
}

impl RichText {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_span(mut self, span: TextSpan) -> Self {
        self.spans.push(span);
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.spans.push(TextSpan::text(text));
        self
    }

    pub fn with_bold(mut self, text: impl Into<String>) -> Self {
        self.spans.push(TextSpan::text(text).bold());
        self
    }

    pub fn with_italic(mut self, text: impl Into<String>) -> Self {
        self.spans.push(TextSpan::text(text).italic());
        self
    }

    pub fn with_underline(mut self, text: impl Into<String>) -> Self {
        self.spans.push(TextSpan::text(text).underline());
        self
    }

    pub fn with_link(mut self, text: impl Into<String>, url: impl Into<String>) -> Self {
        self.spans.push(TextSpan::link(text, url));
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.spans.push(TextSpan::code(code));
        self
    }

    pub fn with_code_block(mut self, code: impl Into<String>) -> Self {
        self.spans.push(TextSpan::code_block(code));
        self
    }

    pub fn add_span(&mut self, span: TextSpan) {
        self.spans.push(span);
    }

    pub fn add_text(&mut self, text: impl Into<String>) {
        self.spans.push(TextSpan::text(text));
    }

    pub fn spans(&self) -> &[TextSpan] {
        &self.spans
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn clear(&mut self) {
        self.spans.clear();
    }

    pub fn plain_text(&self) -> String {
        self.spans.iter().map(|s| s.content()).collect()
    }
}

impl From<Vec<TextSpan>> for RichText {
    fn from(spans: Vec<TextSpan>) -> Self {
        Self { spans }
    }
}

impl From<&str> for RichText {
    fn from(text: &str) -> Self {
        Self {
            spans: vec![TextSpan::text(text)],
        }
    }
}

impl From<String> for RichText {
    fn from(s: String) -> Self {
        Self {
            spans: vec![TextSpan::text(s)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_span_creation() {
        let span = TextSpan::text("Hello");
        assert_eq!(span.as_text(), Some("Hello"));
        assert!(!span.is_bold());
        assert!(!span.is_italic());
    }

    #[test]
    fn test_text_span_bold() {
        let span = TextSpan::text("Hello").bold();
        assert!(span.is_bold());
    }

    #[test]
    fn test_text_span_italic() {
        let span = TextSpan::text("Hello").italic();
        assert!(span.is_italic());
    }

    #[test]
    fn test_text_span_underline() {
        let span = TextSpan::text("Hello").underline();
        assert!(span.is_underline());
    }

    #[test]
    fn test_text_span_combined_styles() {
        let span = TextSpan::text("Hello").bold().italic().underline();
        assert!(span.is_bold());
        assert!(span.is_italic());
        assert!(span.is_underline());
    }

    #[test]
    fn test_text_span_all_modifiers() {
        let span = TextSpan::text("Hello")
            .bold()
            .italic()
            .underline()
            .dim()
            .blink()
            .reverse()
            .hidden();

        if let TextSpan::Text {
            bold,
            italic,
            underline,
            dim,
            blink,
            reverse,
            hidden,
            ..
        } = span
        {
            assert!(bold);
            assert!(italic);
            assert!(underline);
            assert!(dim);
            assert!(blink);
            assert!(reverse);
            assert!(hidden);
        } else {
            panic!("Expected Text variant");
        }
    }

    #[test]
    fn test_text_span_link() {
        let span = TextSpan::link("Click here", "https://example.com");
        assert_eq!(span.as_link(), Some(("Click here", "https://example.com")));
        assert_eq!(span.content(), "Click here");
    }

    #[test]
    fn test_text_span_code() {
        let span = TextSpan::code("println!(\"Hello\")");
        assert_eq!(span.as_code(), Some("println!(\"Hello\")"));
        assert_eq!(span.content(), "println!(\"Hello\")");
    }

    #[test]
    fn test_text_span_code_block() {
        let span = TextSpan::code_block("fn main() { println!(\"Hello\"); }");
        if let TextSpan::Code { inline, .. } = span {
            assert!(!inline);
        } else {
            panic!("Expected Code variant");
        }
    }

    #[test]
    fn test_rich_text_creation() {
        let rich_text = RichText::new();
        assert!(rich_text.is_empty());
        assert_eq!(rich_text.len(), 0);
    }

    #[test]
    fn test_rich_text_with_text() {
        let rich_text = RichText::new().with_text("Hello");
        assert_eq!(rich_text.len(), 1);
        assert_eq!(rich_text.plain_text(), "Hello");
    }

    #[test]
    fn test_rich_text_with_multiple_spans() {
        let rich_text = RichText::new()
            .with_text("Hello ")
            .with_bold("World")
            .with_text("!")
            .with_link(" Click here", "https://example.com");

        assert_eq!(rich_text.len(), 4);
        assert_eq!(rich_text.plain_text(), "Hello World! Click here");
    }

    #[test]
    fn test_rich_text_add_span() {
        let mut rich_text = RichText::new();
        rich_text.add_span(TextSpan::text("Hello"));
        rich_text.add_span(TextSpan::text(" ").bold());
        rich_text.add_span(TextSpan::text("World"));

        assert_eq!(rich_text.len(), 3);
    }

    #[test]
    fn test_rich_text_from_vec() {
        let spans = vec![
            TextSpan::text("Hello"),
            TextSpan::text(" ").bold(),
            TextSpan::text("World"),
        ];
        let rich_text = RichText::from(spans);
        assert_eq!(rich_text.len(), 3);
    }

    #[test]
    fn test_rich_text_from_str() {
        let rich_text = RichText::from("Hello World");
        assert_eq!(rich_text.len(), 1);
        assert_eq!(rich_text.plain_text(), "Hello World");
    }

    #[test]
    fn test_rich_text_clear() {
        let mut rich_text = RichText::new().with_text("Hello").with_bold("World");

        rich_text.clear();
        assert!(rich_text.is_empty());
        assert_eq!(rich_text.len(), 0);
    }

    #[test]
    fn test_rich_text_builder_pattern() {
        let rich_text = RichText::new()
            .with_text("这是一段")
            .with_bold("富文本")
            .with_text("示例，包含")
            .with_italic("斜体")
            .with_text("和")
            .with_underline("下划线")
            .with_text("，以及")
            .with_code("代码")
            .with_text("。");

        assert_eq!(rich_text.len(), 7);
        assert!(rich_text.plain_text().contains("富文本"));
        assert!(rich_text.plain_text().contains("斜体"));
        assert!(rich_text.plain_text().contains("下划线"));
        assert!(rich_text.plain_text().contains("代码"));
    }

    #[test]
    fn test_text_span_content_extraction() {
        let text_span = TextSpan::text("Test");
        let link_span = TextSpan::link("Link", "https://example.com");
        let code_span = TextSpan::code("code");

        assert_eq!(text_span.content(), "Test");
        assert_eq!(link_span.content(), "Link");
        assert_eq!(code_span.content(), "code");
    }

    #[test]
    fn test_rich_text_with_code_block() {
        let rich_text = RichText::new()
            .with_text("这是一个代码块：\n")
            .with_code_block("fn main() {\n    println!(\"Hello\");\n}");

        assert_eq!(rich_text.len(), 2);

        if let TextSpan::Code { inline, .. } = &rich_text.spans()[1] {
            assert!(!inline);
        } else {
            panic!("Expected Code variant");
        }
    }

    #[test]
    fn test_text_span_clone() {
        let span = TextSpan::text("Hello").bold().italic();
        let cloned = span.clone();
        assert_eq!(span.content(), cloned.content());
        assert_eq!(span.is_bold(), cloned.is_bold());
        assert_eq!(span.is_italic(), cloned.is_italic());
    }

    #[test]
    fn test_rich_text_clone() {
        let rich_text = RichText::new()
            .with_text("Hello")
            .with_bold("World")
            .with_link("Link", "https://example.com");

        let cloned = rich_text.clone();
        assert_eq!(rich_text.len(), cloned.len());
        assert_eq!(rich_text.plain_text(), cloned.plain_text());
    }
}
