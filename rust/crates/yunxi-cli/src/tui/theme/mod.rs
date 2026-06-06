use ratatui::style::Color;

pub mod manager;
pub mod presets;

pub use manager::ThemeManager;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub colors: ColorPalette,
}

#[derive(Debug, Clone)]
pub struct ColorPalette {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    pub bg_input: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_accent: Color,
    pub border: Color,
    pub border_focus: Color,
    pub border_active: Color,
    pub brand: Color,
    pub brand_shimmer: Color,
}

impl Theme {
    pub fn default_dark() -> Self {
        Self {
            name: "default_dark".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: Color::Rgb(120, 175, 240),
                secondary: Color::Rgb(180, 160, 220),
                accent: Color::Rgb(230, 185, 105),
                success: Color::Rgb(130, 200, 160),
                warning: Color::Rgb(230, 185, 105),
                error: Color::Rgb(230, 130, 125),
                info: Color::Rgb(120, 175, 240),
                bg_primary: Color::Rgb(20, 20, 24),
                bg_secondary: Color::Rgb(27, 27, 32),
                bg_tertiary: Color::Rgb(35, 35, 40),
                bg_input: Color::Rgb(35, 35, 40),
                text_primary: Color::Rgb(235, 235, 240),
                text_secondary: Color::Rgb(168, 168, 175),
                text_muted: Color::Rgb(115, 115, 122),
                text_accent: Color::Rgb(180, 160, 220),
                border: Color::Rgb(45, 45, 51),
                border_focus: Color::Rgb(80, 80, 90),
                border_active: Color::Rgb(120, 175, 240),
                brand: Color::Rgb(100, 150, 215),
                brand_shimmer: Color::Rgb(130, 178, 235),
            },
        }
    }

    pub fn default_light() -> Self {
        Self {
            name: "default_light".to_string(),
            is_dark: false,
            colors: ColorPalette {
                primary: Color::Rgb(55, 66, 81),
                secondary: Color::Rgb(162, 213, 244),
                accent: Color::Rgb(208, 135, 112),
                success: Color::Rgb(152, 195, 121),
                warning: Color::Rgb(229, 192, 123),
                error: Color::Rgb(224, 108, 117),
                info: Color::Rgb(86, 182, 194),
                bg_primary: Color::Rgb(255, 255, 255),
                bg_secondary: Color::Rgb(248, 248, 248),
                bg_tertiary: Color::Rgb(240, 240, 240),
                bg_input: Color::Rgb(250, 250, 250),
                text_primary: Color::Rgb(47, 47, 47),
                text_secondary: Color::Rgb(138, 138, 138),
                text_muted: Color::Rgb(165, 165, 165),
                text_accent: Color::Rgb(59, 130, 246),
                border: Color::Rgb(224, 224, 224),
                border_focus: Color::Rgb(100, 149, 237),
                border_active: Color::Rgb(59, 130, 246),
                brand: Color::Rgb(59, 130, 246),
                brand_shimmer: Color::Rgb(100, 149, 237),
            },
        }
    }

    pub fn preview(&self) -> String {
        let c = &self.colors;
        let colors = [
            ("主色", c.primary),
            ("次色", c.secondary),
            ("强调", c.accent),
            ("成功", c.success),
            ("警告", c.warning),
            ("错误", c.error),
            ("信息", c.info),
            ("品牌", c.brand),
        ];

        let mut preview = format!("{} ", self.name);
        for (label, color) in colors {
            let rgb = match color {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => continue,
            };
            preview.push_str(&format!(
                "\x1b[48;2;{};{};{}m {} \x1b[0m ",
                rgb.0, rgb.1, rgb.2, label
            ));
        }
        preview
    }
}

pub struct ThemeRegistry {
    themes: Vec<Theme>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self { themes: Vec::new() };
        registry.register(Theme::default_dark());
        registry.register(Theme::default_light());
        registry
    }

    pub fn register(&mut self, theme: Theme) {
        self.themes.push(theme);
    }

    pub fn get(&self, name: &str) -> Theme {
        self.themes
            .iter()
            .find(|t| t.name == name)
            .cloned()
            .unwrap_or_else(Theme::default_dark)
    }

    pub fn list_names(&self) -> Vec<String> {
        self.themes.iter().map(|t| t.name.clone()).collect()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dark_theme() {
        let theme = Theme::default_dark();
        assert_eq!(theme.name, "default_dark");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_default_light_theme() {
        let theme = Theme::default_light();
        assert_eq!(theme.name, "default_light");
        assert!(!theme.is_dark);
    }

    #[test]
    fn test_theme_registry_get() {
        let registry = ThemeRegistry::new();
        let dark = registry.get("default_dark");
        assert_eq!(dark.name, "default_dark");

        let unknown = registry.get("unknown_theme");
        assert_eq!(unknown.name, "default_dark");
    }

    #[test]
    fn test_theme_registry_list() {
        let registry = ThemeRegistry::new();
        let names = registry.list_names();
        assert!(names.contains(&"default_dark".to_string()));
        assert!(names.contains(&"default_light".to_string()));
    }

    #[test]
    fn test_theme_preview() {
        let theme = Theme::default_dark();
        let preview = theme.preview();
        assert!(preview.contains("default_dark"));
        assert!(preview.contains("\x1b[48;2;"));
        assert!(preview.contains("主色"));
        assert!(preview.contains("次色"));
    }
}
