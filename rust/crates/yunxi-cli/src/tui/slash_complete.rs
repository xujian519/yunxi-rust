//! TUI 斜杠命令补全。

use crate::slash_complete_shared::slash_line_completions;

/// 斜杠命令补全菜单状态。
#[derive(Debug, Clone)]
pub(crate) struct SlashCompletion {
    pub matches: Vec<(String, String)>,
    pub selected: usize,
}

impl SlashCompletion {
    pub(crate) fn refresh(input: &str, cursor_at_end: bool) -> Option<Self> {
        let matches = slash_line_completions(input, cursor_at_end);
        if matches.is_empty() {
            return None;
        }

        Some(Self {
            matches,
            selected: 0,
        })
    }

    pub(crate) fn move_up(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.selected = (self.selected + self.matches.len() - 1) % self.matches.len();
    }

    pub(crate) fn move_down(&mut self) {
        if self.matches.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.matches.len();
    }

    pub(crate) fn selected_replacement(&self) -> Option<&str> {
        self.matches
            .get(self.selected)
            .map(|(_, replacement)| replacement.as_str())
    }

    pub(crate) fn hint_line(&self) -> String {
        use crate::tui::ui_palette::{input_bold, input_faint};
        let (display, _) = &self.matches[self.selected];
        format!(
            "{} · {} · {}",
            input_faint("Tab 应用"),
            input_faint("↑↓ 选择"),
            input_bold(display)
        )
    }

    /// 在输入区上方绘制候选列表（最多 6 行）。
    pub(crate) fn render_menu_lines(&self, width: usize) -> Vec<String> {
        use crate::tui::frame::truncate_ansi_to_width;
        use crate::tui::ui_palette::{
            input_completion_item, input_completion_selected, input_line_padded,
        };

        self.matches
            .iter()
            .take(6)
            .enumerate()
            .map(|(index, (display, _))| {
                let line = if index == self.selected {
                    input_line_padded(&input_completion_selected(&format!("▸ {display}")), width)
                } else {
                    input_line_padded(
                        &truncate_ansi_to_width(
                            &input_completion_item(&format!("  {display}")),
                            width,
                        ),
                        width,
                    )
                };
                line
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_help_prefix() {
        let menu = SlashCompletion::refresh("/he", true).expect("matches");
        assert!(menu.matches.iter().any(|(_, r)| r == "/help"));
    }

    #[test]
    fn matches_model_argument() {
        let menu = SlashCompletion::refresh("/model deep", true).expect("matches");
        assert!(menu.matches.iter().any(|(d, _)| d.contains("deepseek")));
    }

    #[test]
    fn ignores_non_slash_input() {
        assert!(SlashCompletion::refresh("hello", true).is_none());
    }
}
