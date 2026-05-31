#![allow(dead_code)]

use std::io::Write;

use crossterm::cursor::MoveToColumn;
use crossterm::queue;
use crossterm::style::Color;
use crossterm::terminal::{Clear, ClearType};

/// 当前状态栏快照——由 REPL 循环每轮采集。
#[derive(Debug, Clone)]
pub(crate) struct StatusBarSnapshot {
    pub model: String,
    pub auto_decision: Option<String>,
    pub permission_mode: String,
    pub session_id: String,
    pub cumulative_input_tokens: u64,
    pub cumulative_output_tokens: u64,
    pub estimated_cost_usd: f64,
    pub git_branch: Option<String>,
    pub thinking: bool,
    pub turn_elapsed_secs: Option<f64>,
    pub active_tool: Option<String>,
    pub turn_output_tokens: u32,
    pub turn_output_max: u32,
    pub route_hint: Option<String>,
    pub semantic_on: bool,
    pub patent_case_hint: Option<String>,
    pub flow_hitl_hint: Option<String>,
}

impl Default for StatusBarSnapshot {
    fn default() -> Self {
        Self {
            model: String::new(),
            auto_decision: None,
            permission_mode: String::new(),
            session_id: String::new(),
            cumulative_input_tokens: 0,
            cumulative_output_tokens: 0,
            estimated_cost_usd: 0.0,
            git_branch: None,
            thinking: false,
            turn_elapsed_secs: None,
            active_tool: None,
            turn_output_tokens: 0,
            turn_output_max: 0,
            route_hint: None,
            semantic_on: false,
            patent_case_hint: None,
            flow_hitl_hint: None,
        }
    }
}

/// ANSI 状态栏渲染器。
pub(crate) struct StatusBar {
    terminal_width: u16,
}

impl StatusBar {
    pub(crate) fn new() -> Self {
        let terminal_width = crossterm::terminal::size().map_or(80, |(w, _)| w);
        Self { terminal_width }
    }

    #[must_use]
    pub(crate) fn with_width(width: u16) -> Self {
        Self {
            terminal_width: width.max(20),
        }
    }

    /// 刷新终端宽度（窗口大小可能变化）。
    pub(crate) fn refresh_width(&mut self) {
        self.terminal_width = crossterm::terminal::size().map_or(80, |(w, _)| w);
    }

    /// 将状态栏渲染为一行 ANSI 字符串。
    pub(crate) fn render(&self, snapshot: &StatusBarSnapshot) -> String {
        let segments = self.build_segments(snapshot);
        let mut out = String::new();
        let mut first = true;
        let mut used = 0u16;

        for seg in &segments {
            if !first {
                if used + 3 > self.terminal_width {
                    break;
                }
                out.push_str("\x1b[38;5;8m │ \x1b[0m");
                used += 3;
            }
            first = false;
            let seg_width = seg.display_width();
            if used + seg_width > self.terminal_width {
                let available = self.terminal_width.saturating_sub(used);
                if available > 0 {
                    let truncated: String = seg.text.chars().take(available as usize).collect();
                    out.push_str(&color_fg(&truncated, seg.color));
                }
                break;
            }
            out.push_str(&color_fg(&seg.text, seg.color));
            used += seg_width;
        }

        out
    }

    /// 将状态栏写入 stdout（清除当前行后重绘）。
    pub(crate) fn write_to(
        &self,
        writer: &mut impl Write,
        snapshot: &StatusBarSnapshot,
    ) -> std::io::Result<()> {
        let line = self.render(snapshot);
        queue!(writer, MoveToColumn(0))?;
        queue!(writer, Clear(ClearType::FromCursorDown))?;
        queue!(writer, crossterm::style::Print(line))?;
        writer.flush()?;
        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn build_segments(&self, snapshot: &StatusBarSnapshot) -> Vec<StatusBarSegment> {
        let mut segments = Vec::new();

        let neutral = Color::Reset;

        // 模型
        if !snapshot.model.is_empty() {
            let display = snapshot.auto_decision.as_deref().unwrap_or(&snapshot.model);
            segments.push(StatusBarSegment {
                text: if snapshot.thinking {
                    format!("⏳ {}", display)
                } else {
                    display.to_string()
                },
                color: neutral,
            });
        }

        // 权限（仅 danger-full-access 保留警示色）
        if !snapshot.permission_mode.is_empty() {
            segments.push(StatusBarSegment {
                text: snapshot.permission_mode.clone(),
                color: permission_mode_color(&snapshot.permission_mode),
            });
        }

        // 会话 ID（截断为前 8 位）
        if !snapshot.session_id.is_empty() {
            let id_preview = if snapshot.session_id.len() > 8 {
                &snapshot.session_id[..8]
            } else {
                &snapshot.session_id
            };
            segments.push(StatusBarSegment {
                text: id_preview.to_string(),
                color: neutral,
            });
        }

        // Token 统计
        if snapshot.cumulative_input_tokens > 0 || snapshot.cumulative_output_tokens > 0 {
            segments.push(StatusBarSegment {
                text: format!(
                    "{} in / {} out",
                    format_tokens(snapshot.cumulative_input_tokens),
                    format_tokens(snapshot.cumulative_output_tokens)
                ),
                color: neutral,
            });
        }

        // 费用
        if snapshot.estimated_cost_usd > 0.0 {
            segments.push(StatusBarSegment {
                text: format!("${:.2}", snapshot.estimated_cost_usd),
                color: neutral,
            });
        }

        // Git 分支
        if let Some(branch) = &snapshot.git_branch {
            segments.push(StatusBarSegment {
                text: branch.clone(),
                color: neutral,
            });
        }

        if let Some(hint) = &snapshot.route_hint {
            segments.push(StatusBarSegment {
                text: hint.clone(),
                color: neutral,
            });
        }

        if snapshot.semantic_on {
            segments.push(StatusBarSegment {
                text: "semantic".to_string(),
                color: neutral,
            });
        }

        if let Some(hint) = &snapshot.patent_case_hint {
            segments.push(StatusBarSegment {
                text: hint.clone(),
                color: neutral,
            });
        }

        if let Some(hint) = &snapshot.flow_hitl_hint {
            segments.push(StatusBarSegment {
                text: hint.clone(),
                color: neutral,
            });
        }

        if let Some(tool) = &snapshot.active_tool {
            segments.push(StatusBarSegment {
                text: format!("tool:{tool}"),
                color: neutral,
            });
        }

        segments
    }
}

struct StatusBarSegment {
    text: String,
    color: Color,
}

impl StatusBarSegment {
    fn display_width(&self) -> u16 {
        let mut w = 0u16;
        for ch in self.text.chars() {
            if ch.is_ascii() {
                w += 1;
            } else {
                w += 2; // CJK 和 emoji 粗略按双宽处理
            }
        }
        w
    }
}

/// 权限模式对应的状态栏颜色（仅危险模式着色）。
fn permission_mode_color(mode: &str) -> Color {
    if mode == "danger-full-access" {
        Color::Red
    } else {
        Color::Reset
    }
}

/// 将 crossterm Color 转换为 ANSI 前景色 escape code + 文本 + reset。
fn color_fg(text: &str, color: Color) -> String {
    let code = match color {
        Color::Black => "30",
        Color::DarkRed => "31",
        Color::DarkGreen => "32",
        Color::DarkYellow => "33",
        Color::DarkBlue => "34",
        Color::DarkMagenta => "35",
        Color::DarkCyan => "36",
        Color::Grey => "37",
        Color::DarkGrey => "90",
        Color::Red => "91",
        Color::Green => "92",
        Color::Yellow => "93",
        Color::Blue => "94",
        Color::Magenta => "95",
        Color::Cyan => "96",
        Color::White => "97",
        Color::Reset => "0",
        Color::Rgb { r, g, b } => {
            return format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m");
        }
        Color::AnsiValue(n) => return format!("\x1b[38;5;{n}m{text}\x1b[0m"),
    };
    format!("\x1b[{code}m{text}\x1b[0m")
}

/// 将 token 数量格式化为人类可读形式。
#[allow(clippy::cast_precision_loss)]
fn format_tokens(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}m", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}k", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_contains_all_fields() {
        let bar = StatusBar {
            terminal_width: 200,
        };
        let snapshot = StatusBarSnapshot {
            model: "claude-opus-4-6".to_string(),
            permission_mode: "danger-full-access".to_string(),
            session_id: "abc123def456".to_string(),
            cumulative_input_tokens: 12_500,
            cumulative_output_tokens: 8_200,
            estimated_cost_usd: 0.42,
            git_branch: Some("main".to_string()),
            thinking: false,
            ..StatusBarSnapshot::default()
        };
        let rendered = bar.render(&snapshot);
        assert!(rendered.contains("claude-opus-4-6"));
        assert!(rendered.contains("danger-full-access"));
        assert!(rendered.contains("abc123de"));
        assert!(rendered.contains("12.5k in"));
        assert!(rendered.contains("8.2k out"));
        assert!(rendered.contains("$0.42"));
        assert!(rendered.contains("main"));
    }

    #[test]
    fn render_truncates_long_content() {
        let bar = StatusBar { terminal_width: 40 };
        let snapshot = StatusBarSnapshot {
            model: "claude-opus-4-6-very-long-model-name".to_string(),
            permission_mode: "danger-full-access".to_string(),
            session_id: "abc123".to_string(),
            cumulative_input_tokens: 0,
            cumulative_output_tokens: 0,
            estimated_cost_usd: 0.0,
            git_branch: Some("feature/very-long-branch-name".to_string()),
            thinking: false,
            ..StatusBarSnapshot::default()
        };
        let rendered = bar.render(&snapshot);
        assert!(!rendered.contains("very-long-branch"));
    }

    #[test]
    fn thinking_indicator_shows_in_model() {
        let bar = StatusBar {
            terminal_width: 200,
        };
        let snapshot = StatusBarSnapshot {
            model: "claude-opus-4-6".to_string(),
            permission_mode: "danger-full-access".to_string(),
            session_id: "abc".to_string(),
            thinking: true,
            ..StatusBarSnapshot::default()
        };
        let rendered = bar.render(&snapshot);
        assert!(rendered.contains("⏳ claude-opus-4-6"));
    }

    #[test]
    fn format_tokens_various() {
        assert_eq!(format_tokens(0), "0");
        assert_eq!(format_tokens(999), "999");
        assert_eq!(format_tokens(1_000), "1.0k");
        assert_eq!(format_tokens(12_500), "12.5k");
        assert_eq!(format_tokens(1_500_000), "1.5m");
    }

    #[test]
    fn danger_full_access_renders_red() {
        let bar = StatusBar {
            terminal_width: 200,
        };
        let snapshot = StatusBarSnapshot {
            model: "deepseek-v4-pro".to_string(),
            permission_mode: "danger-full-access".to_string(),
            ..StatusBarSnapshot::default()
        };
        let rendered = bar.render(&snapshot);
        assert!(rendered.contains("\x1b[91m"));
        assert!(rendered.contains("danger-full-access"));
    }

    #[test]
    fn non_warning_segments_use_neutral_color() {
        let bar = StatusBar {
            terminal_width: 200,
        };
        let snapshot = StatusBarSnapshot {
            model: "deepseek-v4-pro".to_string(),
            permission_mode: "read-only".to_string(),
            semantic_on: true,
            ..StatusBarSnapshot::default()
        };
        let rendered = bar.render(&snapshot);
        assert!(!rendered.contains("\x1b[96m"), "model should not be cyan");
        assert!(
            !rendered.contains("\x1b[92m"),
            "semantic should not be green"
        );
        assert!(
            !rendered.contains("\x1b[91m"),
            "read-only should not be red"
        );
    }

    #[test]
    fn git_branch_renders_without_leaf_emoji() {
        let bar = StatusBar {
            terminal_width: 200,
        };
        let snapshot = StatusBarSnapshot {
            model: "m".to_string(),
            git_branch: Some("main".to_string()),
            ..StatusBarSnapshot::default()
        };
        let rendered = bar.render(&snapshot);
        assert!(rendered.contains("main"));
        assert!(!rendered.contains('\u{1f33f}'));
    }

    #[test]
    fn snapshot_default_is_blank() {
        let snap = StatusBarSnapshot::default();
        assert!(snap.model.is_empty());
        assert!(!snap.thinking);
        assert!(snap.git_branch.is_none());
    }
}
