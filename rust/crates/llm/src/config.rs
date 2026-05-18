use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::LlmError;

#[derive(Debug, Clone, Default)]
pub struct LlmConfig {
    pub providers: BTreeMap<String, ProviderConfig>,
    pub aliases: BTreeMap<String, String>,
    pub pricing: BTreeMap<String, PricingConfig>,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub base_url: String,
    pub api_key_env: String,
    pub match_pattern: String,
}

#[derive(Debug, Clone)]
pub struct PricingConfig {
    pub input_cost_per_million: f64,
    pub output_cost_per_million: f64,
}

impl LlmConfig {
    /// 加载 LLM 配置
    ///
    /// # Errors
    ///
    /// - 如果配置加载失败,返回 Llm 错误
    pub fn load() -> Result<Self, LlmError> {
        let config_path = Self::discover_config_path();
        let Some(path) = config_path else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        Self::load_from_path(&path)
    }

    fn discover_config_path() -> Option<PathBuf> {
        // 1. 环境变量指定路径
        if let Ok(path) = std::env::var("YUNXI_LLM_CONFIG") {
            return Some(PathBuf::from(path));
        }

        // 2. ~/.yunxi/llm.toml
        if let Ok(home) = std::env::var("HOME") {
            let yunxi_path = PathBuf::from(home).join(".yunxi").join("llm.toml");
            if yunxi_path.exists() {
                return Some(yunxi_path);
            }
        }

        // 3. ~/.claude/llm.toml
        if let Ok(home) = std::env::var("HOME") {
            let claude_path = PathBuf::from(home).join(".claude").join("llm.toml");
            if claude_path.exists() {
                return Some(claude_path);
            }
        }

        None
    }

    fn load_from_path(path: &PathBuf) -> Result<Self, LlmError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            LlmError::config(format!("failed to read config {}: {e}", path.display()))
        })?;
        Self::parse(&content)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn parse(content: &str) -> Result<Self, LlmError> {
        let mut config = Self::default();
        let mut current_section = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // 跳过空行和注释
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Section header
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                current_section = trimmed[1..trimmed.len() - 1].to_string();
                continue;
            }

            // Key = Value
            let Some((key, value)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim().trim_matches('"');

            if current_section == "aliases" {
                config.aliases.insert(key.to_string(), value.to_string());
            } else if let Some(provider_name) = current_section.strip_prefix("providers.") {
                let entry = config
                    .providers
                    .entry(provider_name.to_string())
                    .or_insert_with(|| ProviderConfig {
                        base_url: String::new(),
                        api_key_env: String::new(),
                        match_pattern: provider_name.to_string(),
                    });
                match key {
                    "base_url" => entry.base_url = value.to_string(),
                    "api_key_env" => entry.api_key_env = value.to_string(),
                    "match_pattern" => entry.match_pattern = value.to_string(),
                    _ => {}
                }
            } else if let Some(model_name) = current_section.strip_prefix("pricing.") {
                let entry = config
                    .pricing
                    .entry(model_name.to_string())
                    .or_insert_with(|| PricingConfig {
                        input_cost_per_million: 0.0,
                        output_cost_per_million: 0.0,
                    });
                match key {
                    "input_cost_per_million" => {
                        entry.input_cost_per_million = value.parse().unwrap_or(0.0);
                    }
                    "output_cost_per_million" => {
                        entry.output_cost_per_million = value.parse().unwrap_or(0.0);
                    }
                    _ => {}
                }
            }
        }

        Ok(config)
    }

    pub fn resolve_alias(&self, model: &str) -> String {
        self.aliases
            .get(model)
            .map_or_else(|| model.to_string(), ToString::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_config() {
        let config = LlmConfig::parse("").unwrap();
        assert!(config.providers.is_empty());
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn parses_aliases() {
        let config = LlmConfig::parse(
            r#"
[aliases]
ds = "deepseek-chat"
qwen = "qwen-plus"
"#,
        )
        .unwrap();
        assert_eq!(config.resolve_alias("ds"), "deepseek-chat");
        assert_eq!(config.resolve_alias("qwen"), "qwen-plus");
        assert_eq!(config.resolve_alias("unknown"), "unknown");
    }

    #[test]
    fn parses_providers() {
        let config = LlmConfig::parse(
            r#"
[providers.deepseek]
base_url = "https://api.deepseek.com"
api_key_env = "DEEPSEEK_API_KEY"
match_pattern = "deepseek"
"#,
        )
        .unwrap();
        let provider = config.providers.get("deepseek").expect("deepseek provider");
        assert_eq!(provider.base_url, "https://api.deepseek.com");
        assert_eq!(provider.api_key_env, "DEEPSEEK_API_KEY");
        assert_eq!(provider.match_pattern, "deepseek");
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn parses_pricing() {
        let config = LlmConfig::parse(
            r"
[pricing.deepseek-chat]
input_cost_per_million = 1.0
output_cost_per_million = 2.0
",
        )
        .unwrap();
        let pricing = config.pricing.get("deepseek-chat").expect("pricing");
        assert_eq!(pricing.input_cost_per_million, 1.0);
        assert_eq!(pricing.output_cost_per_million, 2.0);
    }

    #[test]
    fn defaults_match_pattern_to_provider_name() {
        let config = LlmConfig::parse(
            r#"
[providers.myprovider]
base_url = "https://my.api.com/v1"
api_key_env = "MY_API_KEY"
"#,
        )
        .unwrap();
        let provider = config.providers.get("myprovider").expect("provider");
        assert_eq!(provider.match_pattern, "myprovider");
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let config = LlmConfig::parse(
            r#"
# This is a comment

[aliases]
# Another comment
ds = "deepseek-chat"
"#,
        )
        .unwrap();
        assert_eq!(config.resolve_alias("ds"), "deepseek-chat");
    }
}
