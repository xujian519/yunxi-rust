use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

pub struct RenderCache<K: Hash + Eq + Clone, V: Clone> {
    cache: HashMap<K, CachedItem<V>>,
    max_size: usize,
    ttl: Duration,
}

struct CachedItem<V> {
    value: V,
    created_at: Instant,
}

impl<K: Hash + Eq + Clone, V: Clone> RenderCache<K, V> {
    pub fn new(max_size: usize, ttl_secs: u64) -> Self {
        Self { cache: HashMap::new(), max_size, ttl: Duration::from_secs(ttl_secs) }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.cache.get(key).and_then(|item| {
            if item.created_at.elapsed() < self.ttl { Some(item.value.clone()) } else { None }
        })
    }

    pub fn set(&mut self, key: K, value: V) {
        if self.cache.len() >= self.max_size {
            if let Some(oldest_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(key, CachedItem { value, created_at: Instant::now() });
    }

    pub fn invalidate(&mut self, key: &K) { self.cache.remove(key); }
    pub fn clear(&mut self) { self.cache.clear(); }
    pub fn len(&self) -> usize { self.cache.len() }
    pub fn is_empty(&self) -> bool { self.cache.is_empty() }
}

impl<K: Hash + Eq + Clone, V: Clone> Default for RenderCache<K, V> {
    fn default() -> Self { Self::new(128, 60) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_set_get() {
        let mut c: RenderCache<String, String> = RenderCache::new(10, 60);
        c.set("k".into(), "v".into());
        assert_eq!(c.get(&"k".into()), Some("v".into()));
    }

    #[test]
    fn test_cache_miss() {
        let c: RenderCache<String, String> = RenderCache::new(10, 60);
        assert_eq!(c.get(&"x".into()), None);
    }

    #[test]
    fn test_cache_invalidate() {
        let mut c: RenderCache<String, String> = RenderCache::new(10, 60);
        c.set("k".into(), "v".into());
        c.invalidate(&"k".into());
        assert_eq!(c.get(&"k".into()), None);
    }

    #[test]
    fn test_cache_clear() {
        let mut c: RenderCache<String, String> = RenderCache::new(10, 60);
        c.set("a".into(), "1".into());
        c.set("b".into(), "2".into());
        c.clear();
        assert!(c.is_empty());
    }

    #[test]
    fn test_cache_max_size() {
        let mut c: RenderCache<String, String> = RenderCache::new(2, 60);
        c.set("a".into(), "1".into());
        c.set("b".into(), "2".into());
        c.set("c".into(), "3".into());
        assert!(c.len() <= 2);
    }
}
