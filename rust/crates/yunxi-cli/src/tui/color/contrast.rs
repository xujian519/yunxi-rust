//! WCAG 对比度计算与验证。
//!
//! 基于 WCAG 2.1 标准：
//! - AA 级：普通文本 ≥ 4.5:1
//! - AAA 级：普通文本 ≥ 7:1

use super::oklch;

/// 计算两个 sRGB 颜色之间的对比度。
///
/// 返回比值（如 4.5 表示 4.5:1）。
pub fn contrast_ratio(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> f64 {
    let l1 = relative_luminance(bg);
    let l2 = relative_luminance(fg);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// 是否满足 WCAG AA 级对比度（≥ 4.5:1）。
pub fn meets_wcag_aa(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> bool {
    contrast_ratio(fg, bg) >= 4.5
}

/// 是否满足 WCAG AAA 级对比度（≥ 7:1）。
pub fn meets_wcag_aaa(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> bool {
    contrast_ratio(fg, bg) >= 7.0
}

/// 是否满足 WCAG 非文本元素对比度（≥ 3:1）。
pub fn meets_wcag_non_text(fg: (u8, u8, u8), bg: (u8, u8, u8)) -> bool {
    contrast_ratio(fg, bg) >= 3.0
}

/// 从 OKLCH 坐标计算对比度（便捷函数）。
pub fn contrast_oklch(fg_lch: (f64, f64, f64), bg_lch: (f64, f64, f64)) -> f64 {
    let fg = oklch::oklch_to_rgb(fg_lch.0, fg_lch.1, fg_lch.2);
    let bg = oklch::oklch_to_rgb(bg_lch.0, bg_lch.1, bg_lch.2);
    contrast_ratio(fg, bg)
}

/// 计算相对亮度（WCAG 2.1 算法）。
fn relative_luminance(c: (u8, u8, u8)) -> f64 {
    let r = srgb_to_linear(c.0 as f64 / 255.0);
    let g = srgb_to_linear(c.1 as f64 / 255.0);
    let b = srgb_to_linear(c.2 as f64 / 255.0);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white() {
        let ratio = contrast_ratio((0, 0, 0), (255, 255, 255));
        assert!((ratio - 21.0).abs() < 0.1, "Expected ~21:1, got {ratio}");
    }

    #[test]
    fn white_on_black() {
        let ratio = contrast_ratio((255, 255, 255), (0, 0, 0));
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn same_color_ratio_is_one() {
        let ratio = contrast_ratio((128, 128, 128), (128, 128, 128));
        assert!((ratio - 1.0).abs() < 0.01);
    }

    /// #E0E0E0 on #121212 ≈ 13.5:1（设计规范基准）
    #[test]
    fn ink_garden_text_contrast() {
        let ratio = contrast_ratio((0xE0, 0xE0, 0xE0), (0x12, 0x12, 0x12));
        assert!(
            (12.0..=15.0).contains(&ratio),
            "Expected ~13.5:1, got {ratio}"
        );
        assert!(meets_wcag_aaa((0xE0, 0xE0, 0xE0), (0x12, 0x12, 0x12)));
    }

    /// #D8DEE9 on #1A1D23 ≈ 12.5:1（墨园方案）
    #[test]
    fn ink_garden_exact_text_contrast() {
        let ratio = contrast_ratio((0xD8, 0xDE, 0xE9), (0x1A, 0x1D, 0x23));
        assert!(ratio >= 11.0, "Ink Garden text contrast too low: {ratio}");
        assert!(meets_wcag_aaa((0xD8, 0xDE, 0xE9), (0x1A, 0x1D, 0x23)));
    }

    /// #8A8A8A on #121212 ≈ 4.5:1（辅助文本 AA 级边界）
    #[test]
    fn muted_text_meets_aa() {
        let ratio = contrast_ratio((0x8A, 0x8A, 0x8A), (0x12, 0x12, 0x12));
        assert!(ratio >= 4.5, "Muted text below AA: {ratio}");
    }

    /// #3A3A3A on #3A3A3A = 1:1（禅径方案需要验证）
    #[test]
    fn zen_path_background_is_not_zero() {
        let ratio = contrast_ratio((0xDC, 0xDC, 0xCC), (0x3A, 0x3A, 0x3A));
        assert!(ratio >= 7.0, "Zen Path text contrast too low: {ratio}");
    }

    /// 明晰方案 #FFFFFF on #000000 = 21:1
    #[test]
    fn clear_mode_max_contrast() {
        assert!(meets_wcag_aaa((255, 255, 255), (0, 0, 0)));
    }
}
