use std::collections::BTreeMap;

use crate::config::ProviderConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    Anthropic,
    OpenAICompatible {
        base_url: String,
        api_key_env: String,
    },
}

impl Provider {
    #[must_use]
    pub fn detect(model: &str, config_providers: &BTreeMap<String, ProviderConfig>) -> Self {
        let lower = model.to_ascii_lowercase();

        // 先检查配置文件中的 provider 匹配
        for provider_config in config_providers.values() {
            if lower.contains(&provider_config.match_pattern.to_ascii_lowercase()) {
                return Self::OpenAICompatible {
                    base_url: provider_config.base_url.clone(),
                    api_key_env: provider_config.api_key_env.clone(),
                };
            }
        }

        // Anthropic 系列
        if lower.starts_with("claude-")
            || lower.contains("opus")
            || lower.contains("sonnet")
            || lower.contains("haiku")
        {
            return Self::Anthropic;
        }

        // DeepSeek
        if lower.contains("deepseek") || lower.starts_with("ds-") {
            return Self::OpenAICompatible {
                base_url: "https://api.deepseek.com".to_string(),
                api_key_env: "DEEPSEEK_API_KEY".to_string(),
            };
        }

        // Qwen (通义千问)
        if lower.contains("qwen") {
            return Self::OpenAICompatible {
                base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
                api_key_env: "QWEN_API_KEY".to_string(),
            };
        }

        // Kimi / Moonshot
        if lower.contains("kimi") || lower.contains("moonshot") {
            return Self::OpenAICompatible {
                base_url: "https://api.moonshot.cn/v1".to_string(),
                api_key_env: "MOONSHOT_API_KEY".to_string(),
            };
        }

        // GLM (智谱)
        if lower.contains("glm") || lower.contains("chatglm") {
            return Self::OpenAICompatible {
                base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
                api_key_env: "GLM_API_KEY".to_string(),
            };
        }

        // OpenAI 原生
        if lower.starts_with("gpt-")
            || lower.starts_with("o1-")
            || lower.starts_with("o3-")
            || lower.starts_with("o4-")
            || lower.contains("chatgpt")
        {
            return Self::OpenAICompatible {
                base_url: "https://api.openai.com/v1".to_string(),
                api_key_env: "OPENAI_API_KEY".to_string(),
            };
        }

        // 默认走 Anthropic
        Self::Anthropic
    }

    #[must_use]
    pub fn is_anthropic(&self) -> bool {
        matches!(self, Self::Anthropic)
    }

    #[must_use]
    pub fn is_openai_compatible(&self) -> bool {
        matches!(self, Self::OpenAICompatible { .. })
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        match self {
            Self::Anthropic => "https://api.anthropic.com",
            Self::OpenAICompatible { base_url, .. } => base_url,
        }
    }

    #[must_use]
    pub fn api_key_env(&self) -> &str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::OpenAICompatible { api_key_env, .. } => api_key_env,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_providers() -> BTreeMap<String, ProviderConfig> {
        BTreeMap::new()
    }

    #[test]
    fn detects_anthropic_models() {
        let p = empty_providers();
        assert!(Provider::detect("claude-opus-4-6", &p).is_anthropic());
        assert!(Provider::detect("claude-sonnet-4-6", &p).is_anthropic());
        assert!(Provider::detect("claude-haiku-4-5-20251213", &p).is_anthropic());
    }

    #[test]
    fn detects_deepseek_models() {
        let p = empty_providers();
        let provider = Provider::detect("deepseek-chat", &p);
        assert!(provider.is_openai_compatible());
        let Provider::OpenAICompatible { base_url, .. } = provider else {
            panic!("expected OpenAICompatible");
        };
        assert_eq!(base_url, "https://api.deepseek.com");
    }

    #[test]
    fn detects_qwen_models() {
        let p = empty_providers();
        let provider = Provider::detect("qwen-plus", &p);
        assert!(provider.is_openai_compatible());
        let Provider::OpenAICompatible { base_url, .. } = provider else {
            panic!("expected OpenAICompatible");
        };
        assert!(base_url.contains("dashscope"));
    }

    #[test]
    fn detects_kimi_models() {
        let p = empty_providers();
        let provider = Provider::detect("moonshot-v1-auto", &p);
        assert!(provider.is_openai_compatible());
    }

    #[test]
    fn detects_glm_models() {
        let p = empty_providers();
        let provider = Provider::detect("glm-4-plus", &p);
        assert!(provider.is_openai_compatible());
    }

    #[test]
    fn detects_openai_models() {
        let p = empty_providers();
        assert!(Provider::detect("gpt-4o", &p).is_openai_compatible());
        assert!(Provider::detect("o1-preview", &p).is_openai_compatible());
        assert!(Provider::detect("o3-mini", &p).is_openai_compatible());
    }

    #[test]
    fn defaults_to_anthropic_for_unknown() {
        let p = empty_providers();
        assert!(Provider::detect("some-unknown-model", &p).is_anthropic());
    }

    #[test]
    fn config_provider_overrides_default() {
        let mut p = BTreeMap::new();
        p.insert(
            "custom".to_string(),
            ProviderConfig {
                base_url: "https://custom.api.com/v1".to_string(),
                api_key_env: "CUSTOM_API_KEY".to_string(),
                match_pattern: "my-model".to_string(),
            },
        );
        let provider = Provider::detect("my-model-v2", &p);
        assert!(provider.is_openai_compatible());
        let Provider::OpenAICompatible {
            base_url,
            api_key_env,
        } = provider
        else {
            panic!("expected OpenAICompatible");
        };
        assert_eq!(base_url, "https://custom.api.com/v1");
        assert_eq!(api_key_env, "CUSTOM_API_KEY");
    }
}
