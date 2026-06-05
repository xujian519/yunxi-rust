//! LLM 反思器 LLM 调用适配器
//!
//! 封装 LlmClient 的同步调用，为反思器提供简洁的文本生成接口。
//! 使用 `reqwest::blocking` 直接调用 OpenAI 兼容 API，
//! 避免对 llm crate 的编译期依赖（防止循环依赖）。

use serde_json::Value;

/// LLM 调用适配器，用于反思器内部同步调用 LLM。
///
/// 不依赖 llm crate，通过 reqwest::blocking 直接调用 API。
pub struct ReflectionLlmClient {
    base_url: String,
    api_key: String,
    model: String,
}

impl ReflectionLlmClient {
    /// 创建新的 LLM 调用适配器。
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
        }
    }

    /// 从环境变量自动构建适配器。
    ///
    /// 按以下优先级读取配置：
    /// 1. `YUNXI_REFLECTION_LLM_BASE_URL` / `YUNXI_REFLECTION_LLM_API_KEY` / `YUNXI_REFLECTION_LLM_MODEL`
    /// 2. `OPENAI_BASE_URL` / `OPENAI_API_KEY`，默认模型 `gpt-4o-mini`
    pub fn from_env(model_override: Option<&str>) -> Option<Self> {
        let base_url = std::env::var("YUNXI_REFLECTION_LLM_BASE_URL")
            .ok()
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .or_else(|| std::env::var("OPENAI_API_BASE").ok())?;

        let api_key = std::env::var("YUNXI_REFLECTION_LLM_API_KEY")
            .ok()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())?;

        let model = model_override
            .map(str::to_string)
            .or_else(|| std::env::var("YUNXI_REFLECTION_LLM_MODEL").ok())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

        Some(Self::new(base_url, api_key, model))
    }

    /// 同步调用 LLM 生成文本。
    ///
    /// 向 OpenAI 兼容的 `/v1/chat/completions` 端点发送请求，
    /// 返回 assistant 的文本回复。失败时返回 `Err(String)`。
    pub fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 2048
        });

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .map_err(|e| format!("LLM 请求失败: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            return Err(format!("LLM 返回错误 {}: {}", status, text));
        }

        let json: Value = response
            .json()
            .map_err(|e| format!("解析 LLM 响应失败: {e}"))?;

        json.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .map(str::to_string)
            .ok_or_else(|| "LLM 响应格式异常：缺少 choices[0].message.content".to_string())
    }

    /// 获取当前使用的模型名称。
    pub fn model(&self) -> &str {
        &self.model
    }
}

/// 基于 trait object 的 LLM 调用接口，支持运行时注入。
///
/// 提供与 `ReflectionLlmClient` 相同的功能，但通过 trait object 动态派发。
/// 当没有 LLM 可用时返回 `None`。
pub trait ReflectionLlm: Send + Sync {
    /// 同步调用 LLM 生成文本。失败时返回 `Err(String)`。
    fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String>;

    /// 获取模型名称。
    fn model_name(&self) -> &str {
        "unknown"
    }
}

impl ReflectionLlm for ReflectionLlmClient {
    fn generate(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        ReflectionLlmClient::generate(self, system_prompt, user_prompt)
    }

    fn model_name(&self) -> &str {
        self.model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        let client = ReflectionLlmClient::new(
            "https://api.example.com".to_string(),
            "test-key".to_string(),
            "gpt-4o-mini".to_string(),
        );
        assert_eq!(client.model(), "gpt-4o-mini");
    }

    #[test]
    fn test_trait_object() {
        let client: Box<dyn ReflectionLlm> = Box::new(ReflectionLlmClient::new(
            "https://api.example.com".to_string(),
            "test-key".to_string(),
            "test-model".to_string(),
        ));
        assert_eq!(client.model_name(), "test-model");
    }

    #[test]
    fn test_generate_with_invalid_url() {
        let client = ReflectionLlmClient::new(
            "http://invalid-host-that-does-not-exist.local".to_string(),
            "test-key".to_string(),
            "test-model".to_string(),
        );
        let result = client.generate("system", "user");
        assert!(result.is_err());
    }
}
