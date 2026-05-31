//! TUI 品牌色与常用 ANSI 着色（云熙紫 / 粉强调 / 灰提示）。

/// 品牌标识（樱花）。
pub(crate) const BRAND_MARK: &str = "🌸";

/// 深底品牌主色（紫，256 色）。
const BRAND_DARK: u8 = 183;
/// 深底强调色（粉）。
const ACCENT_DARK: u8 = 213;
/// 深底次要文字。
const DIM_DARK: u8 = 245;

/// 浅底品牌主色（深紫）。
const BRAND_LIGHT: u8 = 55;
/// 浅底强调色（深粉/洋红）。
const ACCENT_LIGHT: u8 = 162;
/// 浅底次要文字（中灰）。
const DIM_LIGHT: u8 = 243;
/// 浅底选中 / 高亮（深橙）。
const HIGHLIGHT_LIGHT: u8 = 166;

/// 浅底对话区正文（深灰紫，保证可读）。
pub(crate) const LIGHT_BODY: u8 = 237;
/// 浅底对话区次要信息。
pub(crate) const LIGHT_META: u8 = 243;

/// 品牌主色：根据终端背景自适应。
pub(crate) fn brand() -> u8 {
    if terminal_light_background() {
        BRAND_LIGHT
    } else {
        BRAND_DARK
    }
}

/// 强调色：根据终端背景自适应。
pub(crate) fn accent() -> u8 {
    if terminal_light_background() {
        ACCENT_LIGHT
    } else {
        ACCENT_DARK
    }
}

/// 次要文字：根据终端背景自适应。
pub(crate) fn dim_color() -> u8 {
    if terminal_light_background() {
        DIM_LIGHT
    } else {
        DIM_DARK
    }
}

/// 选中 / 高亮。
pub(crate) fn highlight() -> u8 {
    if terminal_light_background() {
        HIGHLIGHT_LIGHT
    } else {
        214
    }
}

/// 用户角色标签色。
pub(crate) fn user_role_color() -> u8 {
    if terminal_light_background() {
        25
    } else {
        183
    }
}

/// 助手角色标签色。
pub(crate) fn assistant_role_color() -> u8 {
    if terminal_light_background() {
        162
    } else {
        213
    }
}

/// 系统角色标签色。
pub(crate) fn system_role_color() -> u8 {
    if terminal_light_background() {
        242
    } else {
        246
    }
}

/// 正文内容色。
pub(crate) fn content_color() -> u8 {
    if terminal_light_background() {
        235
    } else {
        252
    }
}

/// 输入区背景色（256 色索引）。
pub(crate) fn input_bg_color() -> u8 {
    if terminal_light_background() {
        254
    } else {
        236
    }
}

/// 输入区正文色（ratatui 路径；深底浅字 / 浅底深字）。
pub(crate) fn input_text_color() -> u8 {
    if terminal_light_background() {
        0
    } else {
        252
    }
}

/// 向后兼容：保留原常量名供 ANSI 路径使用（深色默认值）。
pub(crate) const BRAND: u8 = BRAND_DARK;
pub(crate) const ACCENT: u8 = ACCENT_DARK;
pub(crate) const HIGHLIGHT: u8 = 214;
pub(crate) const DIM: u8 = DIM_DARK;

pub(crate) fn fg256(n: u8, text: &str) -> String {
    format!("\x1b[38;5;{n}m{text}\x1b[0m")
}

pub(crate) fn bold_fg256(n: u8, text: &str) -> String {
    format!("\x1b[1m\x1b[38;5;{n}m{text}\x1b[0m")
}

pub(crate) fn dim(text: &str) -> String {
    format!("\x1b[2m{text}\x1b[0m")
}

/// 对话区正文色：浅底用深灰紫，深底用品牌紫。
pub(crate) fn chat_body(text: &str) -> String {
    if terminal_light_background() {
        fg256(LIGHT_BODY, text)
    } else {
        fg256(BRAND, text)
    }
}

/// 对话区次要信息（模型、目录等）。
pub(crate) fn chat_meta(text: &str) -> String {
    if terminal_light_background() {
        fg256(LIGHT_META, text)
    } else {
        dim(text)
    }
}

/// 终端是否为浅色背景（`COLORFGBG` / 环境变量推断）。
#[must_use]
pub(crate) fn terminal_light_background() -> bool {
    if let Ok(mode) = std::env::var("YUNXI_TUI_BACKGROUND") {
        if mode.eq_ignore_ascii_case("dark") {
            return false;
        }
        if mode.eq_ignore_ascii_case("light") {
            return true;
        }
    }
    if let Ok(theme) = std::env::var("TERMINAL_THEME") {
        if theme.eq_ignore_ascii_case("dark") {
            return false;
        }
        if theme.eq_ignore_ascii_case("light") {
            return true;
        }
    }
    parse_colorfgbg_light_background().unwrap_or_else(default_light_background_guess)
}

fn parse_colorfgbg_light_background() -> Option<bool> {
    let val = std::env::var("COLORFGBG").ok()?;
    let bg = val.rsplit(';').next()?.parse::<u16>().ok()?;
    Some(matches!(bg, 7 | 9..=15 | 250..=255))
}

fn default_light_background_guess() -> bool {
    match std::env::var("TERM_PROGRAM").as_deref() {
        Ok("vscode" | "Visual Studio Code" | "iTerm.app" | "cursor") => false,
        Ok("Apple_Terminal") => true,
        _ => true,
    }
}

/// 输入区背景色（256 色）：与终端底色区分，便于 IME 候选窗对比。
fn input_bg256() -> u8 {
    input_bg_color()
}

fn input_bg_prefix() -> String {
    format!("\x1b[48;5;{}m", input_bg256())
}

/// 输入区正文色（浅底黑字 / 深底白字）。
fn input_fg_escape() -> &'static str {
    if terminal_light_background() {
        "\x1b[30m"
    } else {
        "\x1b[97m"
    }
}

/// 输入区弱化前景（占位、快捷键提示），仍保持可读对比度。
fn input_faint_fg_escape() -> &'static str {
    if terminal_light_background() {
        "\x1b[90m"
    } else {
        "\x1b[37m"
    }
}

fn input_line_reset() -> &'static str {
    "\x1b[0m"
}

/// 将输入行内容铺满指定可见宽度并统一铺底，避免 IME 候选区与界面底色混淆。
pub(crate) fn input_line_padded(content: &str, width: usize) -> String {
    use crate::tui::frame::{truncate_ansi_to_width, visible_width};

    let clipped = truncate_ansi_to_width(content, width);
    let used = usize::from(visible_width(&clipped));
    let pad = width.saturating_sub(used);
    format!(
        "{clipped}{}{}",
        if pad > 0 {
            format!("{}{}", input_bg_prefix(), " ".repeat(pad))
        } else {
            String::new()
        },
        input_line_reset()
    )
}

/// 输入区普通文字（带背景）。
pub(crate) fn input_text(text: &str) -> String {
    format!("{}{text}{}", input_bg_prefix(), input_fg_escape())
}

/// 输入区加粗（提示符等，带背景）。
pub(crate) fn input_bold(text: &str) -> String {
    if terminal_light_background() {
        format!("{}\x1b[1;30m{text}", input_bg_prefix())
    } else {
        format!("{}\x1b[1;97m{text}", input_bg_prefix())
    }
}

/// 输入区弱化（占位、快捷键提示，带背景）。
pub(crate) fn input_faint(text: &str) -> String {
    format!("{}{}{text}", input_bg_prefix(), input_faint_fg_escape())
}

/// 斜杠补全选中行（高对比底色，替代反色以免 IME 区域发灰）。
pub(crate) fn input_completion_selected(text: &str) -> String {
    if terminal_light_background() {
        format!("\x1b[48;5;25;38;5;231;1m{text}")
    } else {
        format!("\x1b[48;5;183;38;5;235;1m{text}")
    }
}

/// 斜杠补全普通行（带输入区背景）。
pub(crate) fn input_completion_item(text: &str) -> String {
    input_text(text)
}

#[cfg(test)]
mod input_color_tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn colorfgbg_light_background() {
        let _guard = env_lock();
        std::env::set_var("COLORFGBG", "0;15");
        std::env::remove_var("YUNXI_TUI_BACKGROUND");
        assert!(terminal_light_background());
        std::env::set_var("COLORFGBG", "15;0");
        assert!(!terminal_light_background());
        std::env::remove_var("COLORFGBG");
    }

    #[test]
    fn yunxi_background_override() {
        let _guard = env_lock();
        std::env::set_var("YUNXI_TUI_BACKGROUND", "dark");
        assert!(!terminal_light_background());
        std::env::set_var("YUNXI_TUI_BACKGROUND", "light");
        assert!(terminal_light_background());
        std::env::remove_var("YUNXI_TUI_BACKGROUND");
    }
}
