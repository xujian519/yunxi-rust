use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

mod pipeline;
mod registry;
mod tracing;

pub use registry::*;
pub use tracing::*;

/// 统一评估器。
///
/// 集成所有评估器类型（规则型、LLM型、混合型）。
pub struct UnifiedEvaluator {
    registry: EvalRegistry,
    cache: Arc<Mutex<EvalCache>>,
    config: EvalConfig,
}

impl UnifiedEvaluator {
    pub fn new(config: EvalConfig) -> Self {
        Self {
            registry: EvalRegistry::new(),
            cache: Arc::new(Mutex::new(EvalCache::new(config.cache_size))),
            config,
        }
    }

    pub fn with_default() -> Self {
        Self::new(EvalConfig::default())
    }

    /// 注册评估器。
    pub fn register(&mut self, name: String, evaluator: Box<dyn Evaluator>) -> Result<(), String> {
        self.registry.register(name, evaluator)
    }

    /// 执行评估（带缓存）。
    pub fn evaluate(&self, eval_request: &EvalRequest) -> Result<EvalResponse, String> {
        // 检查缓存
        let cache_key = self.generate_cache_key(eval_request);
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return Ok(cached.clone());
        }

        // 执行评估
        let response = self.evaluate_uncached(eval_request)?;

        // 缓存结果
        self.cache.lock().unwrap().put(cache_key, response.clone());

        Ok(response)
    }

    fn evaluate_uncached(&self, request: &EvalRequest) -> Result<EvalResponse, String> {
        let evaluator = self
            .registry
            .get(&request.evaluator_type)
            .ok_or_else(|| format!("Evaluator {} not found", request.evaluator_type))?;

        // 添加追踪
        let trace = EvalTrace::start(&request.evaluator_type, &request.input_data);

        let result = evaluator.evaluate(&request.input_data, &request.context);

        let response = match result {
            Ok(data) => {
                let trace = trace.complete(&data);
                EvalResponse {
                    success: true,
                    data,
                    trace: Some(trace),
                    error: None,
                }
            }
            Err(error) => {
                let trace = trace.fail(&error);
                EvalResponse {
                    success: false,
                    data: Value::Null,
                    trace: Some(trace),
                    error: Some(error),
                }
            }
        };

        Ok(response)
    }

    fn generate_cache_key(&self, request: &EvalRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        request.evaluator_type.hash(&mut hasher);
        serde_json::to_string(&request.input_data)
            .unwrap()
            .hash(&mut hasher);
        format!("{}:{:x}", request.evaluator_type, hasher.finish())
    }
}

/// 评估器类型。
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EvaluatorType {
    /// 规则型评估器
    RuleBased(String),
    /// LLM 型评估器
    LLMBased(String),
    /// 混合型评估器
    Hybrid(String),
}

/// 评估请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalRequest {
    pub evaluator_type: String,
    pub input_data: Value,
    pub context: Value,
}

/// 评估响应。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalResponse {
    pub success: bool,
    pub data: Value,
    pub trace: Option<EvalTrace>,
    pub error: Option<String>,
}

/// 评估配置。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalConfig {
    pub cache_size: usize,
    pub enable_tracing: bool,
    pub timeout_ms: u64,
    pub max_retries: usize,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            cache_size: 1000,
            enable_tracing: true,
            timeout_ms: 30000,
            max_retries: 3,
        }
    }
}

/// 评估器 trait。
pub trait Evaluator: Send + Sync {
    fn evaluate(&self, input: &Value, context: &Value) -> Result<Value, String>;
    fn evaluator_name(&self) -> &str;
}

/// 评估缓存。
struct EvalCache {
    size: usize,
    entries: HashMap<String, EvalResponse>,
    access_order: Vec<String>,
}

impl EvalCache {
    fn new(size: usize) -> Self {
        Self {
            size,
            entries: HashMap::new(),
            access_order: Vec::new(),
        }
    }

    fn get(&mut self, key: &str) -> Option<EvalResponse> {
        if let Some(response) = self.entries.get(key) {
            // 更新访问顺序
            self.access_order.retain(|k| k != key);
            self.access_order.push(key.to_string());
            Some(response.clone())
        } else {
            None
        }
    }

    fn put(&mut self, key: String, response: EvalResponse) {
        if self.entries.len() >= self.size {
            if let Some(oldest) = self.access_order.first() {
                self.entries.remove(oldest);
                self.access_order.remove(0);
            }
        }
        self.entries.insert(key.clone(), response);
        self.access_order.push(key);
    }
}
