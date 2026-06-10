use ratatui::style::Color;

pub mod manager;
pub mod presets;

pub use manager::ThemeManager;

/// 边框样式配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderStyle {
    /// 边框类型（None / Plain / Rounded / Double / Thick 等）。
    pub border_type: BorderType,
    /// 是否显示标题栏边框。
    pub title_bar_border: bool,
    /// 是否显示面板边框。
    pub panel_border: bool,
    /// 是否显示输入框边框。
    pub input_border: bool,
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self {
            border_type: BorderType::Plain,
            title_bar_border: false,
            panel_border: true,
            input_border: true,
        }
    }
}

/// 边框类型枚举（对齐 ratatui BorderType）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    Plain,
    Rounded,
    Double,
    Thick,
    None,
}

impl BorderType {
    pub fn to_ratatui(self) -> ratatui::widgets::BorderType {
        match self {
            Self::Plain => ratatui::widgets::BorderType::Plain,
            Self::Rounded => ratatui::widgets::BorderType::Rounded,
            Self::Double => ratatui::widgets::BorderType::Double,
            Self::Thick => ratatui::widgets::BorderType::Thick,
            Self::None => ratatui::widgets::BorderType::Plain,
        }
    }
}

/// 动画配置。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationConfig {
    /// 是否启用 spinner 动画。
    pub spinner_enabled: bool,
    /// spinner 帧间隔（毫秒）。
    pub spinner_interval_ms: u16,
    /// 是否启用 brand shimmer 动画。
    pub shimmer_enabled: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            spinner_enabled: true,
            spinner_interval_ms: 80,
            shimmer_enabled: false,
        }
    }
}

/// 主题样式集（非颜色配置）。
#[derive(Debug, Clone, Default)]
pub struct StyleSet {
    pub borders: BorderStyle,
    pub animations: AnimationConfig,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub colors: ColorPalette,
    pub styles: StyleSet,
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
            styles: StyleSet::default(),
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
            styles: StyleSet::default(),
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
        // 规范标准主题
        registry.register(Theme::ink_garden_dark()); // 默认暗色
        registry.register(Theme::ink_garden_light()); // 默认亮色
        registry.register(Theme::zen_path_dark());
        registry.register(Theme::zen_path_light());
        registry.register(Theme::clear_mode_dark());
        registry.register(Theme::clear_mode_light());
        // 社区流行主题
        registry.register(Theme::nord());
        registry.register(Theme::dracula());
        registry.register(Theme::gruvbox());
        registry.register(Theme::catppuccin());
        registry.register(Theme::tokyo_night());
        // 无障碍主题
        registry.register(Theme::daltonized_dark());
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

    /// 列出所有暗色主题
    pub fn dark_themes(&self) -> Vec<&Theme> {
        self.themes.iter().filter(|t| t.is_dark).collect()
    }

    /// 列出所有亮色主题
    pub fn light_themes(&self) -> Vec<&Theme> {
        self.themes.iter().filter(|t| !t.is_dark).collect()
    }

    /// 列出高对比度主题（明晰方案）
    pub fn high_contrast_themes(&self) -> Vec<&Theme> {
        self.themes
            .iter()
            .filter(|t| t.name.starts_with("clear_mode"))
            .collect()
    }

    /// 列出色盲友好主题
    pub fn colorblind_themes(&self) -> Vec<&Theme> {
        self.themes
            .iter()
            .filter(|t| t.name.starts_with("daltonized"))
            .collect()
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
        assert!(theme.is_dark);
    }

    #[test]
    fn test_default_light_theme() {
        let theme = Theme::default_light();
        assert!(!theme.is_dark);
    }

    #[test]
    fn test_theme_registry_get() {
        let registry = ThemeRegistry::new();
        let dark = registry.get("ink_garden_dark");
        assert_eq!(dark.name, "ink_garden_dark");

        let unknown = registry.get("unknown_theme");
        assert_eq!(unknown.name, "default_dark");
    }

    #[test]
    fn test_theme_registry_list() {
        let registry = ThemeRegistry::new();
        let names = registry.list_names();
        // 规范三套方案
        assert!(names.contains(&"ink_garden_dark".to_string()));
        assert!(names.contains(&"ink_garden_light".to_string()));
        assert!(names.contains(&"zen_path_dark".to_string()));
        assert!(names.contains(&"clear_mode_dark".to_string()));
        // 社区主题
        assert!(names.contains(&"tokyo_night".to_string()));
        // 无障碍
        assert!(names.contains(&"daltonized_dark".to_string()));
    }

    #[test]
    fn test_theme_preview() {
        let theme = Theme::ink_garden_dark();
        let preview = theme.preview();
        assert!(preview.contains("ink_garden_dark"));
        assert!(preview.contains("\x1b[48;2;"));
    }

    #[test]
    fn test_registry_has_13_themes() {
        let registry = ThemeRegistry::new();
        assert!(
            registry.list_names().len() >= 12,
            "Expected >= 12 themes, got {}",
            registry.list_names().len()
        );
    }

    #[test]
    fn test_dark_themes_filter() {
        let registry = ThemeRegistry::new();
        let dark = registry.dark_themes();
        assert!(dark.len() >= 7);
        assert!(dark.iter().all(|t| t.is_dark));
    }

    #[test]
    fn test_light_themes_filter() {
        let registry = ThemeRegistry::new();
        let light = registry.light_themes();
        assert!(light.len() >= 3);
        assert!(light.iter().all(|t| !t.is_dark));
    }

    #[test]
    fn test_high_contrast_themes() {
        let registry = ThemeRegistry::new();
        let hc = registry.high_contrast_themes();
        assert_eq!(hc.len(), 2); // clear_mode_dark + clear_mode_light
    }

    #[test]
    fn test_colorblind_themes() {
        let registry = ThemeRegistry::new();
        let cb = registry.colorblind_themes();
        assert!(!cb.is_empty());
    }

    #[test]
    fn test_all_themes_contrast_compliance() {
        use crate::tui::color::contrast::meets_wcag_aa;
        let registry = ThemeRegistry::new();
        for theme in &registry.themes {
            let text = theme.colors.text_primary;
            let bg = theme.colors.bg_primary;
            if let (Color::Rgb(tr, tg, tb), Color::Rgb(br, bg_, bb)) = (text, bg) {
                let fg_rgb = (tr, tg, tb);
                let bg_rgb = (br, bg_, bb);
                assert!(
                    meets_wcag_aa(fg_rgb, bg_rgb),
                    "{}: text/bg contrast below AA (fg={:?}, bg={:?})",
                    theme.name,
                    fg_rgb,
                    bg_rgb,
                );
            }
        }
    }
}
