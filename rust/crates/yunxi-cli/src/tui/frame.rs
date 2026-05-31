//! 终端帧缓冲：按 (x, y) 合成各区域，避免流式换行导致布局错乱。

use crate::tui::layout::Rect;

/// 固定行数的终端帧。
pub(crate) struct Frame {
    width: u16,
    height: u16,
    rows: Vec<String>,
}

impl Frame {
    pub(crate) fn new(width: u16, height: u16) -> Self {
        let blank = " ".repeat(width as usize);
        Self {
            width,
            height,
            rows: vec![blank; height as usize],
        }
    }

    /// 写入整行（截断或填充到终端宽度）。
    pub(crate) fn set_row(&mut self, y: u16, line: &str) {
        if y >= self.height {
            return;
        }
        self.rows[y as usize] = pad_ansi_line(line, self.width);
    }

    /// 写入已按区域宽度准备好的行（不再追加无底色空格）。
    pub(crate) fn set_row_prepared(&mut self, y: u16, line: &str) {
        if y >= self.height {
            return;
        }
        self.rows[y as usize] = truncate_ansi_to_width(line, self.width as usize);
    }

    /// 在指定区域覆盖写入多行（用于帮助层等）。
    pub(crate) fn overlay_lines(&mut self, area: Rect, lines: &[String]) {
        if !area.is_valid() {
            return;
        }
        for (i, line) in lines.iter().enumerate() {
            let y = area.y.saturating_add(i as u16);
            if y >= area.y.saturating_add(area.height) || y >= self.height {
                break;
            }
            let clipped = truncate_ansi_to_width(line, area.width as usize);
            let existing = self.rows[y as usize].clone();
            self.rows[y as usize] = splice_ansi_at(&existing, area.x, &clipped, self.width);
        }
    }

    /// 在矩形区域内绘制多行文本（从区域左上角起，按行截断/填充）。
    pub(crate) fn paint_area(&mut self, area: Rect, body: &str) {
        if !area.is_valid() {
            return;
        }
        let lines = fit_lines(body, area.width as usize, area.height as usize);
        for (i, line) in lines.iter().enumerate() {
            if i >= area.height as usize {
                break;
            }
            self.set_row_prepared(area.y.saturating_add(i as u16), line);
        }
    }

    /// 生成带光标定位的 ANSI 帧（不消耗 self）。
    pub(crate) fn as_ansi(&self) -> String {
        // 不用全屏擦除（\x1b[2J）或逐行 \x1b[2K，避免 IME 候选窗闪烁/跳动
        let mut out = String::from("\x1b[H");
        for (i, row) in self.rows.iter().enumerate() {
            out.push_str(&format!("\x1b[{};1H", i + 1));
            out.push_str(row);
        }
        out.push_str("\x1b[J");
        out
    }
}

/// 将多行文本裁剪/填充到固定行数。
pub(crate) fn fit_lines(content: &str, width: usize, height: usize) -> Vec<String> {
    let mut lines: Vec<String> = if content.is_empty() {
        Vec::new()
    } else {
        content
            .lines()
            .map(|line| truncate_ansi_to_width(line, width))
            .collect()
    };
    while lines.len() < height {
        lines.push(String::new());
    }
    lines.truncate(height);
    lines
}

/// 左右两列合成一行。
pub(crate) fn compose_row(left: &str, left_width: u16, right: &str, right_width: u16) -> String {
    let mut row = pad_ansi_line(left, left_width);
    row.push_str(&pad_ansi_line(right, right_width));
    row
}

/// 填充 ANSI 行到指定可见宽度。
pub(crate) fn pad_ansi_line(line: &str, width: u16) -> String {
    let truncated = truncate_ansi_to_width(line, width as usize);
    let used = visible_width(&truncated);
    if used >= width {
        truncated
    } else {
        format!("{truncated}{}", " ".repeat((width - used) as usize))
    }
}

/// 粗略截断 ANSI 字符串到指定可见宽度。
pub(crate) fn truncate_ansi_to_width(s: &str, max_width: usize) -> String {
    let mut visible_width = 0usize;
    let mut result = String::new();
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            result.push(c);
            continue;
        }
        if in_escape {
            result.push(c);
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        let cw = char_width(c);
        if visible_width + cw > max_width {
            break;
        }
        result.push(c);
        visible_width += cw;
    }

    result
}

/// 按可见宽度折行；不在 ANSI 转义中间断开，折行处补齐 reset 避免花屏。
pub(crate) fn wrap_ansi_to_width(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    let mut line = String::new();
    let mut visible = 0usize;
    let mut in_escape = false;

    for c in text.chars() {
        if c == '\x1b' {
            in_escape = true;
            line.push(c);
            continue;
        }
        if in_escape {
            line.push(c);
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        let cw = char_width(c);
        if visible + cw > max_width && !line.is_empty() {
            result.push(format!("{line}\x1b[0m"));
            line.clear();
            visible = 0;
        }
        line.push(c);
        visible += cw;
    }

    if !line.is_empty() {
        result.push(line);
    }
    if result.is_empty() {
        result.push(String::new());
    }
    result
}

/// 可见字符宽度（CJK 计 2）。
pub(crate) fn visible_width(s: &str) -> u16 {
    let mut width = 0u16;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        width += char_width(c) as u16;
    }
    width
}

fn char_width(c: char) -> usize {
    if c.is_ascii() {
        1
    } else if ('\u{2500}'..='\u{257F}').contains(&c) || ('\u{2550}'..='\u{256C}').contains(&c) {
        1
    } else {
        2
    }
}

/// 在基行指定列插入片段（保留 ANSI 前缀区域）。
fn splice_ansi_at(base: &str, x: u16, insert: &str, max_width: u16) -> String {
    if x >= max_width {
        return base.to_string();
    }
    let left = truncate_ansi_to_width(base, x as usize);
    let left_vis = visible_width(&left);
    let pad = if left_vis < x {
        format!("{}{}", left, " ".repeat((x - left_vis) as usize))
    } else {
        left
    };
    let room = max_width.saturating_sub(x);
    let clipped = truncate_ansi_to_width(insert, room as usize);
    let combined = format!("{pad}{clipped}");
    pad_ansi_line(&combined, max_width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ansi_preserves_escapes() {
        let s = "\x1b[1mhello\x1b[0m world";
        let truncated = truncate_ansi_to_width(s, 8);
        assert!(truncated.contains("\x1b[1m"));
        assert!(truncated.contains("hello"));
    }

    #[test]
    fn wrap_ansi_does_not_split_escape_sequences() {
        let colored = "\x1b[1m\x1b[38;5;183m云熙智能体\x1b[0m".to_string();
        let wrapped = wrap_ansi_to_width(&colored, 6);
        assert!(wrapped.len() >= 2);
        for line in &wrapped {
            assert!(
                !line.contains("[0m") || line.contains("\x1b[0m"),
                "leaked bare reset: {line:?}"
            );
        }
    }

    #[test]
    fn compose_row_fills_width() {
        let row = compose_row("left", 10, "right", 10);
        assert_eq!(visible_width(&row), 20);
    }

    #[test]
    fn fit_lines_pads_height() {
        let lines = fit_lines("a\nb", 20, 4);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[2], "");
    }

    #[test]
    fn frame_set_row_updates_line() {
        let mut frame = Frame::new(40, 5);
        frame.set_row(0, "title");
        let rendered = frame.as_ansi();
        assert!(rendered.contains("title"));
    }

    #[test]
    fn overlay_lines_splices_at_column() {
        let mut frame = Frame::new(60, 3);
        frame.overlay_lines(Rect::new(20, 1, 15, 1), &["hello".to_string()]);
        let rendered = frame.as_ansi();
        assert!(rendered.contains("hello"));
    }

    #[test]
    fn frame_positions_each_row_at_column_one() {
        let mut frame = Frame::new(40, 3);
        frame.set_row(0, "top");
        frame.set_row(2, "bottom");
        let rendered = frame.as_ansi();
        assert!(rendered.contains("\x1b[1;1H"));
        assert!(rendered.contains("\x1b[3;1H"));
        assert!(rendered.contains("top"));
        assert!(rendered.contains("bottom"));
    }
}
