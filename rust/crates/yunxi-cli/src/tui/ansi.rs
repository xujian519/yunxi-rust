//! 终端 ANSI 着色工具（TUI / REPL 共用）。

use crossterm::style::Color;

/// 为文本套上前景色 ANSI 转义序列。
pub(crate) fn color_fg(text: &str, color: Color) -> String {
    match color {
        Color::Rgb { r, g, b } => format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m"),
        Color::AnsiValue(n) => format!("\x1b[38;5;{n}m{text}\x1b[0m"),
        _ => {
            let code = match color {
                Color::Black => "30",
                Color::DarkRed => "31",
                Color::DarkGreen => "32",
                Color::DarkYellow => "33",
                Color::DarkBlue => "34",
                Color::DarkMagenta => "35",
                Color::DarkCyan => "36",
                Color::Grey => "37",
                Color::DarkGrey => "90",
                Color::Red => "91",
                Color::Green => "92",
                Color::Yellow => "93",
                Color::Blue => "94",
                Color::Magenta => "95",
                Color::Cyan => "96",
                Color::White => "97",
                Color::Reset => "0",
                Color::Rgb { .. } | Color::AnsiValue(_) => unreachable!(),
            };
            format!("\x1b[{code}m{text}\x1b[0m")
        }
    }
}
