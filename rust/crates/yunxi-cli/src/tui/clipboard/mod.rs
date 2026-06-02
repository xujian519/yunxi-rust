pub mod manager;

pub use manager::{ClipboardHistory, ClipboardManager};

/// 去除 ANSI 转义序列，便于复制到剪贴板。
#[must_use]
pub(crate) fn strip_ansi(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        output.push(ch);
    }

    output
}

/// 写入系统剪贴板。
pub(crate) fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .map_err(|e| format!("无法打开剪贴板: {e}"))?
        .set_text(text)
        .map_err(|e| format!("写入剪贴板失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_color_codes() {
        let plain = strip_ansi("\x1b[31m错误\x1b[0m");
        assert_eq!(plain, "错误");
    }
}
