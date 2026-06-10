pub mod manager;

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

/// 标准Base64编码（无外部依赖）。
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    let chunks = input.chunks_exact(3);
    let rem = chunks.remainder();
    for chunk in chunks {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        out.push(TABLE[(n & 0x3F) as usize] as char);
    }
    if rem.len() == 1 {
        let n = u32::from(rem[0]) << 16;
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem.len() == 2 {
        let n = (u32::from(rem[0]) << 16) | (u32::from(rem[1]) << 8);
        out.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3F) as usize] as char);
        out.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}

/// 通过 OSC 52 转义序列将文本复制到系统剪贴板。
///
/// 支持 SSH 远程场景，多层 fallback：
/// 1. tmux 环境：DCS passthrough 包装
/// 2. 普通终端：直接发送 OSC 52 序列
pub fn osc52_copy(text: &str) -> Result<(), String> {
    let encoded = base64_encode(text.as_bytes());

    let sequence = if std::env::var("TMUX").is_ok() {
        // tmux 需要 DCS passthrough 包装
        format!("\x1bPtmux;\x1b\x1b]52;c;{encoded}\x07\x1b\\")
    } else {
        format!("\x1b]52;c;{encoded}\x07")
    };

    // 尝试写入 /dev/tty（绕过 PTY 层）
    #[cfg(unix)]
    {
        let tty = std::fs::File::open("/dev/tty").map_err(|e| format!("无法打开 /dev/tty: {e}"))?;
        let mut stdout = std::io::BufWriter::new(tty);
        std::io::Write::write_all(&mut stdout, sequence.as_bytes())
            .map_err(|e| format!("OSC 52 写入失败: {e}"))?;
        std::io::Write::flush(&mut stdout).map_err(|e| format!("OSC 52 flush 失败: {e}"))?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let _ = sequence;
        Err("OSC 52 仅支持 Unix 系统".to_string())
    }
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
