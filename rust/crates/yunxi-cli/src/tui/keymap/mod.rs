mod commands;
mod context;
mod key_sequence;
mod keys;

pub use commands::{Command, CommandRegistry};
pub use context::{ContextPriority, KeyContext};
pub use key_sequence::{KeySequence, SequenceTracker};
pub use keys::{Key, KeyBinding};

use crate::tui::core::action::Action;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub sequence: KeySequence,
    pub command: String,
    pub context: KeyContext,
    pub description: Option<String>,
}

impl Binding {
    pub fn new(sequence: KeySequence, command: impl Into<String>, context: KeyContext) -> Self {
        Self {
            sequence,
            command: command.into(),
            context,
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn display(&self) -> String {
        if let Some(desc) = &self.description {
            format!("{}: {} ({})", self.sequence, self.command, desc)
        } else {
            format!("{}: {}", self.sequence, self.command)
        }
    }
}

pub struct KeyMap {
    bindings: HashMap<KeyContext, HashMap<KeySequence, String>>,
    command_registry: CommandRegistry,
    tracker: SequenceTracker,
    context_priority: ContextPriority,
}

impl KeyMap {
    pub fn new() -> Self {
        let mut keymap = Self {
            bindings: HashMap::new(),
            command_registry: CommandRegistry::new(),
            tracker: SequenceTracker::with_default_timeout(),
            context_priority: ContextPriority::global(),
        };

        keymap.register_default_bindings();
        keymap
    }

    fn register_default_bindings(&mut self) {
        use Key::*;

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('q'))),
            "Quit",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('s'))),
            "SaveSession",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('b'))),
            "ToggleSidebar",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('p'))),
            "ShowCommandPalette",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::simple(Esc)),
            "HideCommandPalette",
            KeyContext::CommandPalette,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('n'))),
            "NewSession",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('d'))),
            "ToggleDarkMode",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('r'))),
            "Refresh",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('h'))),
            "GoBack",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('l'))),
            "GoForward",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('c'))),
            "Copy",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('v'))),
            "Paste",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('z'))),
            "EditorUndo",
            KeyContext::Editor,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl_shift(Char('y'))),
            "EditorRedo",
            KeyContext::Editor,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('?'))),
            "Help",
            KeyContext::Global,
        );

        self.bind(
            KeySequence::new(vec![
                KeyBinding::simple(Char('g')),
                KeyBinding::simple(Char('g')),
            ]),
            "GoToTop",
            KeyContext::List,
        );

        self.bind(
            KeySequence::new(vec![KeyBinding::simple(Char('G'))]),
            "GoToBottom",
            KeyContext::List,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('j'))),
            "NavigateDown",
            KeyContext::List,
        );

        self.bind(
            KeySequence::single(KeyBinding::ctrl(Char('k'))),
            "NavigateUp",
            KeyContext::List,
        );
    }

    pub fn bind(
        &mut self,
        sequence: KeySequence,
        command: impl Into<String>,
        context: KeyContext,
    ) -> &mut Self {
        let context_bindings = self.bindings.entry(context).or_default();
        context_bindings.insert(sequence, command.into());
        self
    }

    pub fn unbind(&mut self, sequence: &KeySequence, context: KeyContext) -> bool {
        if let Some(context_bindings) = self.bindings.get_mut(&context) {
            context_bindings.remove(sequence).is_some()
        } else {
            false
        }
    }

    pub fn resolve(&self, context: KeyContext, sequence: &KeySequence) -> Option<String> {
        let mut priority = self.context_priority.clone();
        priority.set_current(context);
        let contexts = priority.resolve_priority();

        for ctx in contexts {
            if let Some(context_bindings) = self.bindings.get(&ctx) {
                if let Some(command) = context_bindings.get(sequence) {
                    return Some(command.clone());
                }
            }
        }

        None
    }

    pub fn handle_key(&mut self, key: KeyBinding, context: KeyContext) -> Option<Vec<Action>> {
        self.tracker.tick();
        self.tracker.push_key(key);

        let current_seq = self.tracker.current().clone();

        if let Some(command) = self.resolve(context, &current_seq) {
            self.tracker.clear();
            return self.execute_command(&command);
        }

        if !self.has_partial_match(context, &current_seq) {
            self.tracker.clear();
        }

        None
    }

    pub fn has_partial_match(&self, context: KeyContext, sequence: &KeySequence) -> bool {
        let mut priority = self.context_priority.clone();
        priority.set_current(context);
        let contexts = priority.resolve_priority();

        for ctx in contexts {
            if let Some(context_bindings) = self.bindings.get(&ctx) {
                for binding_seq in context_bindings.keys() {
                    if binding_seq.matches_prefix(sequence) && binding_seq != sequence {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn execute_command(&self, command: &str) -> Option<Vec<Action>> {
        self.command_registry.execute(command)
    }

    pub fn register_command(&mut self, command: Command) -> &mut Self {
        self.command_registry.register(command);
        self
    }

    pub fn get_command(&self, name: &str) -> Option<&Command> {
        self.command_registry.get(name)
    }

    pub fn list_commands(&self) -> Vec<&Command> {
        self.command_registry.list_sorted()
    }

    pub fn list_bindings(&self, context: KeyContext) -> Vec<Binding> {
        let mut bindings = Vec::new();

        if let Some(context_bindings) = self.bindings.get(&context) {
            for (sequence, command) in context_bindings {
                let description = self
                    .command_registry
                    .get(command)
                    .map(|c| c.description.clone());
                bindings.push(Binding {
                    sequence: sequence.clone(),
                    command: command.clone(),
                    context,
                    description,
                });
            }
        }

        bindings.sort_by(|a, b| a.sequence.display().cmp(&b.sequence.display()));
        bindings
    }

    pub fn list_all_bindings(&self) -> Vec<Binding> {
        let mut all_bindings = Vec::new();

        for context in KeyContext::all() {
            all_bindings.extend(self.list_bindings(*context));
        }

        all_bindings
    }

    pub fn check_conflicts(&self, sequence: &KeySequence, context: KeyContext) -> Vec<Binding> {
        let mut conflicts = Vec::new();

        if let Some(context_bindings) = self.bindings.get(&context) {
            if let Some(existing_command) = context_bindings.get(sequence) {
                let description = self
                    .command_registry
                    .get(existing_command)
                    .map(|c| c.description.clone());
                conflicts.push(Binding {
                    sequence: sequence.clone(),
                    command: existing_command.clone(),
                    context,
                    description,
                });
            }
        }

        conflicts
    }

    pub fn get_tracker(&self) -> &SequenceTracker {
        &self.tracker
    }

    pub fn get_tracker_mut(&mut self) -> &mut SequenceTracker {
        &mut self.tracker
    }

    pub fn set_context_priority(&mut self, priority: ContextPriority) {
        self.context_priority = priority;
    }

    pub fn get_context_priority(&self) -> &ContextPriority {
        &self.context_priority
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.tracker.set_timeout(timeout);
    }

    pub fn get_timeout(&self) -> Duration {
        *self.tracker.timeout()
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        let config = KeyMapConfig {
            bindings: self
                .list_all_bindings()
                .into_iter()
                .map(|b| BindingConfig {
                    sequence: b.sequence,
                    command: b.command,
                    context: b.context,
                    description: b.description,
                })
                .collect(),
        };
        let json =
            serde_json::to_string_pretty(&config).map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(path, json).map_err(|e| format!("写入文件失败: {}", e))?;
        Ok(())
    }

    pub fn load_from_file(&mut self, path: &Path) -> Result<(), String> {
        let json = fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))?;
        let config: KeyMapConfig =
            serde_json::from_str(&json).map_err(|e| format!("解析失败: {}", e))?;

        for binding in config.bindings {
            self.bind(binding.sequence, binding.command, binding.context);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BindingConfig {
    sequence: KeySequence,
    command: String,
    context: KeyContext,
    description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyMapConfig {
    bindings: Vec<BindingConfig>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keymap_new() {
        let keymap = KeyMap::new();
        assert!(!keymap.list_all_bindings().is_empty());
    }

    #[test]
    fn test_bind_unbind() {
        let mut keymap = KeyMap::new();
        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('t')));
        keymap.bind(seq.clone(), "Test", KeyContext::Global);

        let command = keymap.resolve(KeyContext::Global, &seq);
        assert_eq!(command, Some("Test".to_string()));

        keymap.unbind(&seq, KeyContext::Global);
        let command = keymap.resolve(KeyContext::Global, &seq);
        assert_eq!(command, None);
    }

    #[test]
    fn test_resolve_with_context() {
        let keymap = KeyMap::new();
        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('q')));

        let command = keymap.resolve(KeyContext::Global, &seq);
        assert_eq!(command, Some("Quit".to_string()));
    }

    #[test]
    fn test_resolve_with_fallback() {
        let mut keymap = KeyMap::new();
        keymap.set_context_priority(ContextPriority::with_current(KeyContext::Editor));

        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('q')));
        let command = keymap.resolve(KeyContext::Editor, &seq);
        assert_eq!(command, Some("Quit".to_string()));
    }

    #[test]
    fn test_handle_key() {
        let mut keymap = KeyMap::new();
        let key = KeyBinding::ctrl(Key::Char('q'));

        let actions = keymap.handle_key(key, KeyContext::Global);
        assert!(actions.is_some());
        assert_eq!(actions.unwrap()[0], Action::Quit);
    }

    #[test]
    fn test_handle_key_sequence() {
        let mut keymap = KeyMap::new();
        keymap.bind(
            KeySequence::new(vec![
                KeyBinding::simple(Key::Char('g')),
                KeyBinding::simple(Key::Char('g')),
            ]),
            "GoToTop",
            KeyContext::List,
        );

        let result1 = keymap.handle_key(KeyBinding::simple(Key::Char('g')), KeyContext::List);
        assert!(result1.is_none());

        let result2 = keymap.handle_key(KeyBinding::simple(Key::Char('g')), KeyContext::List);
        assert!(result2.is_some());
    }

    #[test]
    fn test_has_partial_match() {
        let keymap = KeyMap::new();
        let seq = KeySequence::single(KeyBinding::simple(Key::Char('g')));

        assert!(keymap.has_partial_match(KeyContext::List, &seq));
    }

    #[test]
    fn test_list_bindings() {
        let keymap = KeyMap::new();
        let bindings = keymap.list_bindings(KeyContext::Global);
        assert!(!bindings.is_empty());
    }

    #[test]
    fn test_list_all_bindings() {
        let keymap = KeyMap::new();
        let bindings = keymap.list_all_bindings();
        assert!(!bindings.is_empty());
    }

    #[test]
    fn test_check_conflicts() {
        let keymap = KeyMap::new();
        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('q')));

        let conflicts = keymap.check_conflicts(&seq, KeyContext::Global);
        assert!(!conflicts.is_empty());
    }

    #[test]
    fn test_execute_command() {
        let keymap = KeyMap::new();
        let actions = keymap.execute_command("Quit");
        assert!(actions.is_some());
        assert_eq!(actions.unwrap()[0], Action::Quit);
    }

    #[test]
    fn test_list_commands() {
        let keymap = KeyMap::new();
        let commands = keymap.list_commands();
        assert!(!commands.is_empty());
    }

    #[test]
    fn test_tracker_timeout() {
        let mut keymap = KeyMap::new();
        keymap.set_timeout(Duration::from_millis(10));

        keymap.handle_key(KeyBinding::simple(Key::Char('g')), KeyContext::List);
        std::thread::sleep(Duration::from_millis(20));

        keymap.get_tracker_mut().tick();
        assert!(keymap.get_tracker().current().is_empty());
    }

    #[test]
    fn test_tracker_hint() {
        let mut keymap = KeyMap::new();
        keymap.handle_key(KeyBinding::simple(Key::Char('g')), KeyContext::List);

        let hint = keymap.get_tracker().get_hint();
        assert!(hint.is_some());
    }

    #[test]
    fn test_context_priority() {
        let mut keymap = KeyMap::new();
        let priority = ContextPriority::with_current(KeyContext::Editor);
        keymap.set_context_priority(priority);

        assert_eq!(keymap.get_context_priority().current, KeyContext::Editor);
    }

    #[test]
    fn test_save_and_load_keymap() {
        let mut keymap = KeyMap::new();
        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('t')));
        keymap.bind(seq.clone(), "TestCommand", KeyContext::Global);

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("yunxi_test_keymap.json");

        keymap.save_to_file(&path).expect("保存失败");
        assert!(path.exists());

        let mut loaded = KeyMap::new();
        loaded.load_from_file(&path).expect("加载失败");

        let resolved = loaded.resolve(KeyContext::Global, &seq);
        assert_eq!(resolved, Some("TestCommand".to_string()));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_save_keymap_overrides_defaults() {
        let mut keymap = KeyMap::new();
        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('q')));

        keymap.bind(seq.clone(), "CustomQuit", KeyContext::Global);

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("yunxi_test_keymap_override.json");

        keymap.save_to_file(&path).expect("保存失败");

        let mut loaded = KeyMap::new();
        loaded.load_from_file(&path).expect("加载失败");

        let resolved = loaded.resolve(KeyContext::Global, &seq);
        assert_eq!(resolved, Some("CustomQuit".to_string()));

        let _ = std::fs::remove_file(&path);
    }
}
