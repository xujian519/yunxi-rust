use std::time::{SystemTime, UNIX_EPOCH};

use crossterm::event::{Event, KeyCode};
use crossterm::style::Color as CrosstermColor;
use ratatui::style::{Color, Style};

use ratatui::{
    layout::Alignment,
    style::Stylize,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::keymap::{Key, KeyBinding, KeySequence};
use crate::tui::layout::Rect as TuiRect;
use crate::tui::plugin::ui::{PluginLayout, PluginUI, SidebarPosition};
use crate::tui::plugin::{
    keymap::PluginKeybinding, layout::PluginLayoutManager, menu::MenuItem,
    status_bar::StatusIndicator, PluginKeymapManager, PluginMenuManager, PluginStatusBarManager,
};

use SidebarPosition as PluginSidebarPosition;

pub struct ExamplePlugin {
    name: String,
    enabled: bool,
    menu_manager: PluginMenuManager,
    status_manager: PluginStatusBarManager,
    keymap_manager: PluginKeymapManager,
    layout_manager: PluginLayoutManager,
    data: Vec<String>,
    selected: usize,
}

impl ExamplePlugin {
    pub fn new(name: impl Into<String>) -> Self {
        let plugin_id = "example_plugin";
        let name = name.into();

        let mut menu_manager = PluginMenuManager::new();
        menu_manager.add_menu_item(
            MenuItem::command("example:refresh", "ExampleRefresh", "刷新示例数据")
                .with_shortcut("Ctrl+E R")
                .with_group("示例插件"),
        );

        menu_manager.add_menu_item(MenuItem::separator("example:sep1"));
        menu_manager.add_menu_item(
            MenuItem::command("example:clear", "ExampleClear", "清空数据").with_group("示例插件"),
        );

        let mut status_manager = PluginStatusBarManager::new();
        status_manager.add_status_indicator(
            StatusIndicator::new("example_status", "示例")
                .with_icon("🎯")
                .with_color(CrosstermColor::Cyan),
        );

        let mut keymap_manager = PluginKeymapManager::new();
        keymap_manager.register(
            PluginKeybinding::new(
                plugin_id,
                KeySequence::single(KeyBinding::ctrl(Key::Char('e'))),
                "ToggleExample",
            )
            .with_description("切换示例插件"),
        );

        let mut layout_manager = PluginLayoutManager::new();
        layout_manager.register_layout(
            plugin_id.to_string(),
            PluginLayout::Sidebar {
                position: PluginSidebarPosition::Right,
                width: 25,
            },
        );

        Self {
            name,
            enabled: true,
            menu_manager,
            status_manager,
            keymap_manager,
            layout_manager,
            data: vec![
                "欢迎使用示例插件".to_string(),
                "按 Ctrl+E 切换启用状态".to_string(),
                "按 R 刷新数据".to_string(),
                "按 C 清空数据".to_string(),
            ],
            selected: 0,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.status_manager
            .update_indicator_visible("example_status", enabled);
    }

    pub fn get_menu_manager(&self) -> &PluginMenuManager {
        &self.menu_manager
    }

    pub fn get_status_manager(&self) -> &PluginStatusBarManager {
        &self.status_manager
    }

    pub fn get_keymap_manager(&self) -> &PluginKeymapManager {
        &self.keymap_manager
    }

    pub fn get_layout_manager(&self) -> &PluginLayoutManager {
        &self.layout_manager
    }

    fn refresh_data(&mut self) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.data = vec![
            "数据已刷新".to_string(),
            format!("刷新时间: {}", timestamp),
            "项目 A: 活跃".to_string(),
            "项目 B: 待处理".to_string(),
            "项目 C: 已完成".to_string(),
        ];
        self.selected = 0;
    }

    fn clear_data(&mut self) {
        self.data.clear();
        self.selected = 0;
    }
}

impl PluginUI for ExamplePlugin {
    fn render(&self, area: TuiRect, frame: &mut Frame) {
        if !self.enabled {
            return;
        }

        let ratatui_area = ratatui::layout::Rect::new(area.x, area.y, area.width, area.height);

        let block = Block::default()
            .title(format!(" {} ", self.name))
            .borders(Borders::ALL)
            .border_style(Style::default().cyan());

        let inner = block.inner(ratatui_area);
        frame.render_widget(block, ratatui_area);

        if self.data.is_empty() {
            let paragraph = Paragraph::new(Text::from(Line::from(Span::styled(
                "无数据",
                Style::default().dim(),
            ))))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
            return;
        }

        let lines: Vec<Line> = self
            .data
            .iter()
            .enumerate()
            .map(|(i, text)| {
                if i == self.selected {
                    Line::from(Span::styled(
                        format!("> {}", text),
                        Style::default().bg(Color::Indexed(17)).fg(Color::White),
                    ))
                } else {
                    Line::from(Span::styled(format!("  {}", text), Style::default()))
                }
            })
            .collect();

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });

        frame.render_widget(paragraph, inner);
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if !self.enabled {
            return false;
        }

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    self.refresh_data();
                    true
                }
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    self.clear_data();
                    true
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.data.is_empty() {
                        self.selected = (self.selected + 1).min(self.data.len() - 1);
                    }
                    true
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn get_layout(&self) -> PluginLayout {
        self.layout_manager
            .get_layout("example_plugin")
            .copied()
            .unwrap_or(PluginLayout::Sidebar {
                position: PluginSidebarPosition::Right,
                width: 25,
            })
    }

    fn get_shortcuts(&self) -> Vec<(KeySequence, String, String)> {
        vec![
            (
                KeySequence::single(KeyBinding::ctrl(Key::Char('e'))),
                "ToggleExample".to_string(),
                "切换示例插件".to_string(),
            ),
            (
                KeySequence::single(KeyBinding::simple(Key::Char('r'))),
                "Refresh".to_string(),
                "刷新数据".to_string(),
            ),
            (
                KeySequence::single(KeyBinding::simple(Key::Char('c'))),
                "Clear".to_string(),
                "清空数据".to_string(),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = ExamplePlugin::new("示例插件");
        assert_eq!(plugin.get_name(), "示例插件");
        assert!(plugin.is_enabled());
    }

    #[test]
    fn test_plugin_enable_disable() {
        let mut plugin = ExamplePlugin::new("示例插件");
        assert!(plugin.is_enabled());

        plugin.set_enabled(false);
        assert!(!plugin.is_enabled());

        plugin.set_enabled(true);
        assert!(plugin.is_enabled());
    }

    #[test]
    fn test_menu_manager() {
        let plugin = ExamplePlugin::new("示例插件");
        let menu = plugin.get_menu_manager();

        assert!(!menu.is_empty());
        assert!(menu.get_item("example:refresh").is_some());
        assert!(menu.get_item("example:clear").is_some());
    }

    #[test]
    fn test_status_manager() {
        let plugin = ExamplePlugin::new("示例插件");
        let status = plugin.get_status_manager();

        assert!(!status.is_empty());
        assert!(status.get_indicator("example_status").is_some());
    }

    #[test]
    fn test_keymap_manager() {
        let plugin = ExamplePlugin::new("示例插件");
        let keymap = plugin.get_keymap_manager();

        let bindings = keymap.list_bindings("example_plugin");
        assert!(!bindings.is_empty());
    }

    #[test]
    fn test_layout_manager() {
        let plugin = ExamplePlugin::new("示例插件");
        let layout = plugin.get_layout_manager();

        assert!(!layout.is_empty());
        let plugin_layout = layout.get_layout("example_plugin");
        assert!(plugin_layout.is_some());
    }

    #[test]
    fn test_get_shortcuts() {
        let plugin = ExamplePlugin::new("示例插件");
        let shortcuts = plugin.get_shortcuts();

        assert!(!shortcuts.is_empty());
        assert_eq!(shortcuts.len(), 3);
    }

    #[test]
    fn test_refresh_data() {
        let mut plugin = ExamplePlugin::new("示例插件");
        let initial_len = plugin.data.len();

        plugin.refresh_data();

        assert!(plugin.data.len() >= initial_len);
        assert!(plugin.data.iter().any(|s| s.contains("刷新时间")));
    }

    #[test]
    fn test_clear_data() {
        let mut plugin = ExamplePlugin::new("示例插件");
        assert!(!plugin.data.is_empty());

        plugin.clear_data();

        assert!(plugin.data.is_empty());
    }

    #[test]
    fn test_navigation() {
        let mut plugin = ExamplePlugin::new("示例插件");
        assert_eq!(plugin.selected, 0);

        let down_event = Event::Key(KeyCode::Down.into());
        plugin.handle_event(&down_event);
        assert_eq!(plugin.selected, 1);

        let up_event = Event::Key(KeyCode::Up.into());
        plugin.handle_event(&up_event);
        assert_eq!(plugin.selected, 0);
    }

    #[test]
    fn test_keyboard_shortcuts() {
        let mut plugin = ExamplePlugin::new("示例插件");

        let refresh_event = Event::Key(KeyCode::Char('r').into());
        assert!(plugin.handle_event(&refresh_event));

        let clear_event = Event::Key(KeyCode::Char('c').into());
        assert!(plugin.handle_event(&clear_event));
        assert!(plugin.data.is_empty());
    }

    #[test]
    fn test_disabled_plugin_ignores_events() {
        let mut plugin = ExamplePlugin::new("示例插件");
        plugin.set_enabled(false);

        let event = Event::Key(KeyCode::Char('r').into());
        assert!(!plugin.handle_event(&event));
    }

    #[test]
    fn test_status_updates_with_enable_state() {
        let mut plugin = ExamplePlugin::new("示例插件");

        plugin.set_enabled(false);
        let status = plugin
            .get_status_manager()
            .get_indicator("example_status")
            .unwrap();
        assert!(!status.visible);

        plugin.set_enabled(true);
        let status = plugin
            .get_status_manager()
            .get_indicator("example_status")
            .unwrap();
        assert!(status.visible);
    }

    #[test]
    fn test_get_layout() {
        let plugin = ExamplePlugin::new("示例插件");
        let layout = plugin.get_layout();

        assert!(matches!(
            layout,
            PluginLayout::Sidebar {
                position: PluginSidebarPosition::Right,
                ..
            }
        ));
    }
}
