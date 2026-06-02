use crossterm::style::Color;

pub struct PluginStatusBarManager {
    indicators: Vec<StatusIndicator>,
}

impl PluginStatusBarManager {
    pub fn new() -> Self {
        Self {
            indicators: Vec::new(),
        }
    }

    pub fn add_status_indicator(&mut self, indicator: StatusIndicator) -> bool {
        if self.indicators.iter().any(|i| i.id == indicator.id) {
            return false;
        }
        self.indicators.push(indicator);
        true
    }

    pub fn remove_status_indicator(&mut self, id: &str) -> bool {
        let original_len = self.indicators.len();
        self.indicators.retain(|i| i.id != id);
        self.indicators.len() != original_len
    }

    pub fn get_indicator(&self, id: &str) -> Option<&StatusIndicator> {
        self.indicators.iter().find(|i| i.id == id)
    }

    pub fn update_indicator_text(&mut self, id: &str, text: impl Into<String>) -> bool {
        if let Some(indicator) = self.indicators.iter_mut().find(|i| i.id == id) {
            indicator.text = text.into();
            true
        } else {
            false
        }
    }

    pub fn update_indicator_color(&mut self, id: &str, color: Color) -> bool {
        if let Some(indicator) = self.indicators.iter_mut().find(|i| i.id == id) {
            indicator.color = color;
            true
        } else {
            false
        }
    }

    pub fn update_indicator_visible(&mut self, id: &str, visible: bool) -> bool {
        if let Some(indicator) = self.indicators.iter_mut().find(|i| i.id == id) {
            indicator.visible = visible;
            true
        } else {
            false
        }
    }

    pub fn list_indicators(&self) -> Vec<&StatusIndicator> {
        self.indicators.iter().filter(|i| i.visible).collect()
    }

    pub fn count(&self) -> usize {
        self.indicators.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indicators.is_empty()
    }

    pub fn render_all(&self) -> Vec<String> {
        self.indicators
            .iter()
            .filter(|i| i.visible)
            .map(|i| i.render())
            .collect()
    }
}

impl Default for PluginStatusBarManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct StatusIndicator {
    pub id: String,
    pub text: String,
    pub icon: Option<String>,
    pub color: Color,
    pub visible: bool,
    pub priority: u8,
}

impl StatusIndicator {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            icon: None,
            color: Color::Reset,
            visible: true,
            priority: 0,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn render(&self) -> String {
        let text = if let Some(ref icon) = self.icon {
            format!("{} {}", icon, self.text)
        } else {
            self.text.clone()
        };

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_indicator() {
        let mut manager = PluginStatusBarManager::new();

        let indicator = StatusIndicator::new("test1", "Test Indicator");
        assert!(manager.add_status_indicator(indicator));

        let duplicate = StatusIndicator::new("test1", "Duplicate");
        assert!(!manager.add_status_indicator(duplicate));

        assert!(manager.remove_status_indicator("test1"));
        assert!(!manager.remove_status_indicator("test1"));
    }

    #[test]
    fn test_indicator_builder() {
        let indicator = StatusIndicator::new("test", "Test Text")
            .with_icon("⚡")
            .with_color(Color::Yellow)
            .with_visible(false)
            .with_priority(10);

        assert_eq!(indicator.id, "test");
        assert_eq!(indicator.text, "Test Text");
        assert_eq!(indicator.icon, Some("⚡".to_string()));
        assert_eq!(indicator.color, Color::Yellow);
        assert!(!indicator.visible);
        assert_eq!(indicator.priority, 10);
    }

    #[test]
    fn test_update_indicator_text() {
        let mut manager = PluginStatusBarManager::new();
        manager.add_status_indicator(StatusIndicator::new("test1", "Original"));

        assert!(manager.update_indicator_text("test1", "Updated"));
        assert_eq!(manager.get_indicator("test1").unwrap().text, "Updated");
    }

    #[test]
    fn test_update_indicator_color() {
        let mut manager = PluginStatusBarManager::new();
        manager.add_status_indicator(StatusIndicator::new("test1", "Test"));

        assert!(manager.update_indicator_color("test1", Color::Red));
        assert_eq!(manager.get_indicator("test1").unwrap().color, Color::Red);
    }

    #[test]
    fn test_update_indicator_visible() {
        let mut manager = PluginStatusBarManager::new();
        manager.add_status_indicator(StatusIndicator::new("test1", "Test"));

        assert!(manager.update_indicator_visible("test1", false));
        assert!(!manager.get_indicator("test1").unwrap().visible);
    }

    #[test]
    fn test_list_indicators() {
        let mut manager = PluginStatusBarManager::new();
        manager.add_status_indicator(StatusIndicator::new("test1", "Visible"));
        manager
            .add_status_indicator(StatusIndicator::new("test2", "Invisible").with_visible(false));

        let indicators = manager.list_indicators();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].id, "test1");
    }

    #[test]
    fn test_render_all() {
        let mut manager = PluginStatusBarManager::new();
        manager.add_status_indicator(StatusIndicator::new("test1", "Text1").with_icon("🔥"));
        manager.add_status_indicator(StatusIndicator::new("test2", "Text2"));

        let rendered = manager.render_all();
        assert_eq!(rendered.len(), 2);
        assert!(rendered[0].contains("🔥"));
        assert!(rendered[1].contains("Text2"));
    }

    #[test]
    fn test_render_with_icon() {
        let indicator = StatusIndicator::new("test", "Text").with_icon("⚡");
        assert_eq!(indicator.render(), "⚡ Text");
    }

    #[test]
    fn test_render_without_icon() {
        let indicator = StatusIndicator::new("test", "Text");
        assert_eq!(indicator.render(), "Text");
    }
}
