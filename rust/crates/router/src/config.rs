//! 路由配置

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{Complexity, Domain};

/// 路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub mode_preference: ModePreference,
    pub workflow_preference: WorkflowPreference,
    pub enabled: bool,
    /// 领域→复杂度→(工具列表, 智能体列表) 的动态映射
    /// 空 HashMap 时使用默认内置规则
    #[serde(skip)]
    pub resource_map: HashMap<(Domain, Complexity), (Vec<String>, Vec<String>)>,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            mode_preference: ModePreference::A,
            workflow_preference: WorkflowPreference::A,
            enabled: true,
            resource_map: HashMap::new(),
        }
    }
}

impl RoutingConfig {
    /// 注册领域-复杂度对应的资源（工具+智能体）
    ///
    /// 会与内置规则合并，自定义条目优先。
    pub fn register_resources(
        &mut self,
        domain: Domain,
        complexity: Complexity,
        tools: Vec<String>,
        agents: Vec<String>,
    ) {
        self.resource_map
            .insert((domain, complexity), (tools, agents));
    }

    /// 获取指定领域+复杂度的资源组合（先查自定义再回退）
    pub fn resolve_resources(
        &self,
        domain: Domain,
        complexity: Complexity,
    ) -> Option<&(Vec<String>, Vec<String>)> {
        self.resource_map.get(&(domain, complexity))
    }
}

/// 模式偏好
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModePreference {
    #[default]
    A, // 默认模式
    B, // 询问/确认模式
    C, // 严格模式
}

/// 工作流偏好
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WorkflowPreference {
    #[default]
    A,
    B,
    C,
    D,
    E,
}
