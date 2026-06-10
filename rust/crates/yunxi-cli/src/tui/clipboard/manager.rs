use super::{copy_text_to_clipboard, strip_ansi};
use std::sync::{Arc, Mutex};

/// 剪贴板历史记录条目
#[derive(Debug, Clone)]
pub struct ClipboardHistory {
    entries: Vec<String>,
    max_size: usize,
}

impl ClipboardHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, content: String) {
        if self.entries.contains(&content) {
            return;
        }
        self.entries.insert(0, content);
        if self.entries.len() > self.max_size {
            self.entries.truncate(self.max_size);
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        self.entries.get(index).map(|s| s.as_str())
    }

    pub fn get_latest(&self) -> Option<&str> {
        self.entries.first().map(|s| s.as_str())
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.entries.iter()
    }
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self::new(10)
    }
}

/// 剪贴板管理器
#[derive(Clone)]
pub struct ClipboardManager {
    history: Arc<Mutex<ClipboardHistory>>,
}

impl ClipboardManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            history: Arc::new(Mutex::new(ClipboardHistory::new(max_history))),
        }
    }

    pub fn new_default() -> Self {
        Self::new(10)
    }

    /// 辅助方法：获取锁，lock poisoning 时恢复内部数据继续运行
    fn locked(&self) -> std::sync::MutexGuard<'_, ClipboardHistory> {
        self.history.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// 复制文本到系统剪贴板并记录历史
    pub fn copy(&self, text: &str) -> Result<(), String> {
        let stripped = strip_ansi(text);
        copy_text_to_clipboard(&stripped)?;
        self.locked().push(stripped);
        Ok(())
    }

    /// 从系统剪贴板粘贴
    pub fn paste(&self) -> Result<String, String> {
        arboard::Clipboard::new()
            .map_err(|e| format!("无法打开剪贴板: {e}"))?
            .get_text()
            .map_err(|e| format!("读取剪贴板失败: {e}"))
    }

    /// 获取历史记录中的条目
    pub fn get_history(&self, index: usize) -> Option<String> {
        self.locked().get(index).map(|s| s.to_string())
    }

    /// 获取最新历史记录
    pub fn get_latest_history(&self) -> Option<String> {
        self.locked().get_latest().map(|s| s.to_string())
    }

    /// 获取历史记录长度
    pub fn history_len(&self) -> usize {
        self.locked().len()
    }

    /// 清空历史记录
    pub fn clear_history(&self) {
        self.locked().clear();
    }

    /// 检查历史记录是否为空
    pub fn is_history_empty(&self) -> bool {
        self.locked().is_empty()
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_history_push() {
        let mut history = ClipboardHistory::new(3);
        history.push("item1".to_string());
        history.push("item2".to_string());
        history.push("item3".to_string());

        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0), Some("item3"));
        assert_eq!(history.get(1), Some("item2"));
        assert_eq!(history.get(2), Some("item1"));
    }

    #[test]
    fn test_clipboard_history_max_size() {
        let mut history = ClipboardHistory::new(2);
        history.push("item1".to_string());
        history.push("item2".to_string());
        history.push("item3".to_string());

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0), Some("item3"));
        assert_eq!(history.get(1), Some("item2"));
    }

    #[test]
    fn test_clipboard_history_no_duplicates() {
        let mut history = ClipboardHistory::new(5);
        history.push("item1".to_string());
        history.push("item2".to_string());
        history.push("item1".to_string());

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0), Some("item2"));
        assert_eq!(history.get(1), Some("item1"));
    }

    #[test]
    fn test_clipboard_history_clear() {
        let mut history = ClipboardHistory::new(3);
        history.push("item1".to_string());
        history.push("item2".to_string());
        history.clear();

        assert_eq!(history.len(), 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_clipboard_manager_create() {
        let manager = ClipboardManager::new_default();
        assert!(manager.is_history_empty());
    }

    #[test]
    fn test_clipboard_manager_history_len() {
        let manager = ClipboardManager::new(5);
        assert_eq!(manager.history_len(), 0);
    }
}
