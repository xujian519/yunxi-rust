use serde::{Deserialize, Serialize};
use serde_json::Value;

mod action_review;
mod llm_reflection;
pub mod reflection_llm_client;

/// 反思结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReflectionResult {
    pub action_type: String,
    pub success: bool,
    pub score: f64,
    pub issues: Vec<ActionIssue>,
    pub improvement_suggestions: Vec<String>,
    pub should_retry: bool,
    pub retry_strategy: Option<String>,
}

/// 动作问题。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionIssue {
    pub category: String,
    pub severity: String,
    pub description: String,
    pub location: Option<String>,
}

/// 反思配置。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReflectionConfig {
    pub max_retries: usize,
    pub score_threshold: f64,
    pub enable_llm_reflection: bool,
    pub enable_action_review: bool,
    pub reflection_timeout_ms: u64,
    /// 反思使用的 LLM 模型名称（可选）。
    #[serde(default)]
    pub reflection_model: Option<String>,
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            score_threshold: 75.0,
            enable_llm_reflection: true,
            enable_action_review: true,
            reflection_timeout_ms: 5000,
            reflection_model: None,
        }
    }
}

/// 统一反思接口。
pub trait Reflector {
    fn reflect(&self, action_result: &Value, action_type: &str)
        -> Result<ReflectionResult, String>;

    fn should_retry(&self, reflection: &ReflectionResult, config: &ReflectionConfig) -> bool {
        if !reflection.success {
            return true;
        }

        if reflection.score < config.score_threshold {
            return true;
        }

        false
    }
}
