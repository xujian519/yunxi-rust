use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use std::sync::Arc;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme as SyntectTheme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxSet};
use syntect::util::LinesWithEndings;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Markdown,
    Json,
    Toml,
    Yaml,
    Plain,
}

impl SyntaxLanguage {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => SyntaxLanguage::Rust,
            "py" | "pyw" => SyntaxLanguage::Python,
            "js" => SyntaxLanguage::JavaScript,
            "ts" => SyntaxLanguage::TypeScript,
            "md" | "markdown" => SyntaxLanguage::Markdown,
            "json" => SyntaxLanguage::Json,
            "toml" => SyntaxLanguage::Toml,
            "yaml" | "yml" => SyntaxLanguage::Yaml,
            _ => SyntaxLanguage::Plain,
        }
    }

    pub fn detect_from_content(content: &str) -> Self {
        let lower = content.to_lowercase();

        if lower.contains("fn main()") || lower.contains("impl ") || lower.contains("pub struct ") {
            SyntaxLanguage::Rust
        } else if lower.contains("def ") && lower.contains(":") {
            SyntaxLanguage::Python
        } else if lower.contains("function ") || lower.contains("const ") || lower.contains("=>") {
            SyntaxLanguage::JavaScript
        } else if lower.contains("interface ") || lower.contains(": type ") {
            SyntaxLanguage::TypeScript
        } else if content.starts_with("# ") || content.contains("## ") {
            SyntaxLanguage::Markdown
        } else if content.trim_start().starts_with('{') && content.contains("\"") {
            SyntaxLanguage::Json
        } else if content.contains("[") && content.contains("=") {
            SyntaxLanguage::Toml
        } else if content.contains(": ") && content.contains("- ") {
            SyntaxLanguage::Yaml
        } else {
            SyntaxLanguage::Plain
        }
    }
}

pub struct SyntaxHighlighter {
    syntax_set: Arc<SyntaxSet>,
    theme_set: Arc<ThemeSet>,
    current_language: SyntaxLanguage,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = Arc::new(SyntaxSet::load_defaults_newlines());
        let theme_set = Arc::new(ThemeSet::load_defaults());

        Self {
            syntax_set,
            theme_set,
            current_language: SyntaxLanguage::Plain,
        }
    }

    pub fn set_language(&mut self, language: SyntaxLanguage) {
        self.current_language = language;
    }

    pub fn highlight(&self, code: &str, width: usize) -> Text<'static> {
        if self.current_language == SyntaxLanguage::Plain {
            return Text::from(code.to_string());
        }

        let syntax = self.get_syntax_for_language();
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(&syntax, theme);

        let mut lines = Vec::new();

        for line in LinesWithEndings::from(code) {
            let ranges = highlighter.highlight_line(line, &self.syntax_set);
            let mut spans = Vec::new();

            for (style, text) in ranges {
                let color = self.syntect_color_to_ratatui_color(style.foreground);
                let span = Span::styled(text.to_string(), Style::default().fg(color));
                spans.push(span);
            }

            let trimmed_spans = self.trim_spans_to_width(spans, width);
            lines.push(Line::from(trimmed_spans));
        }

        Text::from(lines)
    }

    fn trim_spans_to_width(&self, spans: Vec<Span<'static>>, width: usize) -> Vec<Span<'static>> {
        let mut total_width = 0;
        let mut result = Vec::new();

        for span in spans {
            let span_width = span.content.len();

            if total_width + span_width <= width {
                result.push(span);
                total_width += span_width;
            } else if total_width < width {
                let remaining = width - total_width;
                let content = span.content;
                let truncated = String::from(&content[..remaining.min(content.len())]);
                result.push(Span::styled(truncated, span.style));
                break;
            } else {
                break;
            }
        }

        result
    }

    fn get_syntax_for_language(&self) -> SyntaxDefinition {
        let name = match self.current_language {
            SyntaxLanguage::Rust => "Rust",
            SyntaxLanguage::Python => "Python",
            SyntaxLanguage::JavaScript => "JavaScript",
            SyntaxLanguage::TypeScript => "TypeScript",
            SyntaxLanguage::Markdown => "Markdown",
            SyntaxLanguage::Json => "JSON",
            SyntaxLanguage::Toml => "TOML",
            SyntaxLanguage::Yaml => "YAML",
            SyntaxLanguage::Plain => "Plain Text",
        };

        self.syntax_set
            .find_syntax_by_name(name)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            .clone()
    }

    fn syntect_color_to_ratatui_color(&self, color: syntect::highlighting::Color) -> Color {
        Color::Rgb(color.r, color.g, color.b)
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(SyntaxLanguage::from_extension("rs"), SyntaxLanguage::Rust);
        assert_eq!(SyntaxLanguage::from_extension("py"), SyntaxLanguage::Python);
        assert_eq!(
            SyntaxLanguage::from_extension("js"),
            SyntaxLanguage::JavaScript
        );
        assert_eq!(
            SyntaxLanguage::from_extension("ts"),
            SyntaxLanguage::TypeScript
        );
        assert_eq!(
            SyntaxLanguage::from_extension("md"),
            SyntaxLanguage::Markdown
        );
        assert_eq!(SyntaxLanguage::from_extension("json"), SyntaxLanguage::Json);
        assert_eq!(SyntaxLanguage::from_extension("toml"), SyntaxLanguage::Toml);
        assert_eq!(SyntaxLanguage::from_extension("yaml"), SyntaxLanguage::Yaml);
        assert_eq!(SyntaxLanguage::from_extension("txt"), SyntaxLanguage::Plain);
    }

    #[test]
    fn test_detect_rust_from_content() {
        let code = "fn main() { println!(\"Hello\"); }";
        assert_eq!(
            SyntaxLanguage::detect_from_content(code),
            SyntaxLanguage::Rust
        );
    }

    #[test]
    fn test_detect_python_from_content() {
        let code = "def main():\n    print(\"Hello\")";
        assert_eq!(
            SyntaxLanguage::detect_from_content(code),
            SyntaxLanguage::Python
        );
    }

    #[test]
    fn test_detect_javascript_from_content() {
        let code = "function main() { console.log(\"Hello\"); }";
        assert_eq!(
            SyntaxLanguage::detect_from_content(code),
            SyntaxLanguage::JavaScript
        );
    }

    #[test]
    fn test_detect_markdown_from_content() {
        let code = "# Header\n\nThis is a paragraph.";
        assert_eq!(
            SyntaxLanguage::detect_from_content(code),
            SyntaxLanguage::Markdown
        );
    }

    #[test]
    fn test_detect_json_from_content() {
        let code = "{\"key\": \"value\"}";
        assert_eq!(
            SyntaxLanguage::detect_from_content(code),
            SyntaxLanguage::Json
        );
    }

    #[test]
    fn test_highlighter_creation() {
        let highlighter = SyntaxHighlighter::new();
        assert_eq!(highlighter.current_language, SyntaxLanguage::Plain);
    }

    #[test]
    fn test_set_language() {
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(SyntaxLanguage::Rust);
        assert_eq!(highlighter.current_language, SyntaxLanguage::Rust);
    }

    #[test]
    fn test_highlight_plain_text() {
        let highlighter = SyntaxHighlighter::new();
        let code = "Hello, World!";
        let result = highlighter.highlight(code, 100);
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn test_highlight_rust_code() {
        let mut highlighter = SyntaxHighlighter::new();
        highlighter.set_language(SyntaxLanguage::Rust);
        let code = "fn main() { println!(\"Hello\"); }";
        let result = highlighter.highlight(code, 100);
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn test_highlight_with_width_limit() {
        let highlighter = SyntaxHighlighter::new();
        let code = "This is a very long line that should be truncated";
        let result = highlighter.highlight(code, 20);
        assert!(!result.lines.is_empty());
    }
}
