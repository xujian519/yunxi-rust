use ratatui::style::Color;

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
                primary: Color::Rgb(139, 176, 240),
                secondary: Color::Rgb(200, 182, 255),
                accent: Color::Rgb(232, 200, 124),
                success: Color::Rgb(123, 200, 156),
                warning: Color::Rgb(232, 200, 124),
                error: Color::Rgb(232, 132, 124),
                info: Color::Rgb(139, 176, 240),
                bg_primary: Color::Rgb(13, 13, 18),
                bg_secondary: Color::Rgb(22, 22, 30),
                bg_tertiary: Color::Rgb(30, 30, 46),
                bg_input: Color::Rgb(26, 35, 50),
                text_primary: Color::Rgb(232, 232, 237),
                text_secondary: Color::Rgb(160, 160, 176),
                text_muted: Color::Rgb(106, 106, 128),
                text_accent: Color::Rgb(200, 182, 255),
                border: Color::Rgb(42, 42, 58),
                border_focus: Color::Rgb(74, 74, 106),
                border_active: Color::Rgb(139, 176, 240),
                brand: Color::Rgb(107, 141, 214),
                brand_shimmer: Color::Rgb(139, 176, 240),
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
}

pub struct ThemeRegistry {
    themes: Vec<Theme>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            themes: Vec::new(),
        };
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
            .unwrap_or_else(|| Theme::default_dark())
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
