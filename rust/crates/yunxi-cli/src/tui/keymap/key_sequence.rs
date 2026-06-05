use super::keys::{Key, KeyBinding};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeySequence {
    keys: Vec<KeyBinding>,
}

impl Serialize for KeySequence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for KeySequence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        KeySequence::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl KeySequence {
    pub fn new(keys: Vec<KeyBinding>) -> Self {
        Self { keys }
    }

    pub fn single(key: KeyBinding) -> Self {
        Self::new(vec![key])
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn push(&mut self, key: KeyBinding) {
        self.keys.push(key);
    }

    pub fn pop(&mut self) -> Option<KeyBinding> {
        self.keys.pop()
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }

    pub fn matches_prefix(&self, sequence: &KeySequence) -> bool {
        if self.keys.len() < sequence.keys.len() {
            return false;
        }
        self.keys[..sequence.keys.len()] == sequence.keys[..]
    }

    pub fn matches(&self, sequence: &KeySequence) -> bool {
        self.keys == sequence.keys
    }

    pub fn iter(&self) -> impl Iterator<Item = &KeyBinding> {
        self.keys.iter()
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let mut keys = Vec::new();
        let parts: Vec<&str> = s.split(' ').collect();

        for part in parts {
            if part.is_empty() {
                continue;
            }
            keys.push(Self::parse_binding(part)?);
        }

        Ok(Self::new(keys))
    }

    fn parse_binding(s: &str) -> Result<KeyBinding, String> {
        use crossterm::event::KeyModifiers;

        let mut modifiers = KeyModifiers::empty();
        let mut key_str = s;

        while key_str.contains('+') {
            if key_str.starts_with("Ctrl+") {
                modifiers |= KeyModifiers::CONTROL;
                key_str = &key_str[5..];
            } else if key_str.starts_with("Alt+") {
                modifiers |= KeyModifiers::ALT;
                key_str = &key_str[4..];
            } else if key_str.starts_with("Shift+") {
                modifiers |= KeyModifiers::SHIFT;
                key_str = &key_str[6..];
            } else {
                break;
            }
        }

        Ok(KeyBinding::new(Self::parse_key(key_str)?, modifiers))
    }

    fn parse_key(s: &str) -> Result<Key, String> {
        match s {
            "Enter" => Ok(Key::Enter),
            "Esc" => Ok(Key::Esc),
            "Backspace" => Ok(Key::Backspace),
            "Tab" => Ok(Key::Tab),
            "Delete" => Ok(Key::Delete),
            "Insert" => Ok(Key::Insert),
            "Home" => Ok(Key::Home),
            "End" => Ok(Key::End),
            "PageUp" => Ok(Key::PageUp),
            "PageDown" => Ok(Key::PageDown),
            "Up" => Ok(Key::Up),
            "Down" => Ok(Key::Down),
            "Left" => Ok(Key::Left),
            "Right" => Ok(Key::Right),
            s if s.starts_with("F") && s.len() > 1 => {
                let num: u8 = s[1..]
                    .parse()
                    .map_err(|_| format!("Invalid F-key: {}", s))?;
                if num >= 1 && num <= 12 {
                    Ok(Key::F(num))
                } else {
                    Err(format!("Invalid F-key: {}", s))
                }
            }
            s if s.len() == 1 => Ok(Key::Char(s.chars().next().unwrap())),
            _ => Err(format!("Unknown key: {}", s)),
        }
    }

    pub fn to_string(&self) -> String {
        self.keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl std::fmt::Display for KeySequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::iter::FromIterator<KeyBinding> for KeySequence {
    fn from_iter<I: IntoIterator<Item = KeyBinding>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoIterator for KeySequence {
    type Item = KeyBinding;
    type IntoIter = std::vec::IntoIter<KeyBinding>;

    fn into_iter(self) -> Self::IntoIter {
        self.keys.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct SequenceTracker {
    current: KeySequence,
    last_input: Option<Instant>,
    timeout: Duration,
}

impl SequenceTracker {
    pub fn new(timeout: Duration) -> Self {
        Self {
            current: KeySequence::empty(),
            last_input: None,
            timeout,
        }
    }

    pub fn with_default_timeout() -> Self {
        Self::new(Duration::from_millis(1000))
    }

    pub fn push_key(&mut self, key: KeyBinding) -> bool {
        self.current.push(key);
        self.last_input = Some(Instant::now());
        true
    }

    pub fn is_timeout(&self) -> bool {
        if let Some(last) = self.last_input {
            last.elapsed() > self.timeout
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.current.clear();
        self.last_input = None;
    }

    pub fn current(&self) -> &KeySequence {
        &self.current
    }

    pub fn timeout(&self) -> &Duration {
        &self.timeout
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn tick(&mut self) {
        if self.is_timeout() {
            self.clear();
        }
    }

    pub fn get_hint(&self) -> Option<String> {
        if self.current.is_empty() {
            None
        } else {
            Some(format!("{} - 等待下一键...", self.current))
        }
    }
}

impl Default for SequenceTracker {
    fn default() -> Self {
        Self::with_default_timeout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    #[test]
    fn test_key_sequence_single() {
        let seq = KeySequence::single(KeyBinding::ctrl(Key::Char('s')));
        assert_eq!(seq.len(), 1);
        assert!(!seq.is_empty());
    }

    #[test]
    fn test_key_sequence_empty() {
        let seq = KeySequence::empty();
        assert_eq!(seq.len(), 0);
        assert!(seq.is_empty());
    }

    #[test]
    fn test_key_sequence_push() {
        let mut seq = KeySequence::empty();
        seq.push(KeyBinding::simple(Key::Char('g')));
        seq.push(KeyBinding::simple(Key::Char('g')));
        assert_eq!(seq.len(), 2);
    }

    #[test]
    fn test_key_sequence_matches() {
        let seq1 = KeySequence::new(vec![
            KeyBinding::simple(Key::Char('g')),
            KeyBinding::simple(Key::Char('g')),
        ]);
        let seq2 = KeySequence::new(vec![
            KeyBinding::simple(Key::Char('g')),
            KeyBinding::simple(Key::Char('g')),
        ]);
        assert!(seq1.matches(&seq2));
    }

    #[test]
    fn test_key_sequence_matches_prefix() {
        let long = KeySequence::new(vec![
            KeyBinding::simple(Key::Char('g')),
            KeyBinding::simple(Key::Char('g')),
            KeyBinding::simple(Key::Char('a')),
        ]);
        let prefix = KeySequence::new(vec![
            KeyBinding::simple(Key::Char('g')),
            KeyBinding::simple(Key::Char('g')),
        ]);
        assert!(long.matches_prefix(&prefix));
    }

    #[test]
    fn test_key_sequence_from_str() {
        let seq = KeySequence::from_str("Ctrl+s").unwrap();
        assert_eq!(seq.len(), 1);
        assert_eq!(seq.iter().next().unwrap().key, Key::Char('s'));

        let seq = KeySequence::from_str("g g").unwrap();
        assert_eq!(seq.len(), 2);
    }

    #[test]
    fn test_key_sequence_combined_modifiers() {
        use crossterm::event::KeyModifiers;

        let seq = KeySequence::from_str("Ctrl+Alt+s").unwrap();
        assert_eq!(seq.len(), 1);
        let binding = seq.iter().next().unwrap();
        assert_eq!(binding.key, Key::Char('s'));
        assert!(binding.modifiers.contains(KeyModifiers::CONTROL));
        assert!(binding.modifiers.contains(KeyModifiers::ALT));

        let seq = KeySequence::from_str("Alt+Ctrl+s").unwrap();
        assert_eq!(seq.len(), 1);
        let binding = seq.iter().next().unwrap();
        assert_eq!(binding.key, Key::Char('s'));
        assert!(binding.modifiers.contains(KeyModifiers::CONTROL));
        assert!(binding.modifiers.contains(KeyModifiers::ALT));

        let seq = KeySequence::from_str("Ctrl+Shift+A").unwrap();
        assert_eq!(seq.len(), 1);
        let binding = seq.iter().next().unwrap();
        assert_eq!(binding.key, Key::Char('A'));
        assert!(binding.modifiers.contains(KeyModifiers::CONTROL));
        assert!(binding.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_key_sequence_to_string() {
        let seq = KeySequence::new(vec![
            KeyBinding::simple(Key::Char('g')),
            KeyBinding::simple(Key::Char('g')),
        ]);
        assert_eq!(seq.to_string(), "g g");
    }

    #[test]
    fn test_sequence_tracker() {
        let mut tracker = SequenceTracker::with_default_timeout();
        assert!(tracker.current().is_empty());

        tracker.push_key(KeyBinding::simple(Key::Char('g')));
        assert_eq!(tracker.current().len(), 1);

        tracker.clear();
        assert!(tracker.current().is_empty());
    }

    #[test]
    fn test_sequence_tracker_timeout() {
        let mut tracker = SequenceTracker::new(Duration::from_millis(10));
        tracker.push_key(KeyBinding::simple(Key::Char('g')));
        assert!(!tracker.is_timeout());

        std::thread::sleep(Duration::from_millis(20));
        assert!(tracker.is_timeout());
    }

    #[test]
    fn test_sequence_tracker_hint() {
        let mut tracker = SequenceTracker::with_default_timeout();
        assert!(tracker.get_hint().is_none());

        tracker.push_key(KeyBinding::simple(Key::Char('g')));
        let hint = tracker.get_hint();
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("g - 等待下一键"));
    }
}
