//! 分层错误恢复策略
//!
//! L1 - 工具级重试（指数退避）
//! L2 - Agent 级重试（切换策略后重试）
//! L3 - 工作流级恢复（检查点回滚 + 重规划）
//! L4 - 人类介入（HITL 模式兜底）

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 错误恢复层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryLevel {
    /// L1: 工具级重试
    ToolRetry,
    /// L2: Agent 级重试
    AgentRetry,
    /// L3: 工作流级恢复
    WorkflowRecovery,
    /// L4: 人类介入
    HumanIntervention,
}

impl std::fmt::Display for RecoveryLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ToolRetry => write!(f, "L1-tool"),
            Self::AgentRetry => write!(f, "L2-agent"),
            Self::WorkflowRecovery => write!(f, "L3-workflow"),
            Self::HumanIntervention => write!(f, "L4-human"),
        }
    }
}

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// 网络错误（可重试）
    Network,
    /// 超时错误（可重试）
    Timeout,
    /// 速率限制（可重试，需等待）
    RateLimit,
    /// 工具输入错误（不可重试，需修正输入）
    InvalidInput,
    /// 工具不存在（不可重试）
    ToolNotFound,
    /// 权限拒绝（不可重试）
    PermissionDenied,
    /// 内部错误（可能可重试）
    Internal,
}

impl ErrorCategory {
    /// 根据错误消息自动分类
    pub fn from_error_message(msg: &str) -> Self {
        let lower = msg.to_lowercase();
        if lower.contains("timeout") || lower.contains("timed out") {
            Self::Timeout
        } else if lower.contains("rate limit") || lower.contains("429") || lower.contains("too many requests") {
            Self::RateLimit
        } else if lower.contains("network") || lower.contains("connection") || lower.contains("dns") {
            Self::Network
        } else if lower.contains("not found") || lower.contains("unsupported tool") {
            Self::ToolNotFound
        } else if lower.contains("permission") || lower.contains("denied") || lower.contains("unauthorized") {
            Self::PermissionDenied
        } else if lower.contains("invalid") || lower.contains("parse") || lower.contains("deserialize") {
            Self::InvalidInput
        } else {
            Self::Internal
        }
    }

    /// 是否可重试
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Network | Self::Timeout | Self::RateLimit | Self::Internal)
    }

    /// 推荐的恢复层级
    #[must_use]
    pub fn recommended_recovery(&self) -> RecoveryLevel {
        match self {
            Self::Network | Self::Timeout | Self::RateLimit => RecoveryLevel::ToolRetry,
            Self::Internal => RecoveryLevel::AgentRetry,
            Self::InvalidInput => RecoveryLevel::AgentRetry,
            Self::ToolNotFound | Self::PermissionDenied => RecoveryLevel::HumanIntervention,
        }
    }
}

/// 重试策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始退避时间（毫秒）
    pub initial_backoff_ms: u64,
    /// 最大退避时间（毫秒）
    pub max_backoff_ms: u64,
    /// 退避乘数
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30_000,
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// 计算第 N 次重试的等待时间
    #[must_use]
    pub fn backoff_duration(&self, retry_count: u32) -> Duration {
        let millis = self.initial_backoff_ms as f64
            * self.backoff_multiplier.powi(retry_count as i32);
        let capped = millis.min(self.max_backoff_ms as f64);
        Duration::from_millis(capped as u64)
    }
}

/// 恢复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStrategy {
    /// 错误分类
    pub category: ErrorCategory,
    /// 恢复层级
    pub level: RecoveryLevel,
    /// 重试策略（如果是可重试的）
    pub retry_policy: RetryPolicy,
    /// 错误消息
    pub error_message: String,
    /// 建议的修复动作
    pub suggested_action: String,
}

impl RecoveryStrategy {
    /// 根据错误消息创建恢复策略
    pub fn from_error(error: &str) -> Self {
        let category = ErrorCategory::from_error_message(error);
        let level = category.recommended_recovery();
        let suggested_action = match category {
            ErrorCategory::Network => "检查网络连接后重试".to_string(),
            ErrorCategory::Timeout => "增加超时时间或简化请求".to_string(),
            ErrorCategory::RateLimit => "等待一段时间后重试".to_string(),
            ErrorCategory::InvalidInput => "检查工具输入参数格式".to_string(),
            ErrorCategory::ToolNotFound => "检查工具名称是否正确".to_string(),
            ErrorCategory::PermissionDenied => "检查权限配置或请求人工授权".to_string(),
            ErrorCategory::Internal => "查看详细错误日志，尝试重试".to_string(),
        };

        Self {
            category,
            level,
            retry_policy: RetryPolicy::default(),
            error_message: error.to_string(),
            suggested_action,
        }
    }

    /// 是否应该重试
    #[must_use]
    pub fn should_retry(&self, attempt: u32) -> bool {
        self.category.is_retryable() && attempt < self.retry_policy.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_categorization() {
        assert_eq!(ErrorCategory::from_error_message("connection timeout"), ErrorCategory::Timeout);
        assert_eq!(ErrorCategory::from_error_message("rate limit exceeded"), ErrorCategory::RateLimit);
        assert_eq!(ErrorCategory::from_error_message("network error"), ErrorCategory::Network);
        assert_eq!(ErrorCategory::from_error_message("unsupported tool: foo"), ErrorCategory::ToolNotFound);
        assert_eq!(ErrorCategory::from_error_message("permission denied"), ErrorCategory::PermissionDenied);
        assert_eq!(ErrorCategory::from_error_message("invalid json parse"), ErrorCategory::InvalidInput);
        assert_eq!(ErrorCategory::from_error_message("something went wrong"), ErrorCategory::Internal);
    }

    #[test]
    fn retryable_errors() {
        assert!(ErrorCategory::Network.is_retryable());
        assert!(ErrorCategory::Timeout.is_retryable());
        assert!(ErrorCategory::RateLimit.is_retryable());
        assert!(!ErrorCategory::InvalidInput.is_retryable());
        assert!(!ErrorCategory::ToolNotFound.is_retryable());
    }

    #[test]
    fn recovery_strategy_network() {
        let strategy = RecoveryStrategy::from_error("connection refused");
        assert_eq!(strategy.category, ErrorCategory::Network);
        assert_eq!(strategy.level, RecoveryLevel::ToolRetry);
        assert!(strategy.should_retry(0));
        assert!(strategy.should_retry(2));
        assert!(!strategy.should_retry(3));
    }

    #[test]
    fn backoff_calculation() {
        let policy = RetryPolicy::default();
        let d0 = policy.backoff_duration(0);
        let d1 = policy.backoff_duration(1);
        let d2 = policy.backoff_duration(2);

        assert_eq!(d0.as_millis(), 1000);
        assert_eq!(d1.as_millis(), 2000);
        assert_eq!(d2.as_millis(), 4000);
    }

    #[test]
    fn backoff_capped() {
        let policy = RetryPolicy {
            max_backoff_ms: 3000,
            ..RetryPolicy::default()
        };
        let d = policy.backoff_duration(10);
        assert!(d.as_millis() <= 3000);
    }
}
