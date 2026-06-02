use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuItemType {
    Command {
        command: String,
        description: String,
        shortcut: Option<String>,
    },
    Submenu {
        label: String,
        items: Vec<MenuItem>,
    },
    Separator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    pub id: String,
    pub item_type: MenuItemType,
    pub group: Option<String>,
    pub enabled: bool,
    pub visible: bool,
}

impl MenuItem {
    pub fn command(
        id: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            item_type: MenuItemType::Command {
                command: command.into(),
                description: description.into(),
                shortcut: None,
            },
            group: None,
            enabled: true,
            visible: true,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        if let MenuItemType::Command { shortcut: s, .. } = &mut self.item_type {
            *s = Some(shortcut.into());
        }
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn submenu(id: impl Into<String>, label: impl Into<String>, items: Vec<MenuItem>) -> Self {
        Self {
            id: id.into(),
            item_type: MenuItemType::Submenu {
                label: label.into(),
                items,
            },
            group: None,
            enabled: true,
            visible: true,
        }
    }

    pub fn separator(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            item_type: MenuItemType::Separator,
            group: None,
            enabled: true,
            visible: true,
        }
    }
}

pub struct PluginMenuManager {
    items: HashMap<String, MenuItem>,
    order: Vec<String>,
    groups: Vec<String>,
}

impl PluginMenuManager {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            order: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn add_menu_item(&mut self, item: MenuItem) -> bool {
        let id = item.id.clone();
        if self.items.contains_key(&item.id) {
            return false;
        }

        if let Some(ref group) = item.group {
            if !self.groups.contains(group) {
                self.groups.push(group.clone());
            }
        }

        self.items.insert(item.id.clone(), item);
        self.order.push(id);
        true
    }

    pub fn remove_menu_item(&mut self, id: &str) -> bool {
        if self.items.remove(id).is_some() {
            self.order.retain(|item_id| item_id != id);
            self.update_groups();
            true
        } else {
            false
        }
    }

    fn update_groups(&mut self) {
        let active_groups: std::collections::HashSet<String> = self
            .items
            .values()
            .filter_map(|item| item.group.as_ref())
            .cloned()
            .collect();

        self.groups.retain(|g| active_groups.contains(g));
    }

    pub fn get_item(&self, id: &str) -> Option<&MenuItem> {
        self.items.get(id)
    }

    pub fn update_item_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(item) = self.items.get_mut(id) {
            item.enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn update_item_visible(&mut self, id: &str, visible: bool) -> bool {
        if let Some(item) = self.items.get_mut(id) {
            item.visible = visible;
            true
        } else {
            false
        }
    }

    pub fn list_items(&self) -> Vec<&MenuItem> {
        self.order
            .iter()
            .filter_map(|id| self.items.get(id))
            .filter(|item| item.visible)
            .collect()
    }

    pub fn list_items_by_group(&self, group: &str) -> Vec<&MenuItem> {
        self.order
            .iter()
            .filter_map(|id| self.items.get(id))
            .filter(|item| item.visible && item.group.as_deref() == Some(group))
            .collect()
    }

    pub fn list_enabled_items(&self) -> Vec<&MenuItem> {
        self.order
            .iter()
            .filter_map(|id| self.items.get(id))
            .filter(|item| item.visible && item.enabled)
            .collect()
    }

    pub fn list_groups(&self) -> Vec<&str> {
        self.groups.iter().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for PluginMenuManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_menu_item() {
        let mut manager = PluginMenuManager::new();

        let item = MenuItem::command("test1", "TestCommand", "Test description");
        assert!(manager.add_menu_item(item));

        assert!(!manager.add_menu_item(MenuItem::command(
            "test1",
            "TestCommand2",
            "Test description 2",
        )));

        assert!(manager.remove_menu_item("test1"));
        assert!(!manager.remove_menu_item("test1"));
    }

    #[test]
    fn test_command_item_builder() {
        let item = MenuItem::command("test", "TestCommand", "Test description")
            .with_shortcut("Ctrl+T")
            .with_group("Test Group")
            .with_enabled(false)
            .with_visible(false);

        assert_eq!(item.id, "test");
        assert_eq!(item.group, Some("Test Group".to_string()));
        assert!(!item.enabled);
        assert!(!item.visible);

        if let MenuItemType::Command { shortcut, .. } = &item.item_type {
            assert_eq!(shortcut, &Some("Ctrl+T".to_string()));
        } else {
            panic!("Expected Command item type");
        }
    }

    #[test]
    fn test_submenu_item() {
        let item = MenuItem::submenu(
            "submenu1",
            "Submenu Label",
            vec![
                MenuItem::command("sub1", "SubCommand1", "Sub command 1"),
                MenuItem::separator("sep1"),
            ],
        );

        assert_eq!(item.id, "submenu1");
        if let MenuItemType::Submenu { label, items } = &item.item_type {
            assert_eq!(label, "Submenu Label");
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected Submenu item type");
        }
    }

    #[test]
    fn test_separator_item() {
        let item = MenuItem::separator("sep1");
        assert_eq!(item.id, "sep1");
        assert!(matches!(item.item_type, MenuItemType::Separator));
    }

    #[test]
    fn test_update_item_enabled() {
        let mut manager = PluginMenuManager::new();
        manager.add_menu_item(MenuItem::command("test1", "TestCommand", "Test"));

        assert!(manager.update_item_enabled("test1", false));
        assert!(!manager.get_item("test1").unwrap().enabled);
    }

    #[test]
    fn test_update_item_visible() {
        let mut manager = PluginMenuManager::new();
        manager.add_menu_item(MenuItem::command("test1", "TestCommand", "Test"));

        assert!(manager.update_item_visible("test1", false));
        assert!(!manager.get_item("test1").unwrap().visible);
    }

    #[test]
    fn test_list_items() {
        let mut manager = PluginMenuManager::new();
        manager.add_menu_item(MenuItem::command("test1", "Test1", "Test 1"));
        manager.add_menu_item(MenuItem::command("test2", "Test2", "Test 2"));
        manager.update_item_visible("test2", false);

        let items = manager.list_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "test1");
    }

    #[test]
    fn test_list_items_by_group() {
        let mut manager = PluginMenuManager::new();
        manager.add_menu_item(MenuItem::command("test1", "Test1", "Test 1").with_group("Group1"));
        manager.add_menu_item(MenuItem::command("test2", "Test2", "Test 2").with_group("Group2"));

        let group1_items = manager.list_items_by_group("Group1");
        assert_eq!(group1_items.len(), 1);
        assert_eq!(group1_items[0].id, "test1");
    }

    #[test]
    fn test_list_enabled_items() {
        let mut manager = PluginMenuManager::new();
        manager.add_menu_item(MenuItem::command("test1", "Test1", "Test 1"));
        manager.add_menu_item(MenuItem::command("test2", "Test2", "Test 2"));
        manager.update_item_enabled("test2", false);

        let enabled_items = manager.list_enabled_items();
        assert_eq!(enabled_items.len(), 1);
        assert_eq!(enabled_items[0].id, "test1");
    }

    #[test]
    fn test_groups_update() {
        let mut manager = PluginMenuManager::new();
        manager.add_menu_item(MenuItem::command("test1", "Test1", "Test 1").with_group("Group1"));
        manager.add_menu_item(MenuItem::command("test2", "Test2", "Test 2").with_group("Group1"));

        assert_eq!(manager.list_groups(), vec!["Group1"]);

        manager.remove_menu_item("test1");
        assert_eq!(manager.list_groups(), vec!["Group1"]);

        manager.remove_menu_item("test2");
        assert!(manager.list_groups().is_empty());
    }
}
