use std::io::{self, Read, Write};

use crossterm::terminal;

const COLORFGBG: &str = "COLORFGBG";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TerminalBackground {
    Dark,
    Light,
}

impl TerminalBackground {
    pub(crate) fn is_dark(self) -> bool {
        matches!(self, TerminalBackground::Dark)
    }
}

pub(crate) fn detect_background() -> TerminalBackground {
    if let Ok(mode) = std::env::var("YUNXI_TUI_BACKGROUND") {
        if mode.eq_ignore_ascii_case("dark") {
            return TerminalBackground::Dark;
        }
        if mode.eq_ignore_ascii_case("light") {
            return TerminalBackground::Light;
        }
    }

    if let Ok(theme) = std::env::var("TERMINAL_THEME") {
        if theme.eq_ignore_ascii_case("dark") {
            return TerminalBackground::Dark;
        }
        if theme.eq_ignore_ascii_case("light") {
            return TerminalBackground::Light;
        }
    }

    if let Some(bg) = parse_colorfgbg() {
        return bg;
    }

    if let Some(bg) = query_terminal_background() {
        return bg;
    }

    match std::env::var("TERM_PROGRAM").as_deref() {
        Ok("vscode" | "Visual Studio Code" | "iTerm.app" | "cursor" | "WarpTerminal") => {
            TerminalBackground::Dark
        }
        Ok("Apple_Terminal") => TerminalBackground::Light,
        _ => TerminalBackground::Dark,
    }
}

fn parse_colorfgbg() -> Option<TerminalBackground> {
    let val = std::env::var(COLORFGBG).ok()?;
    let bg = val.rsplit(';').next()?.parse::<u16>().ok()?;
    let is_light = matches!(bg, 7 | 9..=15 | 250..=255);
    Some(if is_light {
        TerminalBackground::Light
    } else {
        TerminalBackground::Dark
    })
}

fn query_terminal_background() -> Option<TerminalBackground> {
    let (width, height) = terminal::size().ok()?;

    // 保存光标位置，查询背景色，读取响应，恢复光标
    let query = "\x1b7\x1b]11;?\x1b\\\x1b8";
    let mut stdout = io::stdout();
    if stdout.write_all(query.as_bytes()).is_err() {
        return None;
    }
    if stdout.flush().is_err() {
        return None;
    }

    let mut stdin = io::stdin();
    let mut buf = [0u8; 64];
    let mut response = String::new();

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(200);

    while start.elapsed() < timeout {
        if let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
            for &byte in &buf[..n] {
                if byte == 0x1b {
                    break;
                }
                if byte.is_ascii_graphic() || byte == b';' || byte == b':' || byte == b'/' {
                    response.push(byte as char);
                }
            }
            if response.contains("rgb") {
                break;
            }
        }
    }

    if let Some(rgb_start) = response.find("rgb:") {
        let hex_part = &response[rgb_start + 4..];
        let parts: Vec<&str> = hex_part.split('/').collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u16::from_str_radix(parts[0], 16),
                u16::from_str_radix(parts[1], 16),
                u16::from_str_radix(parts[2], 16),
            ) {
                let (r, g, b) = ((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8);
                return Some(srgb_lightness(r, g, b));
            }
        }
    }

    None

}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Srgb {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

fn srgb_lightness(r: u8, g: u8, b: u8) -> TerminalBackground {
    let luminance = 0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64;
    if luminance > 128.0 {
        TerminalBackground::Light
    } else {
        TerminalBackground::Dark
    }
}

fn srgb_to_linear(c: f64) -> f64 {
    let normalized = c / 255.0;
    if normalized <= 0.04045 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_xyz(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let x = 0.4124564 * r + 0.3575761 * g + 0.1804375 * b;
    let y = 0.2126729 * r + 0.7151522 * g + 0.0721750 * b;
    let z = 0.0193339 * r + 0.1191920 * g + 0.9503041 * b;
    (x, y, z)
}

fn xyz_to_lab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    let xn = 0.95047;
    let yn = 1.0;
    let zn = 1.08883;

    let fx = xyz_to_lab_f(x / xn);
    let fy = xyz_to_lab_f(y / yn);
    let fz = xyz_to_lab_f(z / zn);

    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);

    (l, a, b)
}

fn xyz_to_lab_f(t: f64) -> f64 {
    let delta = 6.0 / 29.0;
    if t > delta * delta * delta {
        t.cbrt()
    } else {
        t / (3.0 * delta * delta) + 4.0 / 29.0
    }
}

pub(crate) fn cie76_distance(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f64 {
    let (r1, g1, b1) = (c1.0 as f64, c1.1 as f64, c1.2 as f64);
    let (r2, g2, b2) = (c2.0 as f64, c2.1 as f64, c2.2 as f64);

    let (lr1, lg1, lb1) = (srgb_to_linear(r1), srgb_to_linear(g1), srgb_to_linear(b1));
    let (lr2, lg2, lb2) = (srgb_to_linear(r2), srgb_to_linear(g2), srgb_to_linear(b2));

    let (x1, y1, z1) = linear_to_xyz(lr1, lg1, lb1);
    let (x2, y2, z2) = linear_to_xyz(lr2, lg2, lb2);

    let (l1, a1, b1) = xyz_to_lab(x1, y1, z1);
    let (l2, a2, b2) = xyz_to_lab(x2, y2, z2);

    let dl = l1 - l2;
    let da = a1 - a2;
    let db = b1 - b2;

    (dl * dl + da * da + db * db).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cie76_same_color_zero_distance() {
        let d = cie76_distance((100, 150, 200), (100, 150, 200));
        assert!(d < 0.01);
    }

    #[test]
    fn cie76_different_colors() {
        let d = cie76_distance((255, 255, 255), (0, 0, 0));
        assert!(d > 50.0);
    }

    #[test]
    fn srgb_lightness_detection() {
        assert_eq!(srgb_lightness(0, 0, 0), TerminalBackground::Dark);
        assert_eq!(srgb_lightness(255, 255, 255), TerminalBackground::Light);
    }

    #[test]
    fn colorfgbg_parsing() {
        let _guard = env_lock();
        std::env::set_var(COLORFGBG, "0;15");
        assert_eq!(parse_colorfgbg(), Some(TerminalBackground::Light));
        std::env::set_var(COLORFGBG, "15;0");
        assert_eq!(parse_colorfgbg(), Some(TerminalBackground::Dark));
        std::env::remove_var(COLORFGBG);
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        use std::sync::{Mutex, OnceLock};
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|e| e.into_inner())
    }
}
