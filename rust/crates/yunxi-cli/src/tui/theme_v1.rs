#![allow(dead_code)]

use crossterm::style::Color;

/// 命名主题。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThemeName {
    Dark,
    Light,
    Monokai,
    Nord,
    HighContrast,
    NoColor,
}

impl ThemeName {
    /// 从字符串解析主题名（命令行参数用）。
    pub(crate) fn from_str(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            "monokai" => Some(Self::Monokai),
            "nord" => Some(Self::Nord),
            "high-contrast" | "highcontrast" => Some(Self::HighContrast),
            "no-color" | "nocolor" | "none" => Some(Self::NoColor),
            _ => None,
        }
    }

    /// 所有可用的主题名列表。
    pub(crate) fn all_names() -> &'static [&'static str] {
        &[
            "dark",
            "light",
            "monokai",
            "nord",
            "high-contrast",
            "no-color",
        ]
    }
}

/// 终端颜色级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorLevel {
    NoColor,
    Basic16,
    Extended256,
    TrueColor,
}

/// Spinner 动画样式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SpinnerStyle {
    Braille,
    Dots,
    Line,
    Arrow,
    Moon,
}

impl SpinnerStyle {
    pub(crate) fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            Self::Dots => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            Self::Line => &["-", "\\", "|", "/"],
            Self::Arrow => &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            Self::Moon => &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
        }
    }
}

/// 扩展主题配置。
#[derive(Debug, Clone)]
pub(crate) struct ExtendedTheme {
    /// 前景/强调色。
    pub foreground: Color,
    pub heading: Color,
    pub emphasis: Color,
    pub strong: Color,
    pub inline_code: Color,
    pub link: Color,
    pub quote: Color,
    pub spinner_active: Color,
    pub spinner_done: Color,
    pub spinner_failed: Color,
    /// Spinner 动画。
    pub spinner_style: SpinnerStyle,
    /// 颜色级别。
    pub color_level: ColorLevel,
}

impl ExtendedTheme {
    /// 根据命名主题生成扩展主题。
    pub(crate) fn from_name(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self::dark(),
            ThemeName::Light => Self::light(),
            ThemeName::Monokai => Self::monokai(),
            ThemeName::Nord => Self::nord(),
            ThemeName::HighContrast => Self::high_contrast(),
            ThemeName::NoColor => Self::no_color(),
        }
    }

    fn dark() -> Self {
        Self {
            foreground: Color::White,
            heading: Color::Cyan,
            emphasis: Color::Magenta,
            strong: Color::Yellow,
            inline_code: Color::Green,
            link: Color::Blue,
            quote: Color::DarkGrey,
            spinner_active: Color::Blue,
            spinner_done: Color::Green,
            spinner_failed: Color::Red,
            spinner_style: SpinnerStyle::Braille,
            color_level: ColorLevel::TrueColor,
        }
    }

    fn light() -> Self {
        Self {
            foreground: Color::Black,
            heading: Color::DarkBlue,
            emphasis: Color::DarkMagenta,
            strong: Color::DarkYellow,
            inline_code: Color::DarkGreen,
            link: Color::Blue,
            quote: Color::Grey,
            spinner_active: Color::DarkCyan,
            spinner_done: Color::DarkGreen,
            spinner_failed: Color::DarkRed,
            spinner_style: SpinnerStyle::Dots,
            color_level: ColorLevel::TrueColor,
        }
    }

    fn monokai() -> Self {
        Self {
            foreground: Color::White,
            heading: Color::Cyan,
            emphasis: Color::Magenta,
            strong: Color::Yellow,
            inline_code: Color::Green,
            link: Color::Cyan,
            quote: Color::DarkGrey,
            spinner_active: Color::Yellow,
            spinner_done: Color::Green,
            spinner_failed: Color::Red,
            spinner_style: SpinnerStyle::Arrow,
            color_level: ColorLevel::TrueColor,
        }
    }

    fn nord() -> Self {
        Self {
            foreground: Color::White,
            heading: Color::Cyan,
            emphasis: Color::Magenta,
            strong: Color::Blue,
            inline_code: Color::Cyan,
            link: Color::Blue,
            quote: Color::DarkGrey,
            spinner_active: Color::Cyan,
            spinner_done: Color::Green,
            spinner_failed: Color::Red,
            spinner_style: SpinnerStyle::Moon,
            color_level: ColorLevel::TrueColor,
        }
    }

    fn high_contrast() -> Self {
        Self {
            foreground: Color::White,
            heading: Color::Cyan,
            emphasis: Color::Magenta,
            strong: Color::Yellow,
            inline_code: Color::Green,
            link: Color::Cyan,
            quote: Color::White,
            spinner_active: Color::Yellow,
            spinner_done: Color::Green,
            spinner_failed: Color::Red,
            spinner_style: SpinnerStyle::Braille,
            color_level: ColorLevel::Basic16,
        }
    }

    fn no_color() -> Self {
        Self {
            foreground: Color::Reset,
            heading: Color::Reset,
            emphasis: Color::Reset,
            strong: Color::Reset,
            inline_code: Color::Reset,
            link: Color::Reset,
            quote: Color::Reset,
            spinner_active: Color::Reset,
            spinner_done: Color::Reset,
            spinner_failed: Color::Reset,
            spinner_style: SpinnerStyle::Line,
            color_level: ColorLevel::NoColor,
        }
    }
}

/// 检测终端颜色级别。
pub(crate) fn detect_color_level() -> ColorLevel {
    if std::env::var_os("NO_COLOR").is_some() {
        return ColorLevel::NoColor;
    }
    match std::env::var("COLORTERM").as_deref() {
        Ok("truecolor" | "24bit") => ColorLevel::TrueColor,
        Ok(_) => ColorLevel::Extended256,
        Err(_) => match std::env::var("TERM").as_deref() {
            Ok(v) if v.contains("256color") => ColorLevel::Extended256,
            Ok(v) if v.contains("xterm") || v.contains("screen") => ColorLevel::Basic16,
            _ => ColorLevel::Basic16,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_name_parsing() {
        assert_eq!(ThemeName::from_str("dark"), Some(ThemeName::Dark));
        assert_eq!(ThemeName::from_str("LIGHT"), Some(ThemeName::Light));
        assert_eq!(ThemeName::from_str("monokai"), Some(ThemeName::Monokai));
        assert_eq!(ThemeName::from_str("nord"), Some(ThemeName::Nord));
        assert_eq!(
            ThemeName::from_str("high-contrast"),
            Some(ThemeName::HighContrast)
        );
        assert_eq!(ThemeName::from_str("no-color"), Some(ThemeName::NoColor));
        assert_eq!(ThemeName::from_str("invalid"), None);
    }

    #[test]
    fn all_themes_produce_valid_extended_themes() {
        for name in ThemeName::all_names() {
            let theme_name = ThemeName::from_str(name).expect("name should parse");
            let theme = ExtendedTheme::from_name(theme_name);
            // 验证可以正常构造
            let _ = &theme.foreground;
        }
    }

    #[test]
    fn spinner_styles_have_frames() {
        for style in &[
            SpinnerStyle::Braille,
            SpinnerStyle::Dots,
            SpinnerStyle::Line,
            SpinnerStyle::Arrow,
            SpinnerStyle::Moon,
        ] {
            assert!(!style.frames().is_empty());
        }
    }

    #[test]
    fn no_color_theme_uses_reset() {
        let theme = ExtendedTheme::from_name(ThemeName::NoColor);
        assert_eq!(theme.foreground, Color::Reset);
        assert_eq!(theme.heading, Color::Reset);
        assert_eq!(theme.color_level, ColorLevel::NoColor);
    }

    #[test]
    fn detect_color_level_returns_some_level() {
        let level = detect_color_level();
        // 只验证它返回了一个有效值，不验证具体值（取决于环境变量）
        assert!(matches!(
            level,
            ColorLevel::NoColor
                | ColorLevel::Basic16
                | ColorLevel::Extended256
                | ColorLevel::TrueColor
        ));
    }
}
