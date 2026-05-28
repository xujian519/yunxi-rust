use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RouterConfig {
    pub enabled: bool,
    pub threshold: Option<u8>,
    #[serde(default = "default_fallback_model")]
    pub fallback_model: String,
    pub logging: Option<LoggingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: String,
}

fn default_fallback_model() -> String {
    "deepseek-v4-pro".to_string()
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: Some(65),
            fallback_model: default_fallback_model(),
            logging: Some(LoggingConfig {
                enabled: true,
                level: "debug".to_string(),
            }),
        }
    }
}
