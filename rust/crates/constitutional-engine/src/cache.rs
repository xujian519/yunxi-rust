//! LLM 分析结果缓存
//!
//! 为宪法引擎的 LLM 深度分析提供 TTL 缓存，
//! 避免对相同输入重复调用 LLM API。

use crate::llm_analyzer::LlmAnalysisResult;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 缓存的 LLM 分析结果
#[derive(Debug, Clone)]
pub struct CachedLlmResult {
    /// LLM 分析结果
    pub result: LlmAnalysisResult,
    /// 缓存写入时间戳（秒级 UNIX 时间戳）
    pub cached_at: u64,
    /// 缓存键
    pub cache_key: String,
}

impl CachedLlmResult {
    /// 创建新的缓存条目。
    pub fn new(cache_key: String, result: LlmAnalysisResult) -> Self {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            result,
            cached_at,
            cache_key,
        }
    }

    /// 检查缓存是否过期。
    ///
    /// # Arguments
    /// * `ttl_secs` - 缓存有效期（秒），默认 3600（1 小时）
    pub fn is_expired(&self, ttl_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        now.saturating_sub(self.cached_at) >= ttl_secs
    }
}

/// LLM 分析结果缓存
pub struct LlmAnalysisCache {
    /// 缓存存储
    store: HashMap<String, CachedLlmResult>,
    /// 默认 TTL（秒），默认 3600 = 1 小时
    default_ttl_secs: u64,
}

impl LlmAnalysisCache {
    /// 创建新的缓存实例，默认 TTL = 1 小时。
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            default_ttl_secs: 3600,
        }
    }

    /// 创建指定 TTL 的缓存实例。
    pub fn with_ttl(ttl_secs: u64) -> Self {
        Self {
            store: HashMap::new(),
            default_ttl_secs: ttl_secs,
        }
    }

    /// 生成缓存键。
    ///
    /// 格式：`{rule_id}:{input_hash}:{output_hash}`
    pub fn make_cache_key(rule_id: &str, input_text: &str, output_text: Option<&str>) -> String {
        let input_hash = simple_hash(input_text);
        let output_hash = output_text.map(simple_hash).unwrap_or(0u64);
        format!("{}:{:016x}:{:016x}", rule_id, input_hash, output_hash)
    }

    /// 获取缓存的分析结果。
    ///
    /// 返回 `Some` 仅当缓存命中且未过期。
    pub fn get(&self, key: &str) -> Option<&LlmAnalysisResult> {
        self.store.get(key).and_then(|cached| {
            if cached.is_expired(self.default_ttl_secs) {
                None
            } else {
                Some(&cached.result)
            }
        })
    }

    /// 写入缓存。
    pub fn put(&mut self, key: String, result: LlmAnalysisResult) {
        let cached = CachedLlmResult::new(key.clone(), result);
        self.store.insert(key, cached);
    }

    /// 清理所有过期缓存条目。
    pub fn evict_expired(&mut self) -> usize {
        let ttl = self.default_ttl_secs;
        let before = self.store.len();
        self.store.retain(|_, cached| !cached.is_expired(ttl));
        before - self.store.len()
    }

    /// 获取缓存条目数量（含可能过期的）。
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// 缓存是否为空。
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// 清空所有缓存。
    pub fn clear(&mut self) {
        self.store.clear();
    }
}

impl Default for LlmAnalysisCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 简单的 FNV-1a 哈希函数，用于生成缓存键。
fn simple_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = LlmAnalysisCache::new();
        let key = "rule-001:abc123:def456".to_string();
        let result = LlmAnalysisResult {
            passed: true,
            confidence: 0.9,
            details: vec!["通过".to_string()],
            reasoning: "测试推理".to_string(),
        };

        cache.put(key.clone(), result.clone());
        let cached = cache.get(&key);
        assert!(cached.is_some());
        assert!(cached.unwrap().passed);
    }

    #[test]
    fn test_cache_miss() {
        let cache = LlmAnalysisCache::new();
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_cache_key_deterministic() {
        let key1 = LlmAnalysisCache::make_cache_key("rule-001", "input", Some("output"));
        let key2 = LlmAnalysisCache::make_cache_key("rule-001", "input", Some("output"));
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_key_differs_for_different_input() {
        let key1 = LlmAnalysisCache::make_cache_key("rule-001", "input1", None);
        let key2 = LlmAnalysisCache::make_cache_key("rule-001", "input2", None);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_cached_result_expiry() {
        let mut cached = CachedLlmResult::new(
            "key".to_string(),
            LlmAnalysisResult {
                passed: true,
                confidence: 0.9,
                details: vec![],
                reasoning: String::new(),
            },
        );
        // 刚创建不应过期
        assert!(!cached.is_expired(3600));
        // TTL=0 应立即过期
        assert!(cached.is_expired(0));
        // 手动设置旧时间戳模拟过期
        cached.cached_at = 0;
        assert!(cached.is_expired(3600));
    }

    #[test]
    fn test_evict_expired() {
        let mut cache = LlmAnalysisCache::with_ttl(0); // TTL=0，立即过期
        let result = LlmAnalysisResult {
            passed: true,
            confidence: 0.5,
            details: vec![],
            reasoning: String::new(),
        };
        cache.put("key1".to_string(), result.clone());
        cache.put("key2".to_string(), result.clone());
        assert_eq!(cache.len(), 2);

        let evicted = cache.evict_expired();
        assert_eq!(evicted, 2);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_simple_hash() {
        let h1 = simple_hash("hello");
        let h2 = simple_hash("hello");
        let h3 = simple_hash("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_default_cache() {
        let cache = LlmAnalysisCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }
}
