//! 交互式会话选择覆盖层（支持模糊筛选）。

use crate::session_mgr::ManagedSessionSummary;
use crate::tui::layout::Rect;

/// 会话选择器状态。
#[derive(Debug, Clone)]
pub(crate) struct SessionPicker {
    sessions: Vec<ManagedSessionSummary>,
    /// 当前可见会话在 `sessions` 中的下标。
    visible: Vec<usize>,
    filter: String,
    /// 在 `visible` 中的选中下标。
    selected_visible: usize,
    active_session_id: String,
}

impl SessionPicker {
    pub(crate) fn new(sessions: Vec<ManagedSessionSummary>, active_session_id: String) -> Self {
        let mut picker = Self {
            sessions,
            visible: Vec::new(),
            filter: String::new(),
            selected_visible: 0,
            active_session_id,
        };
        picker.rebuild_visible();
        picker
    }

    pub(crate) fn filter(&self) -> &str {
        &self.filter
    }

    pub(crate) fn push_filter_char(&mut self, ch: char) {
        if !ch.is_control() {
            self.filter.push(ch);
            self.rebuild_visible();
        }
    }

    pub(crate) fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.rebuild_visible();
    }

    pub(crate) fn clear_filter(&mut self) {
        self.filter.clear();
        self.rebuild_visible();
    }

    pub(crate) fn move_up(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        self.selected_visible =
            (self.selected_visible + self.visible.len() - 1) % self.visible.len();
    }

    pub(crate) fn move_down(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        self.selected_visible = (self.selected_visible + 1) % self.visible.len();
    }

    pub(crate) fn selected_session(&self) -> Option<&ManagedSessionSummary> {
        let index = *self.visible.get(self.selected_visible)?;
        self.sessions.get(index)
    }

    fn rebuild_visible(&mut self) {
        let query = self.filter.trim().to_lowercase();
        let mut ranked: Vec<(u32, usize)> = self
            .sessions
            .iter()
            .enumerate()
            .filter_map(|(index, session)| {
                session_match_score(&query, session).map(|score| (score, index))
            })
            .collect();
        ranked.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        self.visible = ranked.into_iter().map(|(_, index)| index).collect();
        if self.selected_visible >= self.visible.len() {
            self.selected_visible = 0;
        }
    }

    pub(crate) fn render(&self, area: Rect) -> String {
        if !area.is_valid() {
            return String::new();
        }

        let width = area.width as usize;
        let filter_hint = if self.filter.is_empty() {
            "\x1b[2m  输入筛选会话 ID/路径 · ↑↓ 选择 · Enter 切换 · Esc 取消\x1b[0m".to_string()
        } else {
            format!(
                "\x1b[2m  筛选:\x1b[0m \x1b[1m{}\x1b[0m \x1b[2m(Backspace 删除 · Esc 清空)\x1b[0m",
                truncate_id(&self.filter, width.saturating_sub(20))
            )
        };

        let mut lines = vec![
            "\x1b[1;38;5;183m选择会话\x1b[0m".to_string(),
            filter_hint,
            String::new(),
        ];

        if self.sessions.is_empty() {
            lines.push("  \x1b[2m暂无已保存会话\x1b[0m".to_string());
        } else if self.visible.is_empty() {
            lines.push("  \x1b[2m无匹配会话\x1b[0m".to_string());
        } else {
            for (visible_index, &session_index) in self.visible.iter().enumerate() {
                let session = &self.sessions[session_index];
                let marker = if session.id == self.active_session_id {
                    "●"
                } else {
                    "○"
                };
                let cursor = if visible_index == self.selected_visible {
                    "\x1b[1;36m▸\x1b[0m"
                } else {
                    " "
                };
                let id = truncate_id(&session.id, width.saturating_sub(24));
                lines.push(format!(
                    " {cursor} {marker} {id:<18} msgs={:<4} {}",
                    session.message_count,
                    session.path.display()
                ));
            }
        }

        lines.push(String::new());
        lines.push("\x1b[2m— 会话选择器 —\x1b[0m".to_string());

        let visible = area.height as usize;
        let skip = lines.len().saturating_sub(visible) / 3;
        let end = skip.saturating_add(visible).min(lines.len());
        lines[skip..end].join("\n")
    }
}

/// 匹配分数（越小越靠前）；`None` 表示不匹配。
fn session_match_score(query: &str, session: &ManagedSessionSummary) -> Option<u32> {
    if query.is_empty() {
        return Some(0);
    }
    let id = session.id.to_lowercase();
    let path = session.path.to_string_lossy().to_lowercase();
    fuzzy_score(query, &id)
        .or_else(|| fuzzy_score(query, &path))
        .or_else(|| {
            if id.contains(query) || path.contains(query) {
                Some(0)
            } else {
                None
            }
        })
}

/// 子序列模糊匹配分数（连续子串得 0）。
fn fuzzy_score(needle: &str, haystack: &str) -> Option<u32> {
    if needle.is_empty() {
        return Some(0);
    }
    if haystack.contains(needle) {
        return Some(0);
    }
    let mut score = 0u32;
    let mut needle_chars = needle.chars();
    let mut current = needle_chars.next()?;
    let mut last_index = 0usize;
    for (index, ch) in haystack.chars().enumerate() {
        if ch == current {
            if index > last_index {
                score += (index - last_index) as u32;
            }
            last_index = index;
            current = needle_chars.next()?;
            if needle_chars.as_str().is_empty() {
                return Some(score);
            }
        }
    }
    None
}

fn truncate_id(id: &str, max: usize) -> String {
    if id.chars().count() <= max {
        return id.to_string();
    }
    id.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_sessions() -> Vec<ManagedSessionSummary> {
        vec![
            ManagedSessionSummary {
                id: "session-alpha".to_string(),
                path: PathBuf::from("/tmp/session-alpha.json"),
                modified_epoch_secs: 0,
                message_count: 3,
            },
            ManagedSessionSummary {
                id: "session-beta".to_string(),
                path: PathBuf::from("/tmp/session-beta.json"),
                modified_epoch_secs: 0,
                message_count: 1,
            },
        ]
    }

    #[test]
    fn picker_renders_sessions() {
        let picker = SessionPicker::new(sample_sessions(), "session-alpha".to_string());
        let rendered = picker.render(Rect::new(0, 0, 80, 12));
        assert!(rendered.contains("session-alpha"));
        assert!(rendered.contains("选择会话"));
    }

    #[test]
    fn picker_filters_by_id() {
        let mut picker = SessionPicker::new(sample_sessions(), "session-alpha".to_string());
        for ch in "beta".chars() {
            picker.push_filter_char(ch);
        }
        assert_eq!(
            picker.selected_session().map(|s| s.id.as_str()),
            Some("session-beta")
        );
    }

    #[test]
    fn fuzzy_matches_subsequence() {
        assert!(fuzzy_score("sbe", "session-beta").is_some());
        assert_eq!(fuzzy_score("beta", "session-beta"), Some(0));
        assert!(fuzzy_score("zzz", "session-beta").is_none());
    }

    #[test]
    fn picker_fuzzy_finds_subsequence() {
        let mut picker = SessionPicker::new(sample_sessions(), "session-alpha".to_string());
        for ch in "sbe".chars() {
            picker.push_filter_char(ch);
        }
        assert_eq!(
            picker.selected_session().map(|s| s.id.as_str()),
            Some("session-beta")
        );
    }
}
