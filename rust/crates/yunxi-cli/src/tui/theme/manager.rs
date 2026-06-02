use std::path::PathBuf;

use super::{presets::*, ThemeRegistry};
use crate::tui::theme::Theme;

pub struct ThemeManager {
    registry: ThemeRegistry,
    current: Theme,
    config_path: PathBuf,
}

impl ThemeManager {
    pub fn new(registry: ThemeRegistry) -> Self {
        let current = registry.get("default_dark");

        let config_path = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".yunxi")
            .join("theme.toml");

        let mut manager = Self {
            registry,
            current,
            config_path,
        };

        manager.load_config().unwrap_or_else(|e| {
            eprintln!("警告: 无法加载主题配置: {}", e);
        });

        manager
    }

    pub fn set_theme(&mut self, name: &str) {
        let theme = self.registry.get(name);
        self.current = theme;
        self.save_config().unwrap_or_else(|e| {
            eprintln!("警告: 无法保存主题配置: {}", e);
        });
    }

    pub fn get_theme(&self) -> &Theme {
        &self.current
    }

    pub fn toggle_theme(&mut self) {
        let new_theme = if self.current.is_dark {
            Theme::default_light()
        } else {
            Theme::default_dark()
        };
        self.current = new_theme;
        self.save_config().unwrap_or_else(|e| {
            eprintln!("警告: 无法保存主题配置: {}", e);
        });
    }

    pub fn list_presets(&self) -> Vec<String> {
        self.registry
            .themes
            .iter()
            .map(|t| t.name.clone())
            .collect()
    }

    pub fn detect_system_theme() -> Option<bool> {
        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("defaults")
                .args(["read", "-g", "AppleInterfaceStyle"])
                .output();

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Some(stdout.trim() == "Dark")
            } else {
                None
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }

    pub fn set_auto_theme(&mut self) {
        if let Some(is_dark) = Self::detect_system_theme() {
            self.current = if is_dark {
                Theme::default_dark()
            } else {
                Theme::default_light()
            };
            self.save_config().unwrap_or_else(|e| {
                eprintln!("警告: 无法保存主题配置: {}", e);
            });
        }
    }

    fn load_config(&mut self) -> std::io::Result<()> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.config_path)?;
        if let Some(theme_name) = content
            .trim()
            .strip_prefix("theme = \"")
            .and_then(|s| s.strip_suffix("\""))
        {
            self.current = self.registry.get(theme_name);
        }

        Ok(())
    }

    fn save_config(&self) -> std::io::Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = format!("theme = \"{}\"", self.current.name);
        std::fs::write(&self.config_path, content)
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        let mut registry = ThemeRegistry::new();
        register_all_presets(&mut registry);
        Self::new(registry)
    }
}

fn register_all_presets(registry: &mut ThemeRegistry) {
    registry.register(Theme::nord());
    registry.register(Theme::dracula());
    registry.register(Theme::gruvbox());
    registry.register(Theme::solarized_dark());
    registry.register(Theme::solarized_light());
    registry.register(Theme::monokai());
    registry.register(Theme::catppuccin());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> ThemeManager {
        let mut registry = ThemeRegistry::new();
        registry.register(Theme::nord());
        registry.register(Theme::dracula());
        registry.register(Theme::gruvbox());
        registry.register(Theme::solarized_dark());
        registry.register(Theme::solarized_light());
        registry.register(Theme::monokai());
        registry.register(Theme::catppuccin());
        ThemeManager::new(registry)
    }

    #[test]
    fn test_manager_creation() {
        let manager = create_test_manager();
        assert_eq!(manager.get_theme().name, "default_dark");
    }

    #[test]
    fn test_set_theme() {
        let mut manager = create_test_manager();
        manager.set_theme("nord");
        assert_eq!(manager.get_theme().name, "nord");
    }

    #[test]
    fn test_set_theme_unknown() {
        let mut manager = create_test_manager();
        manager.set_theme("unknown_theme");
        assert_eq!(manager.get_theme().name, "default_dark");
    }

    #[test]
    fn test_toggle_theme() {
        let mut manager = create_test_manager();
        assert!(manager.get_theme().is_dark);

        manager.toggle_theme();
        assert!(!manager.get_theme().is_dark);

        manager.toggle_theme();
        assert!(manager.get_theme().is_dark);
    }

    #[test]
    fn test_list_presets() {
        let manager = create_test_manager();
        let presets = manager.list_presets();

        assert!(presets.len() >= 7);
        assert!(presets.contains(&"nord".to_string()));
        assert!(presets.contains(&"dracula".to_string()));
        assert!(presets.contains(&"gruvbox".to_string()));
        assert!(presets.contains(&"solarized_dark".to_string()));
        assert!(presets.contains(&"solarized_light".to_string()));
        assert!(presets.contains(&"monokai".to_string()));
        assert!(presets.contains(&"catppuccin".to_string()));
    }

    #[test]
    fn test_default_manager() {
        let manager = ThemeManager::default();
        assert!(!manager.list_presets().is_empty());
    }
}
