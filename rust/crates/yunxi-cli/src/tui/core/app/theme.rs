//! Theme-related action handling (extracted from dispatch_action).

use crate::tui::core::action::Action;

use super::App;

impl App {
    /// 处理主题相关 Action，返回 true 表示已处理。
    pub(crate) fn dispatch_theme_action(&mut self, action: &Action) -> bool {
        match action {
            Action::ToggleDarkMode => {
                self.cycle_theme();
                true
            }
            Action::SwitchTheme(name) => {
                self.set_theme_by_name(name);
                true
            }
            _ => false,
        }
    }
}
