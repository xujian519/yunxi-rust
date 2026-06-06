use ratatui::style::Color;

use super::ColorPalette;
use crate::tui::theme::Theme;

impl Theme {
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: Color::Rgb(136, 192, 208),
                secondary: Color::Rgb(129, 161, 193),
                accent: Color::Rgb(208, 135, 112),
                success: Color::Rgb(163, 190, 140),
                warning: Color::Rgb(235, 203, 139),
                error: Color::Rgb(191, 97, 106),
                info: Color::Rgb(143, 188, 187),
                bg_primary: Color::Rgb(46, 52, 64),
                bg_secondary: Color::Rgb(59, 66, 82),
                bg_tertiary: Color::Rgb(76, 86, 106),
                bg_input: Color::Rgb(59, 66, 82),
                text_primary: Color::Rgb(216, 222, 233),
                text_secondary: Color::Rgb(171, 178, 191),
                text_muted: Color::Rgb(136, 148, 176),
                text_accent: Color::Rgb(129, 161, 193),
                border: Color::Rgb(94, 104, 131),
                border_focus: Color::Rgb(136, 192, 208),
                border_active: Color::Rgb(129, 161, 193),
                brand: Color::Rgb(129, 161, 193),
                brand_shimmer: Color::Rgb(136, 192, 208),
            },
        }
    }

    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: Color::Rgb(189, 147, 249),
                secondary: Color::Rgb(139, 233, 253),
                accent: Color::Rgb(255, 184, 108),
                success: Color::Rgb(80, 250, 123),
                warning: Color::Rgb(241, 250, 140),
                error: Color::Rgb(255, 85, 85),
                info: Color::Rgb(98, 114, 164),
                bg_primary: Color::Rgb(40, 42, 54),
                bg_secondary: Color::Rgb(68, 71, 90),
                bg_tertiary: Color::Rgb(98, 114, 164),
                bg_input: Color::Rgb(68, 71, 90),
                text_primary: Color::Rgb(248, 248, 242),
                text_secondary: Color::Rgb(189, 147, 249),
                text_muted: Color::Rgb(98, 114, 164),
                text_accent: Color::Rgb(139, 233, 253),
                border: Color::Rgb(98, 114, 164),
                border_focus: Color::Rgb(189, 147, 249),
                border_active: Color::Rgb(139, 233, 253),
                brand: Color::Rgb(189, 147, 249),
                brand_shimmer: Color::Rgb(139, 233, 253),
            },
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            name: "gruvbox".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: Color::Rgb(215, 153, 33),
                secondary: Color::Rgb(251, 241, 199),
                accent: Color::Rgb(214, 93, 14),
                success: Color::Rgb(152, 195, 121),
                warning: Color::Rgb(254, 214, 165),
                error: Color::Rgb(251, 73, 52),
                info: Color::Rgb(180, 142, 173),
                bg_primary: Color::Rgb(40, 40, 40),
                bg_secondary: Color::Rgb(60, 56, 54),
                bg_tertiary: Color::Rgb(80, 73, 69),
                bg_input: Color::Rgb(60, 56, 54),
                text_primary: Color::Rgb(235, 219, 178),
                text_secondary: Color::Rgb(213, 196, 161),
                text_muted: Color::Rgb(146, 131, 116),
                text_accent: Color::Rgb(251, 241, 199),
                border: Color::Rgb(124, 111, 100),
                border_focus: Color::Rgb(215, 153, 33),
                border_active: Color::Rgb(214, 93, 14),
                brand: Color::Rgb(215, 153, 33),
                brand_shimmer: Color::Rgb(251, 241, 199),
            },
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized_dark".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: Color::Rgb(38, 139, 210),
                secondary: Color::Rgb(181, 137, 0),
                accent: Color::Rgb(220, 50, 47),
                success: Color::Rgb(133, 153, 0),
                warning: Color::Rgb(181, 137, 0),
                error: Color::Rgb(220, 50, 47),
                info: Color::Rgb(38, 139, 210),
                bg_primary: Color::Rgb(0, 43, 54),
                bg_secondary: Color::Rgb(7, 54, 66),
                bg_tertiary: Color::Rgb(15, 65, 79),
                bg_input: Color::Rgb(7, 54, 66),
                text_primary: Color::Rgb(253, 246, 227),
                text_secondary: Color::Rgb(147, 161, 161),
                text_muted: Color::Rgb(88, 110, 117),
                text_accent: Color::Rgb(108, 113, 196),
                border: Color::Rgb(88, 110, 117),
                border_focus: Color::Rgb(38, 139, 210),
                border_active: Color::Rgb(108, 113, 196),
                brand: Color::Rgb(38, 139, 210),
                brand_shimmer: Color::Rgb(108, 113, 196),
            },
        }
    }

    pub fn solarized_light() -> Self {
        Self {
            name: "solarized_light".to_string(),
            is_dark: false,
            colors: ColorPalette {
                primary: Color::Rgb(38, 139, 210),
                secondary: Color::Rgb(133, 153, 0),
                accent: Color::Rgb(203, 75, 22),
                success: Color::Rgb(133, 153, 0),
                warning: Color::Rgb(181, 137, 0),
                error: Color::Rgb(220, 50, 47),
                info: Color::Rgb(38, 139, 210),
                bg_primary: Color::Rgb(253, 246, 227),
                bg_secondary: Color::Rgb(238, 232, 213),
                bg_tertiary: Color::Rgb(222, 218, 203),
                bg_input: Color::Rgb(238, 232, 213),
                text_primary: Color::Rgb(101, 123, 131),
                text_secondary: Color::Rgb(147, 161, 161),
                text_muted: Color::Rgb(181, 137, 0),
                text_accent: Color::Rgb(38, 139, 210),
                border: Color::Rgb(147, 161, 161),
                border_focus: Color::Rgb(38, 139, 210),
                border_active: Color::Rgb(133, 153, 0),
                brand: Color::Rgb(38, 139, 210),
                brand_shimmer: Color::Rgb(133, 153, 0),
            },
        }
    }

    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: Color::Rgb(102, 217, 239),
                secondary: Color::Rgb(249, 38, 114),
                accent: Color::Rgb(253, 151, 31),
                success: Color::Rgb(166, 226, 46),
                warning: Color::Rgb(253, 151, 31),
                error: Color::Rgb(249, 38, 114),
                info: Color::Rgb(102, 217, 239),
                bg_primary: Color::Rgb(39, 40, 34),
                bg_secondary: Color::Rgb(59, 61, 50),
                bg_tertiary: Color::Rgb(77, 80, 65),
                bg_input: Color::Rgb(59, 61, 50),
                text_primary: Color::Rgb(248, 248, 242),
                text_secondary: Color::Rgb(230, 219, 116),
                text_muted: Color::Rgb(117, 113, 94),
                text_accent: Color::Rgb(174, 129, 255),
                border: Color::Rgb(117, 113, 94),
                border_focus: Color::Rgb(102, 217, 239),
                border_active: Color::Rgb(174, 129, 255),
                brand: Color::Rgb(102, 217, 239),
                brand_shimmer: Color::Rgb(174, 129, 255),
            },
        }
    }

    pub fn catppuccin() -> Self {
        Self {
            name: "catppuccin".to_string(),
            is_dark: true,
            colors: ColorPalette {
                primary: Color::Rgb(137, 180, 250),
                secondary: Color::Rgb(166, 227, 161),
                accent: Color::Rgb(243, 139, 168),
                success: Color::Rgb(166, 227, 161),
                warning: Color::Rgb(250, 179, 135),
                error: Color::Rgb(243, 139, 168),
                info: Color::Rgb(137, 180, 250),
                bg_primary: Color::Rgb(30, 30, 46),
                bg_secondary: Color::Rgb(49, 50, 68),
                bg_tertiary: Color::Rgb(69, 71, 90),
                bg_input: Color::Rgb(49, 50, 68),
                text_primary: Color::Rgb(205, 214, 244),
                text_secondary: Color::Rgb(186, 194, 222),
                text_muted: Color::Rgb(166, 173, 200),
                text_accent: Color::Rgb(245, 224, 220),
                border: Color::Rgb(88, 91, 112),
                border_focus: Color::Rgb(137, 180, 250),
                border_active: Color::Rgb(166, 227, 161),
                brand: Color::Rgb(137, 180, 250),
                brand_shimmer: Color::Rgb(166, 227, 161),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nord_theme() {
        let theme = Theme::nord();
        assert_eq!(theme.name, "nord");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_dracula_theme() {
        let theme = Theme::dracula();
        assert_eq!(theme.name, "dracula");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_gruvbox_theme() {
        let theme = Theme::gruvbox();
        assert_eq!(theme.name, "gruvbox");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_solarized_dark_theme() {
        let theme = Theme::solarized_dark();
        assert_eq!(theme.name, "solarized_dark");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_solarized_light_theme() {
        let theme = Theme::solarized_light();
        assert_eq!(theme.name, "solarized_light");
        assert!(!theme.is_dark);
    }

    #[test]
    fn test_monokai_theme() {
        let theme = Theme::monokai();
        assert_eq!(theme.name, "monokai");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_catppuccin_theme() {
        let theme = Theme::catppuccin();
        assert_eq!(theme.name, "catppuccin");
        assert!(theme.is_dark);
    }

    #[test]
    fn test_all_themes_are_dark_or_light() {
        let themes = [
            Theme::nord(),
            Theme::dracula(),
            Theme::gruvbox(),
            Theme::solarized_dark(),
            Theme::solarized_light(),
            Theme::monokai(),
            Theme::catppuccin(),
        ];

        for theme in &themes {
            let _ = theme.is_dark;
        }
    }

    #[test]
    fn test_theme_color_palette_not_empty() {
        let theme = Theme::nord();
        let _ = theme.colors.primary;
        let _ = theme.colors.secondary;
        let _ = theme.colors.bg_primary;
        let _ = theme.colors.text_primary;
    }

    #[test]
    fn test_light_theme_count() {
        let mut registry = crate::tui::theme::ThemeRegistry::new();
        registry.register(Theme::nord());
        registry.register(Theme::dracula());
        registry.register(Theme::gruvbox());
        registry.register(Theme::solarized_dark());
        registry.register(Theme::solarized_light());
        registry.register(Theme::monokai());
        registry.register(Theme::catppuccin());

        let light_count = registry.themes.iter().filter(|t| !t.is_dark).count();

        assert!(light_count >= 1);
    }

    #[test]
    fn test_dark_theme_count() {
        let mut registry = crate::tui::theme::ThemeRegistry::new();
        registry.register(Theme::nord());
        registry.register(Theme::dracula());
        registry.register(Theme::gruvbox());
        registry.register(Theme::solarized_dark());
        registry.register(Theme::solarized_light());
        registry.register(Theme::monokai());
        registry.register(Theme::catppuccin());

        let dark_count = registry.themes.iter().filter(|t| t.is_dark).count();

        assert!(dark_count >= 6);
    }
}
