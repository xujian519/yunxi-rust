use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyContext {
    Global,
    Editor,
    List,
    CommandPalette,
    Modal,
}

impl KeyContext {
    pub fn all() -> &'static [KeyContext] {
        &[
            KeyContext::Global,
            KeyContext::Editor,
            KeyContext::List,
            KeyContext::CommandPalette,
            KeyContext::Modal,
        ]
    }

    pub fn is_global(&self) -> bool {
        matches!(self, KeyContext::Global)
    }

    pub fn is_component(&self) -> bool {
        !self.is_global()
    }

    pub fn name(&self) -> &str {
        match self {
            KeyContext::Global => "global",
            KeyContext::Editor => "editor",
            KeyContext::List => "list",
            KeyContext::CommandPalette => "command_palette",
            KeyContext::Modal => "modal",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "global" => Some(KeyContext::Global),
            "editor" => Some(KeyContext::Editor),
            "list" => Some(KeyContext::List),
            "command_palette" => Some(KeyContext::CommandPalette),
            "modal" => Some(KeyContext::Modal),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            KeyContext::Global => "全局",
            KeyContext::Editor => "编辑器",
            KeyContext::List => "列表",
            KeyContext::CommandPalette => "命令面板",
            KeyContext::Modal => "对话框",
        }
    }
}

impl fmt::Display for KeyContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Default for KeyContext {
    fn default() -> Self {
        KeyContext::Global
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ContextPriority {
    pub current: KeyContext,
    pub fallback: KeyContext,
}

impl ContextPriority {
    pub fn new(current: KeyContext, fallback: KeyContext) -> Self {
        Self { current, fallback }
    }

    pub fn with_current(current: KeyContext) -> Self {
        Self {
            current,
            fallback: KeyContext::Global,
        }
    }

    pub fn global() -> Self {
        Self {
            current: KeyContext::Global,
            fallback: KeyContext::Global,
        }
    }

    pub fn contexts(&self) -> Vec<KeyContext> {
        if self.current == KeyContext::Global {
            vec![KeyContext::Global]
        } else {
            vec![self.current, self.fallback]
        }
    }

    pub fn resolve_priority(&self) -> Vec<KeyContext> {
        let mut contexts = self.contexts();
        contexts.dedup();
        contexts
    }

    pub fn set_current(&mut self, context: KeyContext) {
        self.current = context;
    }

    pub fn set_fallback(&mut self, context: KeyContext) {
        self.fallback = context;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_context_names() {
        assert_eq!(KeyContext::Global.name(), "global");
        assert_eq!(KeyContext::Editor.name(), "editor");
        assert_eq!(KeyContext::List.name(), "list");
    }

    #[test]
    fn test_key_context_from_name() {
        assert_eq!(KeyContext::from_name("global"), Some(KeyContext::Global));
        assert_eq!(KeyContext::from_name("editor"), Some(KeyContext::Editor));
        assert_eq!(KeyContext::from_name("invalid"), None);
    }

    #[test]
    fn test_key_context_display_name() {
        assert_eq!(KeyContext::Global.display_name(), "全局");
        assert_eq!(KeyContext::Editor.display_name(), "编辑器");
    }

    #[test]
    fn test_key_context_is_global() {
        assert!(KeyContext::Global.is_global());
        assert!(!KeyContext::Editor.is_global());
    }

    #[test]
    fn test_key_context_is_component() {
        assert!(!KeyContext::Global.is_component());
        assert!(KeyContext::Editor.is_component());
        assert!(KeyContext::List.is_component());
    }

    #[test]
    fn test_context_priority_new() {
        let priority = ContextPriority::new(KeyContext::Editor, KeyContext::Global);
        assert_eq!(priority.current, KeyContext::Editor);
        assert_eq!(priority.fallback, KeyContext::Global);
    }

    #[test]
    fn test_context_priority_with_current() {
        let priority = ContextPriority::with_current(KeyContext::Editor);
        assert_eq!(priority.current, KeyContext::Editor);
        assert_eq!(priority.fallback, KeyContext::Global);
    }

    #[test]
    fn test_context_priority_global() {
        let priority = ContextPriority::global();
        assert_eq!(priority.current, KeyContext::Global);
        assert_eq!(priority.fallback, KeyContext::Global);
    }

    #[test]
    fn test_context_priority_contexts() {
        let priority = ContextPriority::with_current(KeyContext::Editor);
        let contexts = priority.contexts();
        assert_eq!(contexts, vec![KeyContext::Editor, KeyContext::Global]);
    }

    #[test]
    fn test_context_priority_resolve() {
        let priority = ContextPriority::new(KeyContext::Editor, KeyContext::Global);
        let contexts = priority.resolve_priority();
        assert_eq!(contexts, vec![KeyContext::Editor, KeyContext::Global]);

        let priority = ContextPriority::global();
        let contexts = priority.resolve_priority();
        assert_eq!(contexts, vec![KeyContext::Global]);
    }

    #[test]
    fn test_context_priority_set_current() {
        let mut priority = ContextPriority::global();
        priority.set_current(KeyContext::Editor);
        assert_eq!(priority.current, KeyContext::Editor);
    }

    #[test]
    fn test_context_priority_set_fallback() {
        let mut priority = ContextPriority::with_current(KeyContext::Editor);
        priority.set_fallback(KeyContext::List);
        assert_eq!(priority.fallback, KeyContext::List);
    }
}
