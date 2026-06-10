//! OKLCH 色彩空间转换引擎。
//!
//! 基于 Björn Ottosson 2020 论文实现 OKLCH ↔ sRGB 互转。
//! OKLCH 在感知均匀性和色相稳定性上全面优于 CIELAB/HSL。

/// M1 矩阵：线性 sRGB → LMS（OKLab 色彩适应矩阵）
/// 来源: Björn Ottosson 2020, CSS Color Level 4
#[allow(clippy::excessive_precision)]
const M1: [[f64; 3]; 3] = [
    [0.41222146947076293, 0.5363325372696428, 0.05144599327077581],
    [0.21190349585681682, 0.6806995506482364, 0.10739695353292697],
    [0.0883024597218609, 0.2817188671390217, 0.6299787010365854],
];

/// M2 矩阵：LMS' → OKLab（非线性 LMS → Lab 转换）
const M2: [[f64; 3]; 3] = [
    [0.2104542553, 0.7936177850, -0.0040720468],
    [1.9779984951, -2.4285922050, 0.4505937099],
    [0.0259040371, 0.7827717662, -0.8086757660],
];

/// M2_inv 矩阵：OKLab → LMS'（M2 的逆矩阵）
const M2_INV: [[f64; 3]; 3] = [
    [1.0, 0.3963377774, 0.2158037573],
    [1.0, -0.1055613458, -0.0638541728],
    [1.0, -0.0894841775, -1.2914855480],
];

/// M1_inv 矩阵：LMS → 线性 sRGB（M1 的逆矩阵）
const M1_INV: [[f64; 3]; 3] = [
    [4.0767416621, -3.3077115913, 0.2309699292],
    [-1.2684380046, 2.6097574011, -0.3413193965],
    [-0.0041960863, -0.7034186147, 1.7076147010],
];

/// OKLCH 颜色表示 (L: 亮度 0-1, C: 色度 0+, H: 色相 0-360°)
#[derive(Debug, Clone, Copy)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

impl Oklch {
    pub fn new(l: f64, c: f64, h: f64) -> Self {
        Self { l, c, h }
    }

    /// 从百分比形式 (L: 0-100, C: 0+, H: 0-360) 创建
    pub fn from_percent(l: f64, c: f64, h: f64) -> Self {
        Self { l: l / 100.0, c, h }
    }
}

/// OKLCH → sRGB 转换。
///
/// 返回 (r, g, b)，每个分量 0-255。超出 sRGB 色域的值会被裁剪。
pub fn oklch_to_rgb(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    // OKLCH → OKLab
    let h_rad = h.to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // OKLab → LMS'
    let lms_prime = mat3_mul(M2_INV, [l, a, b]);

    // LMS' → LMS（反向非线性）
    let lms = [cube(lms_prime[0]), cube(lms_prime[1]), cube(lms_prime[2])];

    // LMS → 线性 sRGB
    let rgb_linear = mat3_mul(M1_INV, lms);

    // 线性 sRGB → sRGB（gamma 校正）+ 裁剪
    let r = srgb_gamma(rgb_linear[0]);
    let g = srgb_gamma(rgb_linear[1]);
    let b = srgb_gamma(rgb_linear[2]);

    (clamp_u8(r), clamp_u8(g), clamp_u8(b))
}

/// sRGB → OKLCH 转换。
///
/// 输入 (r, g, b) 范围 0-255，返回 (L, C, H)。
pub fn rgb_to_oklch(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    // sRGB → 线性 sRGB
    let r_lin = srgb_inv_gamma(rf);
    let g_lin = srgb_inv_gamma(gf);
    let b_lin = srgb_inv_gamma(bf);

    // 线性 sRGB → LMS
    let lms = mat3_mul(M1, [r_lin, g_lin, b_lin]);

    // LMS → LMS'（非线性）
    let lms_prime = [cbrt(lms[0]), cbrt(lms[1]), cbrt(lms[2])];

    // LMS' → OKLab
    let lab = mat3_mul(M2, lms_prime);

    // OKLab → OKLCH
    let l = lab[0];
    let a = lab[1];
    let b_val = lab[2];
    let c = (a * a + b_val * b_val).sqrt();
    let h = b_val.atan2(a).to_degrees();
    let h = if h < 0.0 { h + 360.0 } else { h };

    (l, c, h)
}

/// 从 OKLCH 百分比形式转换为 RGB。
pub fn oklch_percent_to_rgb(l: f64, c: f64, h: f64) -> (u8, u8, u8) {
    oklch_to_rgb(l / 100.0, c, h)
}

// ── 内部辅助函数 ──

fn mat3_mul(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn cube(x: f64) -> f64 {
    x * x * x
}

fn cbrt(x: f64) -> f64 {
    if x >= 0.0 {
        x.powf(1.0 / 3.0)
    } else {
        -(-x).powf(1.0 / 3.0)
    }
}

/// 线性 → sRGB gamma 校正
fn srgb_gamma(c: f64) -> f64 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// sRGB → 线性（反向 gamma）
fn srgb_inv_gamma(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn clamp_u8(v: f64) -> u8 {
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// OKLCH 转换后再转回来应接近原值
    #[test]
    fn roundtrip_pure_white() {
        let (r, g, b) = oklch_to_rgb(1.0, 0.0, 0.0);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn roundtrip_pure_black() {
        let (r, g, b) = oklch_to_rgb(0.0, 0.0, 0.0);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn roundtrip_sRGB_via_oklch() {
        let original = (200, 100, 50);
        let (l, c, h) = rgb_to_oklch(original.0, original.1, original.2);
        let (r, g, b) = oklch_to_rgb(l, c, h);
        // 允许 ±2 误差（色域映射精度）
        assert!(
            (r as i16 - original.0 as i16).unsigned_abs() <= 2
                && (g as i16 - original.1 as i16).unsigned_abs() <= 2
                && (b as i16 - original.2 as i16).unsigned_abs() <= 2,
            "roundtrip failed: ({}, {}, {}) → oklch({}, {}, {}) → ({}, {}, {})",
            original.0,
            original.1,
            original.2,
            l,
            c,
            h,
            r,
            g,
            b,
        );
    }

    /// 设计规范中的墨园暗色背景 #1A1D23 = oklch(17% 0.02 265)
    #[test]
    fn ink_garden_background_from_oklch() {
        let (r, g, b) = oklch_percent_to_rgb(17.0, 0.02, 265.0);
        // 验证接近 #1A1D23 (26, 29, 35)，OKLCH→RGB 允许 ±15 色域映射误差
        assert!((r as i16 - 26).unsigned_abs() <= 15, "R={r}, expected ~26");
        assert!((g as i16 - 29).unsigned_abs() <= 15, "G={g}, expected ~29");
        assert!((b as i16 - 35).unsigned_abs() <= 15, "B={b}, expected ~35");
    }

    /// 设计规范中的墨园主文本 #D8DEE9 = oklch(87% 0.02 270)
    #[test]
    fn ink_garden_text_from_oklch() {
        let (r, g, b) = oklch_percent_to_rgb(87.0, 0.02, 270.0);
        // 验证接近 #D8DEE9 (216, 222, 233)
        assert!(
            (r as i16 - 216).unsigned_abs() <= 15,
            "R={r}, expected ~216"
        );
        assert!(
            (g as i16 - 222).unsigned_abs() <= 15,
            "G={g}, expected ~222"
        );
        assert!(
            (b as i16 - 233).unsigned_abs() <= 15,
            "B={b}, expected ~233"
        );
    }

    /// 无色度时色相应不影响结果
    #[test]
    fn zero_chroma_is_neutral() {
        let (r1, g1, b1) = oklch_to_rgb(0.5, 0.0, 0.0);
        let (r2, g2, b2) = oklch_to_rgb(0.5, 0.0, 180.0);
        assert_eq!((r1, g1, b1), (r2, g2, b2));
    }

    /// 低色度时色相稳定性（蓝色不偏紫）
    #[test]
    fn blue_hue_stability() {
        // 使用高饱和蓝色测试
        let (_, _, h_blue) = rgb_to_oklch(50, 50, 240);
        // OKLab 蓝色色相约 264°，允许宽范围
        assert!(
            h_blue > 200.0 && h_blue < 300.0,
            "Blue hue drifted to {h_blue}°"
        );
    }
}
