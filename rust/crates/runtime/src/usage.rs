use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::session::Session;

const DEFAULT_INPUT_COST_PER_MILLION: f64 = 15.0;
const DEFAULT_OUTPUT_COST_PER_MILLION: f64 = 75.0;
const DEFAULT_CACHE_CREATION_COST_PER_MILLION: f64 = 18.75;
const DEFAULT_CACHE_READ_COST_PER_MILLION: f64 = 1.5;

static CUSTOM_PRICING: OnceLock<BTreeMap<String, ModelPricing>> = OnceLock::new();

/// 注册自定义模型定价（从配置文件加载）
pub fn register_custom_pricing(pricing: BTreeMap<String, ModelPricing>) {
    CUSTOM_PRICING.set(pricing).ok();
}

/// 模型定价
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPricing {
    /// 每百万输入 token 的价格
    pub input_cost_per_million: f64,
    /// 每百万输出 token 的价格
    pub output_cost_per_million: f64,
    /// 每百万缓存创建 token 的价格
    pub cache_creation_cost_per_million: f64,
    /// 每百万缓存读取 token 的价格
    pub cache_read_cost_per_million: f64,
}

impl ModelPricing {
    /// 默认 Sonnet 层级定价
    #[must_use]
    pub const fn default_sonnet_tier() -> Self {
        Self {
            input_cost_per_million: DEFAULT_INPUT_COST_PER_MILLION,
            output_cost_per_million: DEFAULT_OUTPUT_COST_PER_MILLION,
            cache_creation_cost_per_million: DEFAULT_CACHE_CREATION_COST_PER_MILLION,
            cache_read_cost_per_million: DEFAULT_CACHE_READ_COST_PER_MILLION,
        }
    }
}

/// Token 使用统计
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenUsage {
    /// 输入 token 数量
    pub input_tokens: u32,
    /// 输出 token 数量
    pub output_tokens: u32,
    /// 缓存创建输入 token 数量
    pub cache_creation_input_tokens: u32,
    /// 缓存读取输入 token 数量
    pub cache_read_input_tokens: u32,
}

/// 用量费用估算
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UsageCostEstimate {
    /// 输入费用（美元）
    pub input_cost_usd: f64,
    /// 输出费用（美元）
    pub output_cost_usd: f64,
    /// 缓存创建费用（美元）
    pub cache_creation_cost_usd: f64,
    /// 缓存读取费用（美元）
    pub cache_read_cost_usd: f64,
}

impl UsageCostEstimate {
    /// 计算总费用
    ///
    /// # 返回
    /// 总费用（美元）
    #[must_use]
    pub fn total_cost_usd(self) -> f64 {
        self.input_cost_usd
            + self.output_cost_usd
            + self.cache_creation_cost_usd
            + self.cache_read_cost_usd
    }
}

/// 获取模型定价
///
/// # 参数
/// - `model`: 模型名称
///
/// # 返回
/// 模型定价（如果存在）
#[must_use]
pub fn pricing_for_model(model: &str) -> Option<ModelPricing> {
    let normalized = model.to_ascii_lowercase();

    // 先查自定义定价
    if let Some(custom) = CUSTOM_PRICING.get() {
        for (pattern, pricing) in custom {
            if normalized.contains(&pattern.to_ascii_lowercase()) {
                return Some(*pricing);
            }
        }
    }

    // 内置 Anthropic 定价
    if normalized.contains("haiku") {
        return Some(ModelPricing {
            input_cost_per_million: 1.0,
            output_cost_per_million: 5.0,
            cache_creation_cost_per_million: 1.25,
            cache_read_cost_per_million: 0.1,
        });
    }
    if normalized.contains("opus") {
        return Some(ModelPricing {
            input_cost_per_million: 15.0,
            output_cost_per_million: 75.0,
            cache_creation_cost_per_million: 18.75,
            cache_read_cost_per_million: 1.5,
        });
    }
    if normalized.contains("sonnet") {
        return Some(ModelPricing::default_sonnet_tier());
    }

    // 内置非 Anthropic 模型定价
    if normalized.contains("deepseek") {
        return Some(ModelPricing {
            input_cost_per_million: 1.0,
            output_cost_per_million: 2.0,
            cache_creation_cost_per_million: 0.0,
            cache_read_cost_per_million: 0.0,
        });
    }
    if normalized.contains("qwen") {
        return Some(ModelPricing {
            input_cost_per_million: 0.8,
            output_cost_per_million: 2.0,
            cache_creation_cost_per_million: 0.0,
            cache_read_cost_per_million: 0.0,
        });
    }
    if normalized.contains("gpt-4") {
        return Some(ModelPricing {
            input_cost_per_million: 30.0,
            output_cost_per_million: 60.0,
            cache_creation_cost_per_million: 0.0,
            cache_read_cost_per_million: 0.0,
        });
    }

    None
}

impl TokenUsage {
    /// 计算总 token 数量
    ///
    /// # 返回
    /// 总 token 数量
    #[must_use]
    pub fn total_tokens(self) -> u32 {
        self.input_tokens
            + self.output_tokens
            + self.cache_creation_input_tokens
            + self.cache_read_input_tokens
    }

    /// 估算费用（美元）
    ///
    /// # 返回
    /// 费用估算
    #[must_use]
    pub fn estimate_cost_usd(self) -> UsageCostEstimate {
        self.estimate_cost_usd_with_pricing(ModelPricing::default_sonnet_tier())
    }

    /// 使用指定定价估算费用
    ///
    /// # 参数
    /// - `pricing`: 模型定价
    ///
    /// # 返回
    /// 费用估算
    #[must_use]
    pub fn estimate_cost_usd_with_pricing(self, pricing: ModelPricing) -> UsageCostEstimate {
        UsageCostEstimate {
            input_cost_usd: cost_for_tokens(self.input_tokens, pricing.input_cost_per_million),
            output_cost_usd: cost_for_tokens(self.output_tokens, pricing.output_cost_per_million),
            cache_creation_cost_usd: cost_for_tokens(
                self.cache_creation_input_tokens,
                pricing.cache_creation_cost_per_million,
            ),
            cache_read_cost_usd: cost_for_tokens(
                self.cache_read_input_tokens,
                pricing.cache_read_cost_per_million,
            ),
        }
    }

    /// 生成摘要行
    ///
    /// # 参数
    /// - `label`: 标签
    ///
    /// # 返回
    /// 摘要行列表
    #[must_use]
    pub fn summary_lines(self, label: &str) -> Vec<String> {
        self.summary_lines_for_model(label, None)
    }

    /// 生成指定模型的摘要行
    ///
    /// # 参数
    /// - `label`: 标签
    /// - `model`: 模型名称（可选）
    ///
    /// # 返回
    /// 摘要行列表
    #[must_use]
    pub fn summary_lines_for_model(self, label: &str, model: Option<&str>) -> Vec<String> {
        let pricing = model.and_then(pricing_for_model);
        let cost = pricing.map_or_else(
            || self.estimate_cost_usd(),
            |pricing| self.estimate_cost_usd_with_pricing(pricing),
        );
        let model_suffix =
            model.map_or_else(String::new, |model_name| format!(" model={model_name}"));
        let pricing_suffix = if pricing.is_some() {
            ""
        } else if model.is_some() {
            " pricing=estimated-default"
        } else {
            ""
        };
        vec![
            format!(
                "{label}: total_tokens={} input={} output={} cache_write={} cache_read={} estimated_cost={}{}{}",
                self.total_tokens(),
                self.input_tokens,
                self.output_tokens,
                self.cache_creation_input_tokens,
                self.cache_read_input_tokens,
                format_usd(cost.total_cost_usd()),
                model_suffix,
                pricing_suffix,
            ),
            format!(
                "  cost breakdown: input={} output={} cache_write={} cache_read={}",
                format_usd(cost.input_cost_usd),
                format_usd(cost.output_cost_usd),
                format_usd(cost.cache_creation_cost_usd),
                format_usd(cost.cache_read_cost_usd),
            ),
        ]
    }
}

fn cost_for_tokens(tokens: u32, usd_per_million_tokens: f64) -> f64 {
    f64::from(tokens) / 1_000_000.0 * usd_per_million_tokens
}

/// 格式化美元金额
///
/// # 参数
/// - `amount`: 金额
///
/// # 返回
/// 格式化的美元字符串
#[must_use]
pub fn format_usd(amount: f64) -> String {
    format!("${amount:.4}")
}

/// 用量追踪器
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsageTracker {
    latest_turn: TokenUsage,
    cumulative: TokenUsage,
    turns: u32,
}

impl UsageTracker {
    /// 创建新的用量追踪器
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 从会话创建用量追踪器
    ///
    /// # 参数
    /// - `session`: 会话
    ///
    /// # 返回
    /// 用量追踪器
    #[must_use]
    pub fn from_session(session: &Session) -> Self {
        let mut tracker = Self::new();
        for message in &session.messages {
            if let Some(usage) = message.usage {
                tracker.record(usage);
            }
        }
        tracker
    }

    /// 记录用量
    ///
    /// # 参数
    /// - `usage`: Token 使用统计
    pub fn record(&mut self, usage: TokenUsage) {
        self.latest_turn = usage;
        self.cumulative.input_tokens += usage.input_tokens;
        self.cumulative.output_tokens += usage.output_tokens;
        self.cumulative.cache_creation_input_tokens += usage.cache_creation_input_tokens;
        self.cumulative.cache_read_input_tokens += usage.cache_read_input_tokens;
        self.turns += 1;
    }

    /// 获取当前回合用量
    ///
    /// # 返回
    /// 当前回合用量
    #[must_use]
    pub fn current_turn_usage(&self) -> TokenUsage {
        self.latest_turn
    }

    /// 获取累计用量
    ///
    /// # 返回
    /// 累计用量
    #[must_use]
    pub fn cumulative_usage(&self) -> TokenUsage {
        self.cumulative
    }

    /// 获取回合数
    ///
    /// # 返回
    /// 回合数
    #[must_use]
    pub fn turns(&self) -> u32 {
        self.turns
    }
}

#[cfg(test)]
mod tests {
    use super::{format_usd, pricing_for_model, TokenUsage, UsageTracker};
    use crate::session::{ContentBlock, ConversationMessage, MessageRole, Session};

    #[test]
    fn tracks_true_cumulative_usage() {
        let mut tracker = UsageTracker::new();
        tracker.record(TokenUsage {
            input_tokens: 10,
            output_tokens: 4,
            cache_creation_input_tokens: 2,
            cache_read_input_tokens: 1,
        });
        tracker.record(TokenUsage {
            input_tokens: 20,
            output_tokens: 6,
            cache_creation_input_tokens: 3,
            cache_read_input_tokens: 2,
        });

        assert_eq!(tracker.turns(), 2);
        assert_eq!(tracker.current_turn_usage().input_tokens, 20);
        assert_eq!(tracker.current_turn_usage().output_tokens, 6);
        assert_eq!(tracker.cumulative_usage().output_tokens, 10);
        assert_eq!(tracker.cumulative_usage().input_tokens, 30);
        assert_eq!(tracker.cumulative_usage().total_tokens(), 48);
    }

    #[test]
    fn computes_cost_summary_lines() {
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            cache_creation_input_tokens: 100_000,
            cache_read_input_tokens: 200_000,
        };

        let cost = usage.estimate_cost_usd();
        assert_eq!(format_usd(cost.input_cost_usd), "$15.0000");
        assert_eq!(format_usd(cost.output_cost_usd), "$37.5000");
        let lines = usage.summary_lines_for_model("usage", Some("claude-sonnet-4-20250514"));
        assert!(lines[0].contains("estimated_cost=$54.6750"));
        assert!(lines[0].contains("model=claude-sonnet-4-20250514"));
        assert!(lines[1].contains("cache_read=$0.3000"));
    }

    #[test]
    fn supports_model_specific_pricing() {
        let usage = TokenUsage {
            input_tokens: 1_000_000,
            output_tokens: 500_000,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        let haiku = pricing_for_model("claude-haiku-4-5-20251001").expect("haiku pricing");
        let opus = pricing_for_model("claude-opus-4-6").expect("opus pricing");
        let haiku_cost = usage.estimate_cost_usd_with_pricing(haiku);
        let opus_cost = usage.estimate_cost_usd_with_pricing(opus);
        assert_eq!(format_usd(haiku_cost.total_cost_usd()), "$3.5000");
        assert_eq!(format_usd(opus_cost.total_cost_usd()), "$52.5000");
    }

    #[test]
    fn marks_unknown_model_pricing_as_fallback() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 100,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };
        let lines = usage.summary_lines_for_model("usage", Some("custom-model"));
        assert!(lines[0].contains("pricing=estimated-default"));
    }

    #[test]
    fn reconstructs_usage_from_session_messages() {
        let session = Session {
            version: 1,
            messages: vec![ConversationMessage {
                role: MessageRole::Assistant,
                blocks: vec![ContentBlock::Text {
                    text: "done".to_string(),
                }],
                usage: Some(TokenUsage {
                    input_tokens: 5,
                    output_tokens: 2,
                    cache_creation_input_tokens: 1,
                    cache_read_input_tokens: 0,
                }),
            }],
        };

        let tracker = UsageTracker::from_session(&session);
        assert_eq!(tracker.turns(), 1);
        assert_eq!(tracker.cumulative_usage().total_tokens(), 8);
    }
}
