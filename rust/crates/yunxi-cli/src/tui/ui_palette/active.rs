//! 运行时主题调色板：将 `ThemeManager` 选中的主题映射到 widget 使用的 RGB 三元组。

use std::sync::RwLock;

use ratatui::style::Color;

use crate::tui::theme::{ColorPalette, Theme};

use super::{
    BG_CODE, BG_MESSAGE_AI, BG_MESSAGE_USER, BG_PRIMARY, BG_SECONDARY, BG_TERTIARY, BORDER,
    BORDER_FOCUS, BRAND_YUNXI, BRAND_YUNXI_SHIMMER, ERROR, LABEL_YOU, LABEL_YUNXI, SUCCESS,
    TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY, USAGE_EMPTY, USAGE_FILL, WARNING,
};

static ACTIVE: RwLock<Option<Theme>> = RwLock::new(None);

pub fn apply(theme: Theme) {
    if let Ok(mut guard) = ACTIVE.write() {
        *guard = Some(theme);
    }
}

pub fn clear() {
    if let Ok(mut guard) = ACTIVE.write() {
        *guard = None;
    }
}

pub fn current_name() -> Option<String> {
    ACTIVE
        .read()
        .ok()
        .and_then(|g| g.as_ref().map(|t| t.name.clone()))
}

fn color_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}

fn pick(get: impl FnOnce(&ColorPalette) -> Color, default: (u8, u8, u8)) -> (u8, u8, u8) {
    ACTIVE
        .read()
        .ok()
        .and_then(|g| g.as_ref().map(|t| color_rgb(get(&t.colors))))
        .unwrap_or(default)
}

pub fn bg_primary() -> (u8, u8, u8) {
    pick(|p| p.bg_primary, BG_PRIMARY)
}

pub fn bg_secondary() -> (u8, u8, u8) {
    pick(|p| p.bg_secondary, BG_SECONDARY)
}

pub fn bg_tertiary() -> (u8, u8, u8) {
    pick(|p| p.bg_tertiary, BG_TERTIARY)
}

pub fn bg_message_user() -> (u8, u8, u8) {
    pick(|p| p.bg_input, BG_MESSAGE_USER)
}

pub fn bg_message_ai() -> (u8, u8, u8) {
    pick(|p| p.bg_tertiary, BG_MESSAGE_AI)
}

pub fn bg_code() -> (u8, u8, u8) {
    pick(|p| p.bg_primary, BG_CODE)
}

pub fn text_primary() -> (u8, u8, u8) {
    pick(|p| p.text_primary, TEXT_PRIMARY)
}

pub fn text_secondary() -> (u8, u8, u8) {
    pick(|p| p.text_secondary, TEXT_SECONDARY)
}

pub fn text_muted() -> (u8, u8, u8) {
    pick(|p| p.text_muted, TEXT_MUTED)
}

pub fn brand_yunxi() -> (u8, u8, u8) {
    pick(|p| p.brand, BRAND_YUNXI)
}

pub fn brand_yunxi_shimmer() -> (u8, u8, u8) {
    pick(|p| p.brand_shimmer, BRAND_YUNXI_SHIMMER)
}

pub fn border() -> (u8, u8, u8) {
    pick(|p| p.border, BORDER)
}

pub fn border_focus() -> (u8, u8, u8) {
    pick(|p| p.border_focus, BORDER_FOCUS)
}

pub fn label_you() -> (u8, u8, u8) {
    pick(|p| p.brand, LABEL_YOU)
}

pub fn label_yunxi() -> (u8, u8, u8) {
    pick(|p| p.text_accent, LABEL_YUNXI)
}

pub fn success() -> (u8, u8, u8) {
    pick(|p| p.success, SUCCESS)
}

pub fn error() -> (u8, u8, u8) {
    pick(|p| p.error, ERROR)
}

pub fn warning() -> (u8, u8, u8) {
    pick(|p| p.warning, WARNING)
}

pub fn usage_fill() -> (u8, u8, u8) {
    pick(|p| p.brand, USAGE_FILL)
}

pub fn usage_empty() -> (u8, u8, u8) {
    pick(|p| p.border, USAGE_EMPTY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::theme::Theme;

    #[test]
    fn apply_theme_changes_bg_primary() {
        clear();
        let before = bg_primary();
        apply(Theme::default_light());
        let after = bg_primary();
        assert_ne!(before, after);
        clear();
    }
}
