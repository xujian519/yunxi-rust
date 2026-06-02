use std::collections::HashMap;

use crate::tui::layout::Rect;
use crate::tui::plugin::ui::{PluginLayout, SidebarPosition as UiSidebarPosition};

pub struct PluginLayoutManager {
    layouts: HashMap<String, PluginLayout>,
    order: Vec<String>,
}

impl PluginLayoutManager {
    pub fn new() -> Self {
        Self {
            layouts: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn register_layout(&mut self, plugin_id: String, layout: PluginLayout) -> bool {
        if self.layouts.contains_key(&plugin_id) {
            return false;
        }
        self.layouts.insert(plugin_id.clone(), layout);
        self.order.push(plugin_id);
        true
    }

    pub fn unregister_layout(&mut self, plugin_id: &str) -> bool {
        if self.layouts.remove(plugin_id).is_some() {
            self.order.retain(|id| id != plugin_id);
            true
        } else {
            false
        }
    }

    pub fn get_layout(&self, plugin_id: &str) -> Option<&PluginLayout> {
        self.layouts.get(plugin_id)
    }

    pub fn list_layouts(&self) -> Vec<&str> {
        self.order.iter().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.layouts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.layouts.is_empty()
    }

    pub fn compute_sidebar_layout(
        &self,
        _sidebar_width: u16,
        total_width: u16,
        total_height: u16,
        offset_y: u16,
    ) -> Rect {
        let left_plugins: Vec<_> = self
            .order
            .iter()
            .filter(|id| {
                self.layouts
                    .get(id.as_str())
                    .map(|l| {
                        matches!(
                            l,
                            PluginLayout::Sidebar {
                                position: UiSidebarPosition::Left,
                                ..
                            }
                        )
                    })
                    .unwrap_or(false)
            })
            .collect();

        let right_plugins: Vec<_> = self
            .order
            .iter()
            .filter(|id| {
                self.layouts
                    .get(id.as_str())
                    .map(|l| {
                        matches!(
                            l,
                            PluginLayout::Sidebar {
                                position: UiSidebarPosition::Right,
                                ..
                            }
                        )
                    })
                    .unwrap_or(false)
            })
            .collect();

        let left_width: u16 = left_plugins
            .iter()
            .filter_map(|id| {
                self.layouts.get(id.as_str()).and_then(|l| match l {
                    PluginLayout::Sidebar { width, .. } => Some(*width),
                    _ => None,
                })
            })
            .sum();

        let right_width: u16 = right_plugins
            .iter()
            .filter_map(|id| {
                self.layouts.get(id.as_str()).and_then(|l| match l {
                    PluginLayout::Sidebar { width, .. } => Some(*width),
                    _ => None,
                })
            })
            .sum();

        let used_width = left_width.saturating_add(right_width);
        let remaining_width = total_width.saturating_sub(used_width);
        let sidebar_x = left_width;
        let sidebar_y = offset_y;

        Rect::new(
            sidebar_x,
            sidebar_y,
            remaining_width,
            total_height.saturating_sub(offset_y),
        )
    }

    pub fn compute_bottom_panel_layout(
        &self,
        total_width: u16,
        total_height: u16,
        _offset_y: u16,
    ) -> Rect {
        let bottom_height: u16 = self
            .order
            .iter()
            .filter_map(|id| {
                self.layouts.get(id.as_str()).and_then(|l| match l {
                    PluginLayout::BottomPanel { height } => Some(*height),
                    _ => None,
                })
            })
            .sum();

        let bottom_y = total_height.saturating_sub(bottom_height);

        Rect::new(0, bottom_y, total_width, bottom_height)
    }
}

impl Default for PluginLayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_unregister() {
        let mut manager = PluginLayoutManager::new();

        assert!(manager.register_layout(
            "plugin1".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20,
            }
        ));

        assert!(!manager.register_layout(
            "plugin1".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20,
            }
        ));

        assert!(manager.unregister_layout("plugin1"));

        assert!(!manager.unregister_layout("plugin1"));
    }

    #[test]
    fn test_get_layout() {
        let mut manager = PluginLayoutManager::new();
        manager.register_layout(
            "plugin1".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20,
            },
        );

        let layout = manager.get_layout("plugin1");
        assert!(layout.is_some());
        assert!(matches!(
            layout.unwrap(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20
            }
        ));
    }

    #[test]
    fn test_list_layouts() {
        let mut manager = PluginLayoutManager::new();
        manager.register_layout(
            "plugin1".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20,
            },
        );
        manager.register_layout(
            "plugin2".to_string(),
            PluginLayout::BottomPanel { height: 10 },
        );

        let layouts = manager.list_layouts();
        assert_eq!(layouts, vec!["plugin1", "plugin2"]);
    }

    #[test]
    fn test_count() {
        let mut manager = PluginLayoutManager::new();
        assert_eq!(manager.count(), 0);

        manager.register_layout(
            "plugin1".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20,
            },
        );
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut manager = PluginLayoutManager::new();
        assert!(manager.is_empty());

        manager.register_layout(
            "plugin1".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20,
            },
        );
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_compute_sidebar_layout() {
        let mut manager = PluginLayoutManager::new();
        manager.register_layout(
            "left_plugin".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Left,
                width: 20,
            },
        );
        manager.register_layout(
            "right_plugin".to_string(),
            PluginLayout::Sidebar {
                position: UiSidebarPosition::Right,
                width: 15,
            },
        );

        let rect = manager.compute_sidebar_layout(20, 100, 50, 1);
        assert_eq!(rect.x, 20);
        assert_eq!(rect.y, 1);
        assert_eq!(rect.width, 65);
        assert_eq!(rect.height, 49);
    }

    #[test]
    fn test_compute_bottom_panel_layout() {
        let mut manager = PluginLayoutManager::new();
        manager.register_layout(
            "bottom_plugin".to_string(),
            PluginLayout::BottomPanel { height: 10 },
        );

        let rect = manager.compute_bottom_panel_layout(100, 50, 1);
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 40);
        assert_eq!(rect.width, 100);
        assert_eq!(rect.height, 10);
    }
}
