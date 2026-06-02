use ratatui::style::{Color, Modifier, Style};
use std::collections::HashMap;
use std::sync::Arc;
use syntect::highlighting::{Theme as SyntectTheme, ThemeSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyntaxThemeName {
    Base16Ocean,
    Monokai,
    SolarizedDark,
    SolarizedLight,
    Nord,
    Dracula,
    Custom(String),
}

impl SyntaxThemeName {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Base16Ocean,
            Self::Monokai,
            Self::SolarizedDark,
            Self::SolarizedLight,
            Self::Nord,
            Self::Dracula,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Base16Ocean => "base16-ocean.dark",
            Self::Monokai => "Monokai Extended",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
            Self::Nord => "Nord",
            Self::Dracula => "Dracula",
            Self::Custom(name) => name,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "base16-ocean" | "ocean" => Some(Self::Base16Ocean),
            "monokai" => Some(Self::Monokai),
            "solarized-dark" | "solarized dark" => Some(Self::SolarizedDark),
            "solarized-light" | "solarized light" => Some(Self::SolarizedLight),
            "nord" => Some(Self::Nord),
            "dracula" => Some(Self::Dracula),
            _ => Some(Self::Custom(s.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    pub name: SyntaxThemeName,
    pub is_dark: bool,
    pub colors: SyntaxColors,
}

#[derive(Debug, Clone)]
pub struct SyntaxColors {
    pub keyword: Color,
    pub string: Color,
    pub comment: Color,
    pub function: Color,
    pub number: Color,
    pub operator: Color,
    pub variable: Color,
    pub type_name: Color,
    pub constant: Color,
    pub background: Color,
    pub foreground: Color,
}

pub struct SyntaxThemeManager {
    theme_set: Arc<ThemeSet>,
    themes: HashMap<SyntaxThemeName, SyntaxTheme>,
    current_theme: SyntaxThemeName,
}

impl SyntaxThemeManager {
    pub fn new() -> Self {
        let theme_set = Arc::new(ThemeSet::load_defaults());
        let mut manager = Self {
            theme_set,
            themes: HashMap::new(),
            current_theme: SyntaxThemeName::Base16Ocean,
        };

        manager.register_builtin_themes();
        manager
    }

    fn register_builtin_themes(&mut self) {
        self.register_theme(SyntaxTheme::base16_ocean());
        self.register_theme(SyntaxTheme::monokai());
        self.register_theme(SyntaxTheme::solarized_dark());
        self.register_theme(SyntaxTheme::solarized_light());
        self.register_theme(SyntaxTheme::nord());
        self.register_theme(SyntaxTheme::dracula());
    }

    pub fn register_theme(&mut self, theme: SyntaxTheme) {
        self.themes.insert(theme.name.clone(), theme);
    }

    pub fn set_current_theme(&mut self, name: SyntaxThemeName) {
        self.current_theme = name;
    }

    pub fn get_current_theme(&self) -> &SyntaxTheme {
        self.themes
            .get(&self.current_theme)
            .unwrap_or_else(|| self.themes.get(&SyntaxThemeName::Base16Ocean).unwrap())
    }

    pub fn get_theme(&self, name: &SyntaxThemeName) -> Option<&SyntaxTheme> {
        self.themes.get(name)
    }

    pub fn list_themes(&self) -> Vec<&SyntaxThemeName> {
        self.themes.keys().collect()
    }

    pub fn get_syntect_theme(&self, name: &SyntaxThemeName) -> Option<&SyntectTheme> {
        let theme_name = name.as_str();
        self.theme_set.themes.get(theme_name)
    }
}

impl Default for SyntaxThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxTheme {
    fn base16_ocean() -> Self {
        Self {
            name: SyntaxThemeName::Base16Ocean,
            is_dark: true,
            colors: SyntaxColors {
                keyword: Color::Rgb(207, 135, 255),
                string: Color::Rgb(209, 154, 102),
                comment: Color::Rgb(106, 153, 85),
                function: Color::Rgb(97, 175, 239),
                number: Color::Rgb(209, 154, 102),
                operator: Color::Rgb(196, 200, 219),
                variable: Color::Rgb(232, 232, 237),
                type_name: Color::Rgb(97, 175, 239),
                constant: Color::Rgb(209, 154, 102),
                background: Color::Rgb(26, 35, 50),
                foreground: Color::Rgb(232, 232, 237),
            },
        }
    }

    fn monokai() -> Self {
        Self {
            name: SyntaxThemeName::Monokai,
            is_dark: true,
            colors: SyntaxColors {
                keyword: Color::Rgb(249, 38, 114),
                string: Color::Rgb(230, 219, 116),
                comment: Color::Rgb(117, 113, 94),
                function: Color::Rgb(166, 226, 46),
                number: Color::Rgb(174, 129, 255),
                operator: Color::Rgb(248, 248, 242),
                variable: Color::Rgb(248, 248, 242),
                type_name: Color::Rgb(166, 226, 46),
                constant: Color::Rgb(174, 129, 255),
                background: Color::Rgb(39, 40, 34),
                foreground: Color::Rgb(248, 248, 242),
            },
        }
    }

    fn solarized_dark() -> Self {
        Self {
            name: SyntaxThemeName::SolarizedDark,
            is_dark: true,
            colors: SyntaxColors {
                keyword: Color::Rgb(181, 137, 0),
                string: Color::Rgb(42, 161, 152),
                comment: Color::Rgb(88, 110, 117),
                function: Color::Rgb(38, 139, 210),
                number: Color::Rgb(133, 153, 0),
                operator: Color::Rgb(147, 161, 161),
                variable: Color::Rgb(253, 246, 227),
                type_name: Color::Rgb(211, 54, 130),
                constant: Color::Rgb(133, 153, 0),
                background: Color::Rgb(0, 43, 54),
                foreground: Color::Rgb(131, 148, 150),
            },
        }
    }

    fn solarized_light() -> Self {
        Self {
            name: SyntaxThemeName::SolarizedLight,
            is_dark: false,
            colors: SyntaxColors {
                keyword: Color::Rgb(133, 153, 0),
                string: Color::Rgb(42, 161, 152),
                comment: Color::Rgb(147, 161, 161),
                function: Color::Rgb(38, 139, 210),
                number: Color::Rgb(181, 137, 0),
                operator: Color::Rgb(88, 110, 117),
                variable: Color::Rgb(101, 123, 131),
                type_name: Color::Rgb(211, 54, 130),
                constant: Color::Rgb(181, 137, 0),
                background: Color::Rgb(253, 246, 227),
                foreground: Color::Rgb(88, 110, 117),
            },
        }
    }

    fn nord() -> Self {
        Self {
            name: SyntaxThemeName::Nord,
            is_dark: true,
            colors: SyntaxColors {
                keyword: Color::Rgb(136, 192, 208),
                string: Color::Rgb(163, 190, 140),
                comment: Color::Rgb(94, 129, 172),
                function: Color::Rgb(129, 161, 193),
                number: Color::Rgb(180, 142, 173),
                operator: Color::Rgb(216, 222, 233),
                variable: Color::Rgb(216, 222, 233),
                type_name: Color::Rgb(94, 129, 172),
                constant: Color::Rgb(180, 142, 173),
                background: Color::Rgb(46, 52, 64),
                foreground: Color::Rgb(216, 222, 233),
            },
        }
    }

    fn dracula() -> Self {
        Self {
            name: SyntaxThemeName::Dracula,
            is_dark: true,
            colors: SyntaxColors {
                keyword: Color::Rgb(255, 121, 198),
                string: Color::Rgb(241, 250, 140),
                comment: Color::Rgb(98, 114, 164),
                function: Color::Rgb(139, 233, 253),
                number: Color::Rgb(189, 147, 249),
                operator: Color::Rgb(248, 248, 242),
                variable: Color::Rgb(248, 248, 242),
                type_name: Color::Rgb(80, 250, 123),
                constant: Color::Rgb(189, 147, 249),
                background: Color::Rgb(40, 42, 54),
                foreground: Color::Rgb(248, 248, 242),
            },
        }
    }

    pub fn get_style_for(&self, token_type: TokenType) -> Style {
        let color = match token_type {
            TokenType::Keyword => self.colors.keyword,
            TokenType::String => self.colors.string,
            TokenType::Comment => self.colors.comment,
            TokenType::Function => self.colors.function,
            TokenType::Number => self.colors.number,
            TokenType::Operator => self.colors.operator,
            TokenType::Variable => self.colors.variable,
            TokenType::TypeName => self.colors.type_name,
            TokenType::Constant => self.colors.constant,
        };

        let modifier = match token_type {
            TokenType::Keyword | TokenType::TypeName => Modifier::BOLD,
            TokenType::Function | TokenType::Constant => Modifier::ITALIC,
            _ => Modifier::empty(),
        };

        Style::default().fg(color).add_modifier(modifier)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Keyword,
    String,
    Comment,
    Function,
    Number,
    Operator,
    Variable,
    TypeName,
    Constant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_theme_name_from_str() {
        assert_eq!(
            SyntaxThemeName::from_str("base16-ocean"),
            Some(SyntaxThemeName::Base16Ocean)
        );
        assert_eq!(
            SyntaxThemeName::from_str("monokai"),
            Some(SyntaxThemeName::Monokai)
        );
        assert_eq!(
            SyntaxThemeName::from_str("nord"),
            Some(SyntaxThemeName::Nord)
        );
        assert_eq!(
            SyntaxThemeName::from_str("custom"),
            Some(SyntaxThemeName::Custom("custom".to_string()))
        );
    }

    #[test]
    fn test_syntax_theme_name_as_str() {
        assert_eq!(SyntaxThemeName::Base16Ocean.as_str(), "base16-ocean.dark");
        assert_eq!(SyntaxThemeName::Monokai.as_str(), "Monokai Extended");
        assert_eq!(SyntaxThemeName::Nord.as_str(), "Nord");
    }

    #[test]
    fn test_syntax_theme_manager_creation() {
        let manager = SyntaxThemeManager::new();
        assert_eq!(manager.current_theme, SyntaxThemeName::Base16Ocean);
        assert!(!manager.list_themes().is_empty());
    }

    #[test]
    fn test_set_current_theme() {
        let mut manager = SyntaxThemeManager::new();
        manager.set_current_theme(SyntaxThemeName::Monokai);
        assert_eq!(manager.current_theme, SyntaxThemeName::Monokai);
        assert_eq!(manager.get_current_theme().name, SyntaxThemeName::Monokai);
    }

    #[test]
    fn test_list_themes() {
        let manager = SyntaxThemeManager::new();
        let themes = manager.list_themes();
        assert!(themes.len() >= 6);
        assert!(themes.contains(&&SyntaxThemeName::Base16Ocean));
        assert!(themes.contains(&&SyntaxThemeName::Monokai));
    }

    #[test]
    fn test_base16_ocean_theme() {
        let theme = SyntaxTheme::base16_ocean();
        assert_eq!(theme.name, SyntaxThemeName::Base16Ocean);
        assert!(theme.is_dark);
    }

    #[test]
    fn test_solarized_light_theme() {
        let theme = SyntaxTheme::solarized_light();
        assert_eq!(theme.name, SyntaxThemeName::SolarizedLight);
        assert!(!theme.is_dark);
    }

    #[test]
    fn test_get_style_for() {
        let theme = SyntaxTheme::base16_ocean();
        let style = theme.get_style_for(TokenType::Keyword);
        assert_eq!(style.fg, Some(theme.colors.keyword));
    }

    #[test]
    fn test_register_custom_theme() {
        let mut manager = SyntaxThemeManager::new();
        let custom_theme = SyntaxTheme {
            name: SyntaxThemeName::Custom("my_theme".to_string()),
            is_dark: true,
            colors: SyntaxColors {
                keyword: Color::Red,
                string: Color::Green,
                comment: Color::Blue,
                function: Color::Yellow,
                number: Color::Cyan,
                operator: Color::Magenta,
                variable: Color::White,
                type_name: Color::LightRed,
                constant: Color::LightGreen,
                background: Color::Black,
                foreground: Color::Gray,
            },
        };

        manager.register_theme(custom_theme.clone());
        let retrieved = manager.get_theme(&custom_theme.name);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, custom_theme.name);
    }
}
