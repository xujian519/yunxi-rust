use super::*;
use crate::tui::core::action::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn test_keymap_integration() {
    let mut keymap = KeyMap::new();
    
    let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let key = KeyBinding::from_crossterm(&event);
    
    let actions = keymap.handle_key(key, KeyContext::Global);
    assert!(actions.is_some());
    assert_eq!(actions.unwrap()[0], Action::Quit);
}

#[test]
fn test_keymap_sequence_integration() {
    let mut keymap = KeyMap::new();
    keymap.bind(
        KeySequence::new(vec![
            KeyBinding::simple(Key::Char('g')),
            KeyBinding::simple(Key::Char('d')),
        ]),
        "GoToDefinition",
        KeyContext::Editor,
    );

    let result1 = keymap.handle_key(KeyBinding::simple(Key::Char('g')), KeyContext::Editor);
    assert!(result1.is_none());
    
    let result2 = keymap.handle_key(KeyBinding::simple(Key::Char('d')), KeyContext::Editor);
    assert!(result2.is_some());
}

#[test]
fn test_keymap_context_fallback() {
    let mut keymap = KeyMap::new();
    keymap.set_context_priority(ContextPriority::with_current(KeyContext::Editor));
    
    let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL);
    let key = KeyBinding::from_crossterm(&event);
    
    let actions = keymap.handle_key(key, KeyContext::Editor);
    assert!(actions.is_some());
    assert_eq!(actions.unwrap()[0], Action::Quit);
}

#[test]
fn test_keymap_command_registry() {
    let keymap = KeyMap::new();
    
    let commands = keymap.list_commands();
    assert!(!commands.is_empty());
    
    let quit_cmd = keymap.get_command("Quit");
    assert!(quit_cmd.is_some());
    
    let actions = keymap.execute_command("Quit");
    assert!(actions.is_some());
    assert_eq!(actions.unwrap()[0], Action::Quit);
}

#[test]
fn test_keymap_conflict_detection() {
    let mut keymap = KeyMap::new();
    let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('q')));
    
    let conflicts = keymap.check_conflicts(&seq, KeyContext::Global);
    assert!(!conflicts.is_empty());
    assert_eq!(conflicts[0].command, "Quit");
}

#[test]
fn test_keymap_timeout_reset() {
    let mut keymap = KeyMap::new();
    keymap.set_timeout(Duration::from_millis(10));
    
    keymap.handle_key(KeyBinding::simple(Key::Char('g')), KeyContext::List);
    std::thread::sleep(Duration::from_millis(20));
    
    keymap.get_tracker_mut().tick();
    assert!(keymap.get_tracker().current().is_empty());
}

#[test]
fn test_keymap_partial_match() {
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
    
    assert!(!keymap.get_tracker().current().is_empty());
    
    keymap.get_tracker_mut().tick();
    assert!(!keymap.get_tracker().current().is_empty());
}

#[test]
fn test_keymap_invalid_sequence_reset() {
    let mut keymap = KeyMap::new();
    
    let result1 = keymap.handle_key(KeyBinding::simple(Key::Char('x')), KeyContext::List);
    assert!(result1.is_none());
    assert!(keymap.get_tracker().current().is_empty());
}

#[test]
fn test_keymap_list_bindings() {
    let keymap = KeyMap::new();
    
    let global_bindings = keymap.list_bindings(KeyContext::Global);
    assert!(!global_bindings.is_empty());
    
    let all_bindings = keymap.list_all_bindings();
    assert!(!all_bindings.is_empty());
}

#[test]
fn test_keymap_rebind() {
    let mut keymap = KeyMap::new();
    let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('t')));
    
    keymap.bind(seq.clone(), "CustomCommand", KeyContext::Global);
    let command = keymap.resolve(KeyContext::Global, &seq);
    assert_eq!(command, Some("CustomCommand".to_string()));
    
    keymap.unbind(&seq, KeyContext::Global);
    let command = keymap.resolve(KeyContext::Global, &seq);
    assert_eq!(command, None);
}

#[test]
fn test_keymap_multiple_contexts() {
    let keymap = KeyMap::new();
    let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('z')));
    
    let editor_cmd = keymap.resolve(KeyContext::Editor, &seq);
    assert_eq!(editor_cmd, Some("EditorUndo".to_string()));
    
    let global_cmd = keymap.resolve(KeyContext::Global, &seq);
    assert_eq!(global_cmd, None);
}

#[test]
fn test_keymap_priority_resolution() {
    let mut keymap = KeyMap::new();
    let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('x')));
    
    keymap.bind(seq.clone(), "EditorCommand", KeyContext::Editor);
    keymap.bind(seq.clone(), "GlobalCommand", KeyContext::Global);
    
    keymap.set_context_priority(ContextPriority::with_current(KeyContext::Editor));
    let command = keymap.resolve(KeyContext::Editor, &seq);
    assert_eq!(command, Some("EditorCommand".to_string()));
}

#[test]
fn test_keymap_tracker_hint() {
    let mut keymap = KeyMap::new();
    
    keymap.handle_key(KeyBinding::simple(Key::Char('g')), KeyContext::List);
    let hint = keymap.get_tracker().get_hint();
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("g - 等待下一键"));
}

#[test]
fn test_keymap_command_search() {
    let keymap = KeyMap::new();
    
    let results = keymap.list_commands();
    let save_results: Vec<_> = results.iter().filter(|c| c.name.to_lowercase().contains("save")).collect();
    assert!(!save_results.is_empty());
}

#[test]
fn test_keymap_display() {
    let binding = Binding::new(
        KeySequence::single(KeyBinding::ctrl(Key::Char('s'))),
        "SaveSession",
        KeyContext::Global,
    ).with_description("保存会话");
    
    let display = binding.to_string();
    assert!(display.contains("Ctrl+s"));
    assert!(display.contains("SaveSession"));
    assert!(display.contains("保存会话"));
}

#[test]
fn test_keymap_empty_sequence() {
    let keymap = KeyMap::new();
    let seq = KeySequence::empty();
    
    let command = keymap.resolve(KeyContext::Global, &seq);
    assert_eq!(command, None);
}

#[test]
fn test_keymap_long_sequence() {
    let mut keymap = KeyMap::new();
    let seq = KeySequence::new(vec![
        KeyBinding::ctrl(Key::Char('g')),
        KeyBinding::ctrl(Key::Char('t')),
        KeyBinding::ctrl(Key::Char('p')),
    ]);
    
    keymap.bind(seq.clone(), "ComplexCommand", KeyContext::Global);
    
    let mut results = Vec::new();
    for binding in seq.iter() {
        results.push(keymap.handle_key(*binding, KeyContext::Global));
    }
    
    assert!(results[0].is_none());
    assert!(results[1].is_none());
    assert!(results[2].is_some());
}