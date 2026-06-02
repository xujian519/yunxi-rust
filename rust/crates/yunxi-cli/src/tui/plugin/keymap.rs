use std::collections::HashMap;

use crate::tui::keymap::{KeyBinding, KeyContext, KeySequence};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginKeybinding {
    pub plugin_id: String,
    pub sequence: KeySequence,
    pub command: String,
    pub description: Option<String>,
    pub context: KeyContext,
}

impl PluginKeybinding {
    pub fn new(
        plugin_id: impl Into<String>,
        sequence: KeySequence,
        command: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            sequence,
            command: command.into(),
            description: None,
            context: KeyContext::Global,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_context(mut self, context: KeyContext) -> Self {
        self.context = context;
        self
    }
}

pub struct PluginKeymapManager {
    bindings: HashMap<String, Vec<PluginKeybinding>>,
    context_stack: Vec<KeyContext>,
}

impl PluginKeymapManager {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            context_stack: vec![KeyContext::Global],
        }
    }

    pub fn register(&mut self, binding: PluginKeybinding) -> bool {
        let plugin_id = &binding.plugin_id;
        let bindings = self.bindings.entry(plugin_id.clone()).or_default();

        if bindings
            .iter()
            .any(|b| b.sequence == binding.sequence && b.context == binding.context)
        {
            return false;
        }

        bindings.push(binding);
        true
    }

    pub fn unregister(&mut self, plugin_id: &str, sequence: &KeySequence) -> bool {
        if let Some(bindings) = self.bindings.get_mut(plugin_id) {
            let original_len = bindings.len();
            bindings.retain(|b| &b.sequence != sequence);
            bindings.len() != original_len
        } else {
            false
        }
    }

    pub fn unregister_plugin(&mut self, plugin_id: &str) -> bool {
        self.bindings.remove(plugin_id).is_some()
    }

    pub fn get_binding(
        &self,
        plugin_id: &str,
        sequence: &KeySequence,
    ) -> Option<&PluginKeybinding> {
        self.bindings
            .get(plugin_id)?
            .iter()
            .find(|b| &b.sequence == sequence)
    }

    pub fn resolve(&self, _key: KeyBinding, sequence: &KeySequence) -> Option<&PluginKeybinding> {
        let current_context = *self.context_stack.last().unwrap_or(&KeyContext::Global);

        for bindings in self.bindings.values() {
            for binding in bindings {
                if binding.sequence == *sequence
                    && (binding.context == current_context || binding.context == KeyContext::Global)
                {
                    return Some(binding);
                }
            }
        }

        None
    }

    pub fn list_bindings(&self, plugin_id: &str) -> Vec<&PluginKeybinding> {
        self.bindings
            .get(plugin_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn list_all_bindings(&self) -> Vec<&PluginKeybinding> {
        self.bindings.values().flat_map(|v| v.iter()).collect()
    }

    pub fn push_context(&mut self, context: KeyContext) {
        self.context_stack.push(context);
    }

    pub fn pop_context(&mut self) -> Option<KeyContext> {
        if self.context_stack.len() > 1 {
            self.context_stack.pop()
        } else {
            None
        }
    }

    pub fn current_context(&self) -> KeyContext {
        *self.context_stack.last().unwrap_or(&KeyContext::Global)
    }

    pub fn clear_context(&mut self) {
        self.context_stack = vec![KeyContext::Global];
    }

    pub fn check_conflicts(&self, binding: &PluginKeybinding) -> Vec<&PluginKeybinding> {
        self.bindings
            .values()
            .flat_map(|v| v.iter())
            .filter(|b| {
                b.sequence == binding.sequence
                    && b.context == binding.context
                    && b.plugin_id != binding.plugin_id
            })
            .collect()
    }
}

impl Default for PluginKeymapManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::keymap::{Key, KeyBinding};

    fn create_test_sequence() -> KeySequence {
        KeySequence::single(KeyBinding::ctrl(Key::Char('p')))
    }

    #[test]
    fn test_register_unregister() {
        let mut manager = PluginKeymapManager::new();
        let seq = create_test_sequence();

        let binding = PluginKeybinding::new("plugin1", seq.clone(), "TestCommand");
        assert!(manager.register(binding));

        let duplicate = PluginKeybinding::new("plugin1", seq.clone(), "TestCommand2");
        assert!(!manager.register(duplicate));

        assert!(manager.unregister("plugin1", &seq));
        assert!(!manager.unregister("plugin1", &seq));
    }

    #[test]
    fn test_unregister_plugin() {
        let mut manager = PluginKeymapManager::new();
        let seq = create_test_sequence();

        manager.register(PluginKeybinding::new("plugin1", seq.clone(), "Command1"));
        manager.register(PluginKeybinding::new(
            "plugin2",
            KeySequence::single(KeyBinding::ctrl(Key::Char('x'))),
            "Command2",
        ));

        assert!(manager.unregister_plugin("plugin1"));
        assert!(!manager.unregister_plugin("plugin1"));

        assert!(manager.list_bindings("plugin1").is_empty());
        assert_eq!(manager.list_bindings("plugin2").len(), 1);
    }

    #[test]
    fn test_keybinding_builder() {
        let seq = create_test_sequence();

        let binding = PluginKeybinding::new("plugin1", seq.clone(), "Command")
            .with_description("Test description")
            .with_context(KeyContext::Editor);

        assert_eq!(binding.plugin_id, "plugin1");
        assert_eq!(binding.command, "Command");
        assert_eq!(binding.description, Some("Test description".to_string()));
        assert_eq!(binding.context, KeyContext::Editor);
    }

    #[test]
    fn test_get_binding() {
        let mut manager = PluginKeymapManager::new();
        let seq = create_test_sequence();

        manager.register(PluginKeybinding::new("plugin1", seq.clone(), "Command"));

        let binding = manager.get_binding("plugin1", &seq);
        assert!(binding.is_some());
        assert_eq!(binding.unwrap().command, "Command");
    }

    #[test]
    fn test_resolve_binding() {
        let mut manager = PluginKeymapManager::new();
        let seq = create_test_sequence();

        manager.register(
            PluginKeybinding::new("plugin1", seq.clone(), "Command")
                .with_context(KeyContext::Global),
        );

        let binding = manager.resolve(KeyBinding::ctrl(Key::Char('p')), &seq);
        assert!(binding.is_some());
    }

    #[test]
    fn test_list_bindings() {
        let mut manager = PluginKeymapManager::new();
        let seq1 = KeySequence::single(KeyBinding::ctrl(Key::Char('p')));
        let seq2 = KeySequence::single(KeyBinding::ctrl(Key::Char('x')));

        manager.register(PluginKeybinding::new("plugin1", seq1, "Command1"));
        manager.register(PluginKeybinding::new("plugin1", seq2, "Command2"));

        let bindings = manager.list_bindings("plugin1");
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_list_all_bindings() {
        let mut manager = PluginKeymapManager::new();

        manager.register(PluginKeybinding::new(
            "plugin1",
            KeySequence::single(KeyBinding::ctrl(Key::Char('p'))),
            "Command1",
        ));
        manager.register(PluginKeybinding::new(
            "plugin2",
            KeySequence::single(KeyBinding::ctrl(Key::Char('x'))),
            "Command2",
        ));

        let all_bindings = manager.list_all_bindings();
        assert_eq!(all_bindings.len(), 2);
    }

    #[test]
    fn test_context_stack() {
        let mut manager = PluginKeymapManager::new();
        assert_eq!(manager.current_context(), KeyContext::Global);

        manager.push_context(KeyContext::Editor);
        assert_eq!(manager.current_context(), KeyContext::Editor);

        manager.pop_context();
        assert_eq!(manager.current_context(), KeyContext::Global);
    }

    #[test]
    fn test_clear_context() {
        let mut manager = PluginKeymapManager::new();

        manager.push_context(KeyContext::Editor);
        manager.push_context(KeyContext::CommandPalette);
        manager.clear_context();

        assert_eq!(manager.current_context(), KeyContext::Global);
    }

    #[test]
    fn test_check_conflicts() {
        let mut manager = PluginKeymapManager::new();
        let seq = create_test_sequence();

        manager.register(PluginKeybinding::new("plugin1", seq.clone(), "Command1"));
        manager.register(PluginKeybinding::new("plugin2", seq.clone(), "Command2"));

        let binding = PluginKeybinding::new("plugin3", seq.clone(), "Command3");
        let conflicts = manager.check_conflicts(&binding);

        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn test_resolve_with_different_contexts() {
        let mut manager = PluginKeymapManager::new();
        let seq = create_test_sequence();

        manager.register(
            PluginKeybinding::new("plugin1", seq.clone(), "GlobalCommand")
                .with_context(KeyContext::Global),
        );

        manager.push_context(KeyContext::Editor);

        let binding = manager.resolve(KeyBinding::ctrl(Key::Char('p')), &seq);
        assert!(binding.is_some());
    }
}
