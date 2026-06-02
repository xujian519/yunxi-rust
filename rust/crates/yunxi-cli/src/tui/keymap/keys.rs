use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Char(char),
    F(u8),
    Null,
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    Esc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key: Key,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(key: Key, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn simple(key: Key) -> Self {
        Self::new(key, KeyModifiers::empty())
    }

    pub fn ctrl(key: Key) -> Self {
        Self::new(key, KeyModifiers::CONTROL)
    }

    pub fn alt(key: Key) -> Self {
        Self::new(key, KeyModifiers::ALT)
    }

    pub fn shift(key: Key) -> Self {
        Self::new(key, KeyModifiers::SHIFT)
    }

    pub fn ctrl_alt(key: Key) -> Self {
        Self::new(key, KeyModifiers::CONTROL | KeyModifiers::ALT)
    }

    pub fn ctrl_shift(key: Key) -> Self {
        Self::new(key, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    }

    pub fn alt_shift(key: Key) -> Self {
        Self::new(key, KeyModifiers::ALT | KeyModifiers::SHIFT)
    }

    pub fn ctrl_alt_shift(key: Key) -> Self {
        Self::new(
            key,
            KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT,
        )
    }

    pub fn from_crossterm(event: &KeyEvent) -> Self {
        Self {
            key: Key::from_code(event.code),
            modifiers: event.modifiers,
        }
    }
}

impl Key {
    pub fn from_code(code: KeyCode) -> Self {
        match code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::F(n) => Key::F(n),
            KeyCode::Null => Key::Null,
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Enter => Key::Enter,
            KeyCode::Left => Key::Left,
            KeyCode::Right => Key::Right,
            KeyCode::Up => Key::Up,
            KeyCode::Down => Key::Down,
            KeyCode::Home => Key::Home,
            KeyCode::End => Key::End,
            KeyCode::PageUp => Key::PageUp,
            KeyCode::PageDown => Key::PageDown,
            KeyCode::Tab => Key::Tab,
            KeyCode::BackTab => Key::BackTab,
            KeyCode::Delete => Key::Delete,
            KeyCode::Insert => Key::Insert,
            KeyCode::Esc => Key::Esc,
            _ => Key::Null,
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::Char(c) => write!(f, "{}", c),
            Key::F(n) => write!(f, "F{}", n),
            Key::Null => write!(f, "Null"),
            Key::Backspace => write!(f, "Backspace"),
            Key::Enter => write!(f, "Enter"),
            Key::Left => write!(f, "←"),
            Key::Right => write!(f, "→"),
            Key::Up => write!(f, "↑"),
            Key::Down => write!(f, "↓"),
            Key::Home => write!(f, "Home"),
            Key::End => write!(f, "End"),
            Key::PageUp => write!(f, "PageUp"),
            Key::PageDown => write!(f, "PageDown"),
            Key::Tab => write!(f, "Tab"),
            Key::BackTab => write!(f, "Shift+Tab"),
            Key::Delete => write!(f, "Delete"),
            Key::Insert => write!(f, "Insert"),
            Key::Esc => write!(f, "Esc"),
        }
    }
}

impl fmt::Display for KeyBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mods: Vec<&str> = vec![]
            .into_iter()
            .chain(if self.modifiers.contains(KeyModifiers::CONTROL) {
                Some("Ctrl+")
            } else {
                None
            })
            .chain(if self.modifiers.contains(KeyModifiers::ALT) {
                Some("Alt+")
            } else {
                None
            })
            .chain(if self.modifiers.contains(KeyModifiers::SHIFT) {
                Some("Shift+")
            } else {
                None
            })
            .collect();

        write!(f, "{}{}", mods.join(""), self.key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_from_char() {
        assert_eq!(Key::from_code(KeyCode::Char('a')), Key::Char('a'));
        assert_eq!(Key::from_code(KeyCode::Char('Z')), Key::Char('Z'));
    }

    #[test]
    fn test_key_from_special() {
        assert_eq!(Key::from_code(KeyCode::Enter), Key::Enter);
        assert_eq!(Key::from_code(KeyCode::Esc), Key::Esc);
        assert_eq!(Key::from_code(KeyCode::Backspace), Key::Backspace);
    }

    #[test]
    fn test_key_from_f_keys() {
        assert_eq!(Key::from_code(KeyCode::F(1)), Key::F(1));
        assert_eq!(Key::from_code(KeyCode::F(12)), Key::F(12));
    }

    #[test]
    fn test_key_binding_simple() {
        let binding = KeyBinding::simple(Key::Char('s'));
        assert_eq!(binding.key, Key::Char('s'));
        assert_eq!(binding.modifiers, KeyModifiers::empty());
    }

    #[test]
    fn test_key_binding_ctrl() {
        let binding = KeyBinding::ctrl(Key::Char('s'));
        assert_eq!(binding.key, Key::Char('s'));
        assert_eq!(binding.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_key_binding_ctrl_alt() {
        let binding = KeyBinding::ctrl_alt(Key::Char('s'));
        assert_eq!(binding.key, Key::Char('s'));
        assert_eq!(binding.modifiers, KeyModifiers::CONTROL | KeyModifiers::ALT);
    }

    #[test]
    fn test_key_binding_from_crossterm() {
        let event = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        let binding = KeyBinding::from_crossterm(&event);
        assert_eq!(binding.key, Key::Char('s'));
        assert_eq!(binding.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_key_display() {
        assert_eq!(format!("{}", Key::Char('a')), "a");
        assert_eq!(format!("{}", Key::Enter), "Enter");
        assert_eq!(format!("{}", Key::F(1)), "F1");
    }

    #[test]
    fn test_key_binding_display() {
        assert_eq!(format!("{}", KeyBinding::ctrl(Key::Char('s'))), "Ctrl+s");
        assert_eq!(
            format!("{}", KeyBinding::ctrl_alt(Key::Char('s'))),
            "Ctrl+Alt+s"
        );
        assert_eq!(format!("{}", KeyBinding::simple(Key::Enter)), "Enter");
    }
}
