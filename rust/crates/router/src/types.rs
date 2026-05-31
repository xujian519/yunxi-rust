//! 路由类型定义

use serde::{Deserialize, Serialize};

/// 专业领域
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    Patent,
    Trademark,
    Copyright,
    Legal,
    General,
}

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Patent => write!(f, "专利"),
            Self::Trademark => write!(f, "商标"),
            Self::Copyright => write!(f, "版权"),
            Self::Legal => write!(f, "法律"),
            Self::General => write!(f, "通用"),
        }
    }
}

/// 任务复杂度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Complexity {
    Simple,
    Medium,
    Complex,
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "简单"),
            Self::Medium => write!(f, "中等"),
            Self::Complex => write!(f, "复杂"),
        }
    }
}

/// 工作流类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowType {
    Direct,
    Hitl,
    PlanPlusHitl,
}

/// 路由决策
#[derive(Debug, Clone, Serialize)]
pub struct RoutingDecision {
    pub domain: Domain,
    pub complexity: Complexity,
    pub workflow: WorkflowType,
    pub suggested_tools: Vec<String>,
    pub suggested_agents: Vec<String>,
    pub confidence: f64,
    pub reasoning: String,
    /// 意图分类名称（如 "PATENT_DRAFTING"、"PATENT_SEARCH"）
    pub intent_name: String,
    /// 意图分类置信度 (0.0-1.0)
    pub intent_confidence: f64,
}
