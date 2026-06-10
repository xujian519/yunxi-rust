use crate::tui::components::progress_indicator::{ProgressStyle, ProgressType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProgressId {
    Named(String),
    Id(usize),
}

impl ProgressId {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    pub fn id(id: usize) -> Self {
        Self::Id(id)
    }
}

#[derive(Debug, Clone)]
pub struct ProgressIndicatorData {
    pub id: ProgressId,
    pub title: String,
    pub progress_type: ProgressType,
    pub current: f32,
    pub total: f32,
    pub message: Option<String>,
    pub style: ProgressStyle,
}

impl ProgressIndicatorData {
    pub fn new(id: ProgressId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            progress_type: ProgressType::Spinner,
            current: 0.0,
            total: 100.0,
            message: None,
            style: ProgressStyle::default(),
        }
    }

    pub fn with_type(mut self, progress_type: ProgressType) -> Self {
        self.progress_type = progress_type;
        self
    }

    pub fn with_current(mut self, current: f32) -> Self {
        self.current = current;
        self
    }

    pub fn with_total(mut self, total: f32) -> Self {
        self.total = total;
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    pub fn get_percentage(&self) -> f32 {
        if self.total <= 0.0 {
            return 0.0;
        }
        (self.current / self.total) * 100.0
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.total
    }
}

pub struct ProgressManager {
    indicators: Arc<Mutex<HashMap<ProgressId, ProgressIndicatorData>>>,
    next_id: Arc<Mutex<usize>>,
}

impl ProgressManager {
    pub fn new() -> Self {
        Self {
            indicators: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    /// 辅助方法：获取锁，lock poisoning 时恢复内部数据继续运行
    fn locked<'a, T>(&self, lock: &'a Mutex<T>) -> std::sync::MutexGuard<'a, T> {
        lock.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn create(&self, title: impl Into<String>) -> ProgressId {
        let mut next_id = self.locked(&self.next_id);
        let id = ProgressId::Id(*next_id);
        *next_id += 1;

        let data = ProgressIndicatorData::new(id.clone(), title);
        self.locked(&self.indicators).insert(id.clone(), data);

        id
    }

    pub fn create_named(&self, name: impl Into<String>, title: impl Into<String>) -> ProgressId {
        let id = ProgressId::Named(name.into());
        let data = ProgressIndicatorData::new(id.clone(), title);
        self.locked(&self.indicators).insert(id.clone(), data);

        id
    }

    pub fn update(&self, id: &ProgressId, current: f32, total: Option<f32>) {
        if let Some(indicator) = self.locked(&self.indicators).get_mut(id) {
            indicator.current = current;
            if let Some(total) = total {
                indicator.total = total;
            }
        }
    }

    pub fn set_message(&self, id: &ProgressId, message: impl Into<String>) {
        if let Some(indicator) = self.locked(&self.indicators).get_mut(id) {
            indicator.message = Some(message.into());
        }
    }

    pub fn set_type(&self, id: &ProgressId, progress_type: ProgressType) {
        if let Some(indicator) = self.locked(&self.indicators).get_mut(id) {
            indicator.progress_type = progress_type;
        }
    }

    pub fn set_style(&self, id: &ProgressId, style: ProgressStyle) {
        if let Some(indicator) = self.locked(&self.indicators).get_mut(id) {
            indicator.style = style;
        }
    }

    pub fn complete(&self, id: &ProgressId) {
        if let Some(indicator) = self.locked(&self.indicators).get_mut(id) {
            indicator.current = indicator.total;
        }
    }

    pub fn reset(&self, id: &ProgressId) {
        if let Some(indicator) = self.locked(&self.indicators).get_mut(id) {
            indicator.current = 0.0;
        }
    }

    pub fn remove(&self, id: &ProgressId) {
        self.locked(&self.indicators).remove(id);
    }

    pub fn get(&self, id: &ProgressId) -> Option<ProgressIndicatorData> {
        self.locked(&self.indicators).get(id).cloned()
    }

    pub fn get_all(&self) -> Vec<ProgressIndicatorData> {
        self.locked(&self.indicators).values().cloned().collect()
    }

    pub fn count(&self) -> usize {
        self.locked(&self.indicators).len()
    }

    pub fn is_empty(&self) -> bool {
        self.locked(&self.indicators).is_empty()
    }

    pub fn clear(&self) {
        self.locked(&self.indicators).clear();
    }

    pub fn get_active_count(&self) -> usize {
        self.locked(&self.indicators)
            .values()
            .filter(|i| !i.is_complete())
            .count()
    }

    pub fn get_completed_count(&self) -> usize {
        self.locked(&self.indicators)
            .values()
            .filter(|i| i.is_complete())
            .count()
    }
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ProgressManager {
    fn clone(&self) -> Self {
        Self {
            indicators: Arc::clone(&self.indicators),
            next_id: Arc::clone(&self.next_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_id_creation() {
        let id1 = ProgressId::named("test");
        let id2 = ProgressId::id(42);

        matches!(id1, ProgressId::Named(_));
        matches!(id2, ProgressId::Id(_));
    }

    #[test]
    fn test_progress_indicator_data_creation() {
        let id = ProgressId::named("test");
        let data = ProgressIndicatorData::new(id.clone(), "Test Progress");

        assert_eq!(data.id, id);
        assert_eq!(data.title, "Test Progress");
        assert_eq!(data.current, 0.0);
        assert_eq!(data.total, 100.0);
    }

    #[test]
    fn test_progress_indicator_data_with_type() {
        let id = ProgressId::named("test");
        let data = ProgressIndicatorData::new(id, "Test").with_type(ProgressType::Bar);

        assert_eq!(data.progress_type, ProgressType::Bar);
    }

    #[test]
    fn test_get_percentage() {
        let id = ProgressId::named("test");
        let data = ProgressIndicatorData::new(id, "Test")
            .with_current(50.0)
            .with_total(100.0);

        assert_eq!(data.get_percentage(), 50.0);
    }

    #[test]
    fn test_get_percentage_zero_total() {
        let id = ProgressId::named("test");
        let data = ProgressIndicatorData::new(id, "Test")
            .with_current(50.0)
            .with_total(0.0);

        assert_eq!(data.get_percentage(), 0.0);
    }

    #[test]
    fn test_is_complete() {
        let id = ProgressId::named("test");
        let data = ProgressIndicatorData::new(id, "Test")
            .with_current(100.0)
            .with_total(100.0);

        assert!(data.is_complete());
    }

    #[test]
    fn test_progress_manager_creation() {
        let manager = ProgressManager::new();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_create_progress() {
        let manager = ProgressManager::new();
        let _id = manager.create("Test Progress");

        assert_eq!(manager.count(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_create_named_progress() {
        let manager = ProgressManager::new();
        let id = manager.create_named("test", "Test Progress");

        assert_eq!(manager.count(), 1);
        matches!(id, ProgressId::Named(n) if n == "test");
    }

    #[test]
    fn test_update_progress() {
        let manager = ProgressManager::new();
        let id = manager.create("Test Progress");

        manager.update(&id, 50.0, None);
        let data = manager.get(&id).unwrap();

        assert_eq!(data.current, 50.0);
    }

    #[test]
    fn test_set_message() {
        let manager = ProgressManager::new();
        let id = manager.create("Test Progress");

        manager.set_message(&id, "Processing...");
        let data = manager.get(&id).unwrap();

        assert_eq!(data.message.as_ref().unwrap(), "Processing...");
    }

    #[test]
    fn test_complete_progress() {
        let manager = ProgressManager::new();
        let id = manager.create("Test Progress");

        manager.update(&id, 100.0, Some(100.0));
        let data = manager.get(&id).unwrap();

        assert!(data.is_complete());
    }

    #[test]
    fn test_reset_progress() {
        let manager = ProgressManager::new();
        let id = manager.create("Test Progress");

        manager.update(&id, 50.0, None);
        manager.reset(&id);

        let data = manager.get(&id).unwrap();
        assert_eq!(data.current, 0.0);
    }

    #[test]
    fn test_remove_progress() {
        let manager = ProgressManager::new();
        let id = manager.create("Test Progress");

        assert_eq!(manager.count(), 1);
        manager.remove(&id);
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_get_all() {
        let manager = ProgressManager::new();
        manager.create("Progress 1");
        manager.create("Progress 2");
        manager.create("Progress 3");

        let all = manager.get_all();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_clear_all() {
        let manager = ProgressManager::new();
        manager.create("Progress 1");
        manager.create("Progress 2");

        assert_eq!(manager.count(), 2);
        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_active_count() {
        let manager = ProgressManager::new();
        let _id2 = manager.create("Progress 2");
        let id1 = manager.create("Progress 1");

        manager.update(&id1, 100.0, Some(100.0));

        assert_eq!(manager.get_active_count(), 1);
        assert_eq!(manager.get_completed_count(), 1);
    }

    #[test]
    fn test_progress_manager_clone() {
        let manager1 = ProgressManager::new();
        let id = manager1.create("Test Progress");

        let manager2 = manager1.clone();
        manager2.update(&id, 25.0, None);

        let data = manager1.get(&id).unwrap();
        assert_eq!(data.current, 25.0);
    }
}
