#![allow(dead_code)]

use std::time::Instant;

/// 工具调用时间线条目。
#[derive(Debug, Clone)]
pub(crate) struct TimelineEntry {
    pub tool_name: String,
    pub started_at: Instant,
    pub duration: Option<std::time::Duration>,
    pub is_error: bool,
}

impl TimelineEntry {
    /// 标记完成并记录耗时。
    pub fn finish(&mut self, is_error: bool) {
        self.duration = Some(self.started_at.elapsed());
        self.is_error = is_error;
    }

    /// 格式化为单行摘要。
    pub fn render_summary(&self) -> String {
        let status = if self.is_error { "✘" } else { "✓" };
        let time = self
            .duration
            .map_or_else(|| "...".to_string(), |d| format!("{:.1}s", d.as_secs_f64()));
        format!("{} {} ({})", status, self.tool_name, time)
    }
}

/// 工具输出折叠策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputFold {
    /// 输出较短，无需折叠。
    Expanded,
    /// 输出超过阈值，默认折叠。
    Collapsed { total_lines: usize },
}

impl OutputFold {
    /// 根据行数决定折叠策略。超过 `threshold` 行则折叠。
    pub(crate) fn from_line_count(lines: usize, threshold: usize) -> Self {
        if lines > threshold {
            Self::Collapsed { total_lines: lines }
        } else {
            Self::Expanded
        }
    }

    /// 渲染折叠提示。
    pub(crate) fn render_fold_hint(&self) -> Option<String> {
        match self {
            Self::Expanded => None,
            Self::Collapsed { total_lines } => Some(format!("  ▸ 展开输出 ({total_lines} 行)")),
        }
    }
}

/// 工具时间线——记录一轮对话中所有工具调用的耗时和状态。
#[derive(Debug, Default)]
pub(crate) struct ToolTimeline {
    entries: Vec<TimelineEntry>,
}

impl ToolTimeline {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// 记录一个新的工具调用开始。
    pub(crate) fn start(&mut self, tool_name: &str) {
        self.entries.push(TimelineEntry {
            tool_name: tool_name.to_string(),
            started_at: Instant::now(),
            duration: None,
            is_error: false,
        });
    }

    /// 标记最后一个工具调用完成。
    pub(crate) fn finish_last(&mut self, is_error: bool) {
        if let Some(entry) = self.entries.last_mut() {
            entry.finish(is_error);
        }
    }

    /// 渲染完整时间线摘要。
    pub(crate) fn render_summary(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        let count = self.entries.len();
        let total_time: f64 = self
            .entries
            .iter()
            .filter_map(|e| e.duration.map(|d| d.as_secs_f64()))
            .sum();
        let summaries: Vec<String> = self
            .entries
            .iter()
            .map(TimelineEntry::render_summary)
            .collect();
        format!(
            "\x1b[2m🔧 {} ({count} 个工具, {total_time:.1}s)\x1b[0m",
            summaries.join(" → ")
        )
    }

    /// 清除当前时间线（每轮对话开始时调用）。
    pub(crate) fn reset(&mut self) {
        self.entries.clear();
    }
}

/// Diff 行类型及渲染。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiffLineKind {
    Context,
    Removal,
    Addition,
    Header,
}

/// 渲染一行 Diff 输出为彩色 ANSI。
pub(crate) fn render_diff_line(line: &str) -> String {
    let kind = if line.starts_with("@@") {
        DiffLineKind::Header
    } else if line.starts_with('-') && !line.starts_with("--- ") {
        DiffLineKind::Removal
    } else if line.starts_with('+') && !line.starts_with("+++ ") {
        DiffLineKind::Addition
    } else {
        DiffLineKind::Context
    };

    match kind {
        DiffLineKind::Removal => format!("\x1b[31m{line}\x1b[0m"),
        DiffLineKind::Addition => format!("\x1b[32m{line}\x1b[0m"),
        DiffLineKind::Header => format!("\x1b[36m{line}\x1b[0m"),
        DiffLineKind::Context => line.to_string(),
    }
}

/// 将完整 diff 文本渲染为彩色输出。
pub(crate) fn render_colored_diff(diff: &str) -> String {
    diff.lines()
        .map(render_diff_line)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_fold_short_output_is_expanded() {
        let fold = OutputFold::from_line_count(5, 10);
        assert_eq!(fold, OutputFold::Expanded);
        assert!(fold.render_fold_hint().is_none());
    }

    #[test]
    fn output_fold_long_output_is_collapsed() {
        let fold = OutputFold::from_line_count(23, 10);
        assert_eq!(fold, OutputFold::Collapsed { total_lines: 23 });
        let hint = fold.render_fold_hint().expect("should have hint");
        assert!(hint.contains("23 行"));
    }

    #[test]
    fn timeline_renders_summary() {
        let mut timeline = ToolTimeline::new();
        timeline.start("bash");
        timeline.finish_last(false);
        timeline.start("read_file");
        timeline.finish_last(false);
        let summary = timeline.render_summary();
        assert!(summary.contains("bash"));
        assert!(summary.contains("read_file"));
        assert!(summary.contains("2 个工具"));
    }

    #[test]
    fn diff_removal_is_red() {
        let rendered = render_diff_line("- removed line");
        assert!(rendered.contains("\x1b[31m"));
    }

    #[test]
    fn diff_addition_is_green() {
        let rendered = render_diff_line("+ added line");
        assert!(rendered.contains("\x1b[32m"));
    }

    #[test]
    fn diff_header_is_cyan() {
        let rendered = render_diff_line("@@ -1,3 +1,4 @@");
        assert!(rendered.contains("\x1b[36m"));
    }

    #[test]
    fn diff_context_is_plain() {
        let rendered = render_diff_line(" unchanged line");
        assert_eq!(rendered, " unchanged line");
    }
}
