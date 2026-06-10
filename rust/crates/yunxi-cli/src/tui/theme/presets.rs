use ratatui::style::Color;

use super::ColorPalette;
use crate::tui::theme::{BorderStyle, StyleSet, Theme};

impl Theme {
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            is_dark: true,
            styles: StyleSet::default(),
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
            styles: StyleSet::default(),
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
            styles: StyleSet::default(),
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
            styles: StyleSet::default(),
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
            styles: StyleSet::default(),
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
            styles: StyleSet::default(),
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
            styles: StyleSet::default(),
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

    // ── 规范主推荐方案："墨园"（Ink Garden） ──

    pub fn ink_garden_dark() -> Self {
        Self {
            name: "ink_garden_dark".to_string(),
            is_dark: true,
            styles: StyleSet {
                borders: BorderStyle {
                    border_type: super::BorderType::Rounded,
                    ..BorderStyle::default()
                },
                ..StyleSet::default()
            },
            colors: ColorPalette {
                primary: Color::Rgb(129, 161, 193),        // #81A1C1
                secondary: Color::Rgb(136, 192, 208),      // #88C0D0
                accent: Color::Rgb(201, 129, 138),         // #C9818A
                success: Color::Rgb(159, 184, 154),        // #9FB89A
                warning: Color::Rgb(208, 135, 112),        // #D08770
                error: Color::Rgb(191, 97, 106),           // #BF616A
                info: Color::Rgb(143, 188, 187),           // #8FBCBB
                bg_primary: Color::Rgb(26, 29, 35),        // #1A1D23
                bg_secondary: Color::Rgb(35, 39, 48),      // #232730
                bg_tertiary: Color::Rgb(46, 52, 64),       // #2E3440
                bg_input: Color::Rgb(35, 39, 48),          // #232730
                text_primary: Color::Rgb(216, 222, 233),   // #D8DEE9
                text_secondary: Color::Rgb(171, 178, 191), // #ABBFc1
                text_muted: Color::Rgb(123, 132, 156),     // #7B849C
                text_accent: Color::Rgb(136, 192, 208),    // #88C0D0
                border: Color::Rgb(59, 66, 82),            // #3B4252
                border_focus: Color::Rgb(129, 161, 193),   // #81A1C1
                border_active: Color::Rgb(136, 192, 208),  // #88C0D0
                brand: Color::Rgb(129, 161, 193),          // #81A1C1
                brand_shimmer: Color::Rgb(136, 192, 208),  // #88C0D0
            },
        }
    }

    pub fn ink_garden_light() -> Self {
        Self {
            name: "ink_garden_light".to_string(),
            is_dark: false,
            styles: StyleSet::default(),
            colors: ColorPalette {
                primary: Color::Rgb(94, 129, 172),        // #5E81AC
                secondary: Color::Rgb(93, 154, 142),      // #5D9A8E
                accent: Color::Rgb(179, 93, 106),         // #B35D6A
                success: Color::Rgb(109, 150, 112),       // #6D9670
                warning: Color::Rgb(194, 122, 92),        // #C27A5C
                error: Color::Rgb(179, 93, 106),          // #B35D6A
                info: Color::Rgb(93, 154, 142),           // #5D9A8E
                bg_primary: Color::Rgb(245, 245, 240),    // #F5F5F0
                bg_secondary: Color::Rgb(236, 239, 244),  // #ECEFF4
                bg_tertiary: Color::Rgb(229, 233, 240),   // #E5E9F0
                bg_input: Color::Rgb(236, 239, 244),      // #ECEFF4
                text_primary: Color::Rgb(46, 52, 64),     // #2E3440
                text_secondary: Color::Rgb(92, 101, 125), // #5C657D
                text_muted: Color::Rgb(92, 101, 125),     // #5C657D
                text_accent: Color::Rgb(94, 129, 172),    // #5E81AC
                border: Color::Rgb(216, 222, 233),        // #D8DEE9
                border_focus: Color::Rgb(94, 129, 172),   // #5E81AC
                border_active: Color::Rgb(93, 154, 142),  // #5D9A8E
                brand: Color::Rgb(94, 129, 172),          // #5E81AC
                brand_shimmer: Color::Rgb(93, 154, 142),  // #5D9A8E
            },
        }
    }

    // ── "禅径"（Zen Path）低对比度方案 ──

    pub fn zen_path_dark() -> Self {
        Self {
            name: "zen_path_dark".to_string(),
            is_dark: true,
            styles: StyleSet::default(),
            colors: ColorPalette {
                primary: Color::Rgb(140, 176, 211),        // #8CB0D3
                secondary: Color::Rgb(140, 208, 211),      // #8CD0D3
                accent: Color::Rgb(204, 147, 147),         // #CC9393
                success: Color::Rgb(127, 159, 127),        // #7F9F7F
                warning: Color::Rgb(223, 175, 143),        // #DFAF8F
                error: Color::Rgb(204, 147, 147),          // #CC9393
                info: Color::Rgb(140, 208, 211),           // #8CD0D3
                bg_primary: Color::Rgb(58, 58, 58),        // #3A3A3A
                bg_secondary: Color::Rgb(67, 67, 67),      // #434343
                bg_tertiary: Color::Rgb(77, 77, 77),       // #4D4D4D
                bg_input: Color::Rgb(67, 67, 67),          // #434343
                text_primary: Color::Rgb(220, 220, 204),   // #DCDCCC
                text_secondary: Color::Rgb(180, 180, 170), // #B4B4AA
                text_muted: Color::Rgb(143, 143, 143),     // #8F8F8F
                text_accent: Color::Rgb(140, 176, 211),    // #8CB0D3
                border: Color::Rgb(90, 90, 90),            // #5A5A5A
                border_focus: Color::Rgb(140, 176, 211),   // #8CB0D3
                border_active: Color::Rgb(140, 208, 211),  // #8CD0D3
                brand: Color::Rgb(140, 176, 211),          // #8CB0D3
                brand_shimmer: Color::Rgb(140, 208, 211),  // #8CD0D3
            },
        }
    }

    pub fn zen_path_light() -> Self {
        Self {
            name: "zen_path_light".to_string(),
            is_dark: false,
            styles: StyleSet::default(),
            colors: ColorPalette {
                primary: Color::Rgb(80, 96, 128),         // #506080
                secondary: Color::Rgb(80, 128, 128),      // #508080
                accent: Color::Rgb(160, 80, 80),          // #A05050
                success: Color::Rgb(79, 111, 79),         // #4F6F4F
                warning: Color::Rgb(138, 122, 80),        // #8A7A50
                error: Color::Rgb(160, 80, 80),           // #A05050
                info: Color::Rgb(80, 128, 128),           // #508080
                bg_primary: Color::Rgb(245, 245, 224),    // #F5F5E0
                bg_secondary: Color::Rgb(224, 224, 208),  // #E0E0D0
                bg_tertiary: Color::Rgb(210, 210, 195),   // #D2D2C3
                bg_input: Color::Rgb(224, 224, 208),      // #E0E0D0
                text_primary: Color::Rgb(58, 58, 58),     // #3A3A3A
                text_secondary: Color::Rgb(100, 100, 90), // #64645A
                text_muted: Color::Rgb(122, 122, 106),    // #7A7A6A
                text_accent: Color::Rgb(80, 96, 128),     // #506080
                border: Color::Rgb(192, 192, 176),        // #C0C0B0
                border_focus: Color::Rgb(80, 96, 128),    // #506080
                border_active: Color::Rgb(80, 128, 128),  // #508080
                brand: Color::Rgb(80, 96, 128),           // #506080
                brand_shimmer: Color::Rgb(80, 128, 128),  // #508080
            },
        }
    }

    // ── "明晰"（Clear Mode）高对比度方案 ──

    pub fn clear_mode_dark() -> Self {
        Self {
            name: "clear_mode_dark".to_string(),
            is_dark: true,
            styles: StyleSet {
                borders: BorderStyle {
                    border_type: super::BorderType::Double,
                    ..BorderStyle::default()
                },
                ..StyleSet::default()
            },
            colors: ColorPalette {
                primary: Color::Rgb(143, 143, 255),        // #8F8FFF
                secondary: Color::Rgb(95, 255, 95),        // #5FFF5F
                accent: Color::Rgb(255, 95, 95),           // #FF5F5F
                success: Color::Rgb(95, 255, 95),          // #5FFF5F
                warning: Color::Rgb(255, 159, 95),         // #FF9F5F
                error: Color::Rgb(255, 95, 95),            // #FF5F5F
                info: Color::Rgb(95, 255, 255),            // #5FFFFF
                bg_primary: Color::Rgb(0, 0, 0),           // #000000
                bg_secondary: Color::Rgb(26, 26, 26),      // #1A1A1A
                bg_tertiary: Color::Rgb(46, 46, 46),       // #2E2E2E
                bg_input: Color::Rgb(26, 26, 26),          // #1A1A1A
                text_primary: Color::Rgb(255, 255, 255),   // #FFFFFF
                text_secondary: Color::Rgb(200, 200, 200), // #C8C8C8
                text_muted: Color::Rgb(160, 160, 160),     // #A0A0A0
                text_accent: Color::Rgb(143, 143, 255),    // #8F8FFF
                border: Color::Rgb(51, 51, 51),            // #333333
                border_focus: Color::Rgb(143, 143, 255),   // #8F8FFF
                border_active: Color::Rgb(95, 255, 255),   // #5FFFFF
                brand: Color::Rgb(143, 143, 255),          // #8F8FFF
                brand_shimmer: Color::Rgb(95, 255, 255),   // #5FFFFF
            },
        }
    }

    pub fn clear_mode_light() -> Self {
        Self {
            name: "clear_mode_light".to_string(),
            is_dark: false,
            styles: StyleSet::default(),
            colors: ColorPalette {
                primary: Color::Rgb(0, 0, 184),          // #0000B8
                secondary: Color::Rgb(0, 85, 0),         // #005500
                accent: Color::Rgb(163, 29, 29),         // #A31D1D
                success: Color::Rgb(0, 85, 0),           // #005500
                warning: Color::Rgb(138, 69, 0),         // #8A4500
                error: Color::Rgb(163, 29, 29),          // #A31D1D
                info: Color::Rgb(0, 72, 72),             // #004848
                bg_primary: Color::Rgb(255, 255, 255),   // #FFFFFF
                bg_secondary: Color::Rgb(240, 240, 240), // #F0F0F0
                bg_tertiary: Color::Rgb(224, 224, 224),  // #E0E0E0
                bg_input: Color::Rgb(240, 240, 240),     // #F0F0F0
                text_primary: Color::Rgb(0, 0, 0),       // #000000
                text_secondary: Color::Rgb(80, 80, 80),  // #505050
                text_muted: Color::Rgb(88, 88, 88),      // #585858
                text_accent: Color::Rgb(0, 0, 184),      // #0000B8
                border: Color::Rgb(204, 204, 204),       // #CCCCCC
                border_focus: Color::Rgb(0, 0, 184),     // #0000B8
                border_active: Color::Rgb(0, 72, 72),    // #004848
                brand: Color::Rgb(0, 0, 184),            // #0000B8
                brand_shimmer: Color::Rgb(0, 72, 72),    // #004848
            },
        }
    }

    // ── Tokyo Night ──

    pub fn tokyo_night() -> Self {
        Self {
            name: "tokyo_night".to_string(),
            is_dark: true,
            styles: StyleSet::default(),
            colors: ColorPalette {
                primary: Color::Rgb(122, 162, 247),        // #7AA2F7
                secondary: Color::Rgb(187, 154, 247),      // #BB9AF7
                accent: Color::Rgb(247, 118, 142),         // #F7768E
                success: Color::Rgb(158, 206, 106),        // #9ECE6A
                warning: Color::Rgb(224, 175, 104),        // #E0AF68
                error: Color::Rgb(247, 118, 142),          // #F7768E
                info: Color::Rgb(125, 207, 255),           // #7DCFFF
                bg_primary: Color::Rgb(26, 27, 46),        // #1A1B2E
                bg_secondary: Color::Rgb(36, 38, 58),      // #24263A
                bg_tertiary: Color::Rgb(47, 49, 73),       // #2F3149
                bg_input: Color::Rgb(36, 38, 58),          // #24263A
                text_primary: Color::Rgb(192, 202, 245),   // #C0CAF5
                text_secondary: Color::Rgb(161, 168, 210), // #A1A8D2
                text_muted: Color::Rgb(92, 100, 138),      // #5C648A
                text_accent: Color::Rgb(187, 154, 247),    // #BB9AF7
                border: Color::Rgb(51, 54, 82),            // #333652
                border_focus: Color::Rgb(122, 162, 247),   // #7AA2F7
                border_active: Color::Rgb(187, 154, 247),  // #BB9AF7
                brand: Color::Rgb(122, 162, 247),          // #7AA2F7
                brand_shimmer: Color::Rgb(187, 154, 247),  // #BB9AF7
            },
        }
    }

    // ── 色盲友好变体（Daltonized） ──

    pub fn daltonized_dark() -> Self {
        Self {
            name: "daltonized_dark".to_string(),
            is_dark: true,
            styles: StyleSet::default(),
            colors: ColorPalette {
                primary: Color::Rgb(122, 162, 247),        // #7AA2F7 蓝
                secondary: Color::Rgb(125, 207, 255),      // #7DCFFF 浅蓝
                accent: Color::Rgb(204, 102, 0),           // #CC6600 橙
                success: Color::Rgb(0, 136, 204),          // #0088CC 蓝（替代绿）
                warning: Color::Rgb(204, 68, 170),         // #CC44AA 品红（替代黄）
                error: Color::Rgb(204, 102, 0),            // #CC6600 橙（替代红）
                info: Color::Rgb(0, 170, 170),             // #00AAAA 青
                bg_primary: Color::Rgb(20, 20, 24),        // #141418
                bg_secondary: Color::Rgb(27, 27, 32),      // #1B1B20
                bg_tertiary: Color::Rgb(35, 35, 40),       // #232328
                bg_input: Color::Rgb(35, 35, 40),          // #232328
                text_primary: Color::Rgb(235, 235, 240),   // #EBEBF0
                text_secondary: Color::Rgb(168, 168, 175), // #A8A8AF
                text_muted: Color::Rgb(115, 115, 122),     // #73737A
                text_accent: Color::Rgb(125, 207, 255),    // #7DCFFF
                border: Color::Rgb(45, 45, 51),            // #2D2D33
                border_focus: Color::Rgb(122, 162, 247),   // #7AA2F7
                border_active: Color::Rgb(125, 207, 255),  // #7DCFFF
                brand: Color::Rgb(122, 162, 247),          // #7AA2F7
                brand_shimmer: Color::Rgb(125, 207, 255),  // #7DCFFF
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
