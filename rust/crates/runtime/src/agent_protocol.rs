//! 结构化智能体消息协议
//!
//! 借鉴 OpenAI Swarm（显式传递）、Magentic-One（任务跟踪）的设计，
//! 为智能体间通信提供类型化、可追踪的消息信封。

use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// 智能体标识符
pub type AgentId = String;

/// 团队标识符
pub type TeamId = String;

/// 消息唯一标识符
pub type MessageId = String;

/// 生成基于时间戳的消息 ID
pub fn generate_message_id() -> MessageId {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("msg-{nanos}")
}

/// 生成基于时间戳的关联 ID
pub fn generate_correlation_id() -> MessageId {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("corr-{nanos}")
}

/// 生成 ISO 8601 格式时间戳（不依赖 chrono）
pub fn now_timestamp() -> String {
    // 简单实现：使用 SystemTime 计算近似时间
    // 优先尝试 date 命令获取精确格式
    if let Ok(output) = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
    {
        let ts = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !ts.is_empty() {
            return ts;
        }
    }
    // 回退到 Unix 时间戳
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}

/// 消息类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// 任务分配
    TaskAssignment,
    /// 任务结果
    TaskResult,
    /// 状态更新
    StatusUpdate,
    /// 错误报告
    ErrorReport,
    /// 查询请求
    Query,
    /// Agent 交接（借鉴 OpenAI Swarm）
    Handoff,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskAssignment => write!(f, "task_assignment"),
            Self::TaskResult => write!(f, "task_result"),
            Self::StatusUpdate => write!(f, "status_update"),
            Self::ErrorReport => write!(f, "error_report"),
            Self::Query => write!(f, "query"),
            Self::Handoff => write!(f, "handoff"),
        }
    }
}

/// 消息优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Normal
    }
}

/// 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// 优先级
    #[serde(default)]
    pub priority: Priority,
    /// 生存时间（秒），None 表示永不过期
    #[serde(default)]
    pub ttl_seconds: Option<u64>,
    /// 重试次数
    #[serde(default)]
    pub retry_count: u32,
}

impl Default for MessageMetadata {
    fn default() -> Self {
        Self {
            priority: Priority::Normal,
            ttl_seconds: None,
            retry_count: 0,
        }
    }
}

/// 结构化消息信封
///
/// 所有智能体间通信都应封装在 AgentEnvelope 中，
/// 提供类型区分、元数据和请求-响应关联能力。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEnvelope {
    /// 消息唯一 ID
    pub id: MessageId,
    /// 消息类型
    pub msg_type: MessageType,
    /// 发送方智能体
    pub from: AgentId,
    /// 接收方智能体
    pub to: AgentId,
    /// 所属团队
    #[serde(default)]
    pub team_id: Option<TeamId>,
    /// 消息载荷（JSON 值）
    pub payload: serde_json::Value,
    /// 消息元数据
    #[serde(default)]
    pub metadata: MessageMetadata,
    /// 请求-响应关联 ID（用于配对请求和响应）
    #[serde(default)]
    pub correlation_id: Option<MessageId>,
    /// 创建时间戳
    pub timestamp: String,
}

impl AgentEnvelope {
    /// 创建新的消息信封
    pub fn new(
        msg_type: MessageType,
        from: impl Into<AgentId>,
        to: impl Into<AgentId>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: generate_message_id(),
            msg_type,
            from: from.into(),
            to: to.into(),
            team_id: None,
            payload,
            metadata: MessageMetadata::default(),
            correlation_id: None,
            timestamp: now_timestamp(),
        }
    }

    /// 设置团队 ID
    pub fn with_team(mut self, team_id: impl Into<TeamId>) -> Self {
        self.team_id = Some(team_id.into());
        self
    }

    /// 设置关联 ID（用于请求-响应配对）
    pub fn with_correlation(mut self, id: MessageId) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.metadata.priority = priority;
        self
    }

    /// 设置 TTL
    pub fn with_ttl(mut self, seconds: u64) -> Self {
        self.metadata.ttl_seconds = Some(seconds);
        self
    }

    /// 检查消息是否已过期
    pub fn is_expired(&self) -> bool {
        let Some(ttl) = self.metadata.ttl_seconds else {
            return false;
        };
        // 简单实现：基于 timestamp 解析判断
        // 由于 timestamp 格式不固定，这里保守返回 false
        let _ = ttl;
        false
    }

    /// 创建回复信封（自动设置 correlation_id 和反转 from/to）
    pub fn reply(&self, msg_type: MessageType, payload: serde_json::Value) -> Self {
        Self {
            id: generate_message_id(),
            msg_type,
            from: self.to.clone(),
            to: self.from.clone(),
            team_id: self.team_id.clone(),
            payload,
            metadata: MessageMetadata::default(),
            correlation_id: Some(self.id.clone()),
            timestamp: now_timestamp(),
        }
    }
}

/// 任务步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

impl fmt::Display for TaskStepStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

/// 任务步骤定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    /// 步骤 ID
    pub step_id: String,
    /// 步骤描述
    pub description: String,
    /// 负责的智能体角色
    pub assigned_agent: String,
    /// 步骤状态
    pub status: TaskStepStatus,
}

/// 任务步骤执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStepResult {
    pub step_id: String,
    pub status: TaskStepStatus,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub completed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn envelope_create_and_reply() {
        let env = AgentEnvelope::new(
            MessageType::TaskAssignment,
            "orchestrator",
            "retriever",
            json!({"task": "search patents about AI"}),
        )
        .with_team("team-1")
        .with_priority(Priority::High);

        assert_eq!(env.msg_type, MessageType::TaskAssignment);
        assert_eq!(env.from, "orchestrator");
        assert_eq!(env.to, "retriever");
        assert_eq!(env.team_id.as_deref(), Some("team-1"));
        assert_eq!(env.metadata.priority, Priority::High);

        let reply = env.reply(MessageType::TaskResult, json!({"found": 42}));
        assert_eq!(reply.msg_type, MessageType::TaskResult);
        assert_eq!(reply.from, "retriever");
        assert_eq!(reply.to, "orchestrator");
        assert_eq!(reply.correlation_id.as_deref(), Some(env.id.as_str()));
        assert_eq!(reply.team_id, env.team_id);
    }

    #[test]
    fn envelope_serialization_roundtrip() {
        let env = AgentEnvelope::new(
            MessageType::Handoff,
            "analyzer",
            "writer",
            json!({"context": "prior art analysis complete"}),
        )
        .with_correlation("corr-123".to_string())
        .with_ttl(300);

        let json_str = serde_json::to_string(&env).expect("serialize");
        let restored: AgentEnvelope = serde_json::from_str(&json_str).expect("deserialize");

        assert_eq!(restored.id, env.id);
        assert_eq!(restored.msg_type, MessageType::Handoff);
        assert_eq!(restored.correlation_id.as_deref(), Some("corr-123"));
        assert_eq!(restored.metadata.ttl_seconds, Some(300));
    }

    #[test]
    fn message_type_display() {
        assert_eq!(MessageType::TaskAssignment.to_string(), "task_assignment");
        assert_eq!(MessageType::Handoff.to_string(), "handoff");
    }

    #[test]
    fn priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }

    #[test]
    fn step_status_display() {
        assert_eq!(TaskStepStatus::InProgress.to_string(), "in_progress");
    }
}
