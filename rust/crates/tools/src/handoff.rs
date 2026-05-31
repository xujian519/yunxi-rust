//! Agent Handoff 机制（借鉴 OpenAI Swarm）
//!
//! 提供显式的智能体交接工具，替代隐式的提示词委派。
//! 支持上下文变量传递，确保 Agent 间协作的可控性和可追踪性。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent_roles::AgentRole;

/// Handoff 请求输入
#[derive(Debug, Deserialize)]
pub struct HandoffInput {
    /// 目标智能体角色
    pub target_role: String,
    /// 任务描述
    pub task_description: String,
    /// 交接原因
    #[serde(default)]
    pub reason: String,
    /// 共享上下文变量
    #[serde(default)]
    pub context_variables: HashMap<String, Value>,
}

/// Handoff 执行输出
#[derive(Debug, Serialize)]
pub struct HandoffOutput {
    /// 目标角色
    pub target_role: String,
    /// 角色名称
    pub role_name: String,
    /// 是否接受交接
    pub accepted: bool,
    /// 任务描述
    pub task_description: String,
    /// 系统提示词预览
    pub system_prompt_preview: String,
    /// 允许使用的工具列表
    pub allowed_tools: Vec<String>,
    /// 推荐模型
    pub preferred_model: String,
    /// 传递的上下文变量
    pub context_variables: HashMap<String, Value>,
    /// 交接时间戳
    pub handed_off_at: String,
}

/// Handoff 状态查询输入
#[derive(Debug, Deserialize)]
pub struct HandoffStatusInput {
    /// 要查询的角色
    pub role: String,
}

/// Handoff 状态输出
#[derive(Debug, Serialize)]
pub struct HandoffStatusOutput {
    pub role: String,
    pub role_name: String,
    pub allowed_tools: Vec<String>,
    pub preferred_model: String,
    pub routing_hint: String,
}

/// 执行 Agent Handoff
pub fn execute_handoff(input: &HandoffInput) -> Result<String, String> {
    let role = AgentRole::from_str_opt(&input.target_role).ok_or_else(|| {
        format!(
            "unknown role '{}'. Available: {}",
            input.target_role,
            available_roles()
        )
    })?;

    let prompt = role.system_prompt();
    let tools = role.allowed_tools();
    let model = role.preferred_model();

    let output = HandoffOutput {
        target_role: format!("{:?}", role),
        role_name: role.name().to_string(),
        accepted: true,
        task_description: input.task_description.clone(),
        system_prompt_preview: prompt.chars().take(200).collect(),
        allowed_tools: tools.into_iter().collect(),
        preferred_model: model.to_string(),
        context_variables: input.context_variables.clone(),
        handed_off_at: runtime::now_timestamp(),
    };

    serde_json::to_string(&output).map_err(|e| e.to_string())
}

/// 查询角色 Handoff 状态
pub fn execute_handoff_status(input: &HandoffStatusInput) -> Result<String, String> {
    let role = AgentRole::from_str_opt(&input.role).ok_or_else(|| {
        format!(
            "unknown role '{}'. Available: {}",
            input.role,
            available_roles()
        )
    })?;

    let output = HandoffStatusOutput {
        role: format!("{:?}", role),
        role_name: role.name().to_string(),
        allowed_tools: role.allowed_tools().into_iter().collect(),
        preferred_model: role.preferred_model().to_string(),
        routing_hint: role.routing_hint().to_string(),
    };

    serde_json::to_string(&output).map_err(|e| e.to_string())
}

/// 列出所有可用的 Handoff 角色
pub fn execute_handoff_list() -> Result<String, String> {
    let roles: Vec<serde_json::Value> = AgentRole::all()
        .iter()
        .map(|role| {
            serde_json::json!({
                "role": format!("{:?}", role),
                "name": role.name(),
                "routing_hint": role.routing_hint(),
                "preferred_model": role.preferred_model(),
                "tools_count": role.allowed_tools().len(),
            })
        })
        .collect();

    serde_json::to_string(&serde_json::json!({
        "roles": roles,
        "total": roles.len()
    }))
    .map_err(|e| e.to_string())
}

fn available_roles() -> String {
    AgentRole::all()
        .iter()
        .map(|r| format!("{:?}({})", r, r.name()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_to_retriever() {
        let result = execute_handoff(&HandoffInput {
            target_role: "Retriever".to_string(),
            task_description: "检索 AI 相关专利".to_string(),
            reason: "需要专业检索".to_string(),
            context_variables: {
                let mut vars = HashMap::new();
                vars.insert("query".to_string(), Value::String("AI patent".to_string()));
                vars
            },
        })
        .expect("handoff should succeed");

        let output: serde_json::Value = serde_json::from_str(&result).expect("valid json");
        assert_eq!(output["accepted"], true);
        assert_eq!(output["role_name"], "检索专家");
        assert!(output["allowed_tools"]
            .as_array()
            .expect("tools")
            .iter()
            .any(|t| t == "PatentSearch"));
        assert!(output["context_variables"]["query"].is_string());
    }

    #[test]
    fn handoff_rejects_unknown_role() {
        let err = execute_handoff(&HandoffInput {
            target_role: "UnknownRole".to_string(),
            task_description: "test".to_string(),
            reason: String::new(),
            context_variables: HashMap::new(),
        })
        .expect_err("should fail");
        assert!(err.contains("unknown role"));
    }

    #[test]
    fn handoff_status_returns_role_info() {
        let result = execute_handoff_status(&HandoffStatusInput {
            role: "analyzer".to_string(),
        })
        .expect("should succeed");

        let output: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(output["role_name"], "分析专家");
        assert!(!output["allowed_tools"]
            .as_array()
            .expect("tools")
            .is_empty());
    }

    #[test]
    fn handoff_list_returns_all_roles() {
        let result = execute_handoff_list().expect("should succeed");
        let output: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(output["total"], 9);
        let roles = output["roles"].as_array().expect("roles array");
        assert!(roles.iter().any(|r| r["role"] == "Retriever"));
    }
}
