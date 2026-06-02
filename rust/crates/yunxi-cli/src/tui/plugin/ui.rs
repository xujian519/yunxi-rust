use crate::tui::keymap::KeySequence;
use crate::tui::layout::Rect;

pub trait PluginUI: Send + Sync {
    fn render(&self, area: Rect, frame: &mut ratatui::Frame);

    fn handle_event(&mut self, event: &crossterm::event::Event) -> bool;

    fn get_layout(&self) -> PluginLayout;

    fn get_shortcuts(&self) -> Vec<(KeySequence, String, String)>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginLayout {
    Sidebar {
        position: SidebarPosition,
        width: u16,
    },
    BottomPanel {
        height: u16,
    },
    Modal,
    Overlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginShortcut {
    pub sequence: KeySequence,
    pub command: String,
    pub description: String,
}

impl PluginShortcut {
    pub fn new(
        sequence: KeySequence,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            sequence,
            command: command.into(),
            description: description.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlugin;

    impl PluginUI for MockPlugin {
        fn render(&self, _area: Rect, _frame: &mut ratatui::Frame) {}

        fn handle_event(&mut self, _event: &crossterm::event::Event) -> bool {
            false
        }

        fn get_layout(&self) -> PluginLayout {
            PluginLayout::Sidebar {
                position: SidebarPosition::Left,
                width: 20,
            }
        }

        fn get_shortcuts(&self) -> Vec<(KeySequence, String, String)> {
            vec![]
        }
    }

    #[test]
    fn test_plugin_layout_variants() {
        let layout1 = PluginLayout::Sidebar {
            position: SidebarPosition::Left,
            width: 20,
        };
        let layout2 = PluginLayout::BottomPanel { height: 10 };
        let layout3 = PluginLayout::Modal;
        let layout4 = PluginLayout::Overlay;

        assert_ne!(layout1, layout2);
        assert_ne!(layout2, layout3);
        assert_ne!(layout3, layout4);
    }

    #[test]
    fn test_plugin_shortcut_creation() {
        use crate::tui::keymap::{Key, KeyBinding};
        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('p')));
        let shortcut =
            PluginShortcut::new(seq.clone(), "PluginCommand", "Plugin command description");

        assert_eq!(shortcut.sequence, seq);
        assert_eq!(shortcut.command, "PluginCommand");
        assert_eq!(shortcut.description, "Plugin command description");
    }

    #[test]
    fn test_sidebar_position_variants() {
        assert_ne!(SidebarPosition::Left, SidebarPosition::Right);
    }
}
