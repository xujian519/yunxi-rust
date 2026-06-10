//! 色阶变体系统（Color Scale）。
//!
//! 每个语义色生成 5 个变体，用于组件状态视觉反馈：
//! - lighten_2: 悬浮/高亮状态
//! - lighten_1: 焦点状态
//! - base: 默认
//! - darken_1: 激活/按下状态
//! - darken_2: 阴影/深度

use ratatui::style::Color;

use super::oklch;

/// 五级色阶变体
#[derive(Debug, Clone)]
pub struct ColorScale {
    pub lighten_2: Color,
    pub lighten_1: Color,
    pub base: Color,
    pub darken_1: Color,
    pub darken_2: Color,
}

impl ColorScale {
    /// 从 OKLCH 坐标生成色阶。
    ///
    /// `is_dark` 为 true 时自动降低饱和度 15-30%（暗色模式补偿）。
    pub fn from_oklch(l: f64, c: f64, h: f64, is_dark: bool) -> Self {
        let saturation_factor = if is_dark { 0.80 } else { 1.0 };
        let adj_c = c * saturation_factor;

        let base = oklch::oklch_to_rgb(l, adj_c, h);
        let lighten_1 = oklch::oklch_to_rgb((l + 0.08).min(1.0), adj_c * 0.9, h);
        let lighten_2 = oklch::oklch_to_rgb((l + 0.14).min(1.0), adj_c * 0.8, h);
        let darken_1 = oklch::oklch_to_rgb((l - 0.08).max(0.0), adj_c * 1.05, h);
        let darken_2 = oklch::oklch_to_rgb((l - 0.14).max(0.0), adj_c * 1.1, h);

        Self {
            lighten_2: Color::Rgb(lighten_2.0, lighten_2.1, lighten_2.2),
            lighten_1: Color::Rgb(lighten_1.0, lighten_1.1, lighten_1.2),
            base: Color::Rgb(base.0, base.1, base.2),
            darken_1: Color::Rgb(darken_1.0, darken_1.1, darken_1.2),
            darken_2: Color::Rgb(darken_2.0, darken_2.1, darken_2.2),
        }
    }

    /// 从 RGB 值生成色阶（fallback，通过 OKLCH 中间转换）。
    pub fn from_rgb(r: u8, g: u8, b: u8, is_dark: bool) -> Self {
        let (l, c, h) = oklch::rgb_to_oklch(r, g, b);
        // 如果色度接近 0（灰色），直接用亮度操作
        if c < 0.01 {
            let lf = l * 100.0;
            return Self::gray_scale(lf, is_dark);
        }
        Self::from_oklch(l, c, h, is_dark)
    }

    /// 灰色色阶（无色度，仅亮度变化）
    fn gray_scale(l_percent: f64, _is_dark: bool) -> Self {
        let base = l_percent_to_u8(l_percent);
        let l1 = l_percent_to_u8(l_percent + 8.0);
        let l2 = l_percent_to_u8(l_percent + 14.0);
        let d1 = l_percent_to_u8(l_percent - 8.0);
        let d2 = l_percent_to_u8(l_percent - 14.0);

        Self {
            lighten_2: Color::Rgb(l2, l2, l2),
            lighten_1: Color::Rgb(l1, l1, l1),
            base: Color::Rgb(base, base, base),
            darken_1: Color::Rgb(d1, d1, d1),
            darken_2: Color::Rgb(d2, d2, d2),
        }
    }

    /// 从 ratatui Color 生成色阶
    pub fn from_color(color: Color, is_dark: bool) -> Self {
        match color {
            Color::Rgb(r, g, b) => Self::from_rgb(r, g, b, is_dark),
            _ => Self::gray_scale(50.0, is_dark), // fallback
        }
    }
}

/// 所有语义色的色阶集合
#[derive(Debug, Clone)]
pub struct ScaledPalette {
    pub primary: ColorScale,
    pub secondary: ColorScale,
    pub accent: ColorScale,
    pub success: ColorScale,
    pub warning: ColorScale,
    pub error: ColorScale,
    pub info: ColorScale,
    pub brand: ColorScale,
}

fn l_percent_to_u8(l: f64) -> u8 {
    (l.clamp(0.0, 100.0) * 2.55).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_from_rgb_roundtrip() {
        let scale = ColorScale::from_rgb(100, 150, 200, true);
        // base 应该接近原色（OKLCH 往返允许 ±15 误差，暗色模式饱和度补偿引入偏差）
        if let Color::Rgb(r, g, b) = scale.base {
            assert!(
                (r as i16 - 100).unsigned_abs() <= 15,
                "R={r}, expected ~100"
            );
            assert!(
                (g as i16 - 150).unsigned_abs() <= 15,
                "G={g}, expected ~150"
            );
            assert!(
                (b as i16 - 200).unsigned_abs() <= 15,
                "B={b}, expected ~200"
            );
        }
    }

    #[test]
    fn lighten_is_brighter_than_base() {
        let scale = ColorScale::from_rgb(100, 100, 100, true);
        let base_lum = rgb_luminance(scale.base);
        let l1_lum = rgb_luminance(scale.lighten_1);
        let l2_lum = rgb_luminance(scale.lighten_2);
        assert!(l1_lum > base_lum, "lighten_1 should be brighter than base");
        assert!(
            l2_lum > l1_lum,
            "lighten_2 should be brighter than lighten_1"
        );
    }

    #[test]
    fn darken_is_darker_than_base() {
        let scale = ColorScale::from_rgb(150, 150, 150, true);
        let base_lum = rgb_luminance(scale.base);
        let d1_lum = rgb_luminance(scale.darken_1);
        let d2_lum = rgb_luminance(scale.darken_2);
        assert!(d1_lum < base_lum, "darken_1 should be darker than base");
        assert!(d2_lum < d1_lum, "darken_2 should be darker than darken_1");
    }

    #[test]
    fn gray_scale_generation() {
        let scale = ColorScale::from_rgb(128, 128, 128, true);
        if let Color::Rgb(r, g, b) = scale.base {
            // 灰色的 RGB 三分量应近似相等（允许 ±2 舍入误差）
            assert!((r as i16 - g as i16).unsigned_abs() <= 2);
            assert!((g as i16 - b as i16).unsigned_abs() <= 2);
        }
    }

    #[test]
    fn dark_mode_reduces_saturation() {
        let dark = ColorScale::from_oklch(0.6, 0.15, 280.0, true);
        let light = ColorScale::from_oklch(0.6, 0.15, 280.0, false);
        // 暗色模式的 base 色度应该更低（通过色差间接验证）
        let dark_chroma = color_chroma(dark.base);
        let light_chroma = color_chroma(light.base);
        assert!(
            dark_chroma <= light_chroma + 0.01,
            "Dark mode should have lower chroma"
        );
    }

    fn rgb_luminance(c: Color) -> f64 {
        if let Color::Rgb(r, g, b) = c {
            0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64
        } else {
            0.0
        }
    }

    fn color_chroma(c: Color) -> f64 {
        if let Color::Rgb(r, g, b) = c {
            let (_, c, _) = oklch::rgb_to_oklch(r, g, b);
            c
        } else {
            0.0
        }
    }
}
