//! 工作流类型定义

use serde::{Deserialize, Serialize};

/// 任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub state: TaskState,
    #[serde(default)]
    pub owner: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

/// 调度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub cron: String,
    pub prompt: String,
    pub recurring: bool,
}
