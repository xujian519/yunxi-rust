//! 声明式流程配置 — 从 TOML 文件加载 Flow/FlowGraph 定义。
//!
//! 参考 CrewAI 的 YAML 配置方式，以 TOML 格式定义工作流。

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::flow::{Flow, FlowStep};
use super::graph::{Condition, FlowEdge, FlowGraph, FlowNode};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlowConfigFile {
    flow: FlowConfigDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlowConfigDef {
    id: String,
    name: String,
    #[serde(default)]
    retry_on_failure: Option<u32>,
    #[serde(default)]
    mode: ConfigMode,
    #[serde(default)]
    steps: Vec<ConfigStep>,
    /// DAG 模式专属
    #[serde(default)]
    nodes: Vec<ConfigNode>,
    /// DAG 模式专属
    #[serde(default)]
    edges: Vec<ConfigEdge>,
    /// DAG 模式专属
    #[serde(default)]
    entry_node: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
enum ConfigMode {
    #[default]
    Linear,
    Graph,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigStep {
    #[serde(rename = "type")]
    step_type: String,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    input: Option<serde_json::Value>,
    #[serde(default)]
    criteria: Option<Vec<String>>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigNode {
    id: String,
    #[serde(rename = "type")]
    step_type: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    agent: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    input: Option<serde_json::Value>,
    #[serde(default)]
    criteria: Option<Vec<String>>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigEdge {
    from: String,
    to: String,
    #[serde(default = "default_condition_str")]
    condition: String,
}

fn default_condition_str() -> String {
    "always".to_string()
}

impl FlowConfigFile {
    fn parse_step(config: &ConfigStep) -> Result<FlowStep, String> {
        match config.step_type.as_str() {
            "agent_call" => Ok(FlowStep::AgentCall {
                agent_name: config
                    .agent
                    .clone()
                    .ok_or_else(|| "agent_call 缺少 agent 字段".to_string())?,
                prompt: config
                    .prompt
                    .clone()
                    .ok_or_else(|| "agent_call 缺少 prompt 字段".to_string())?,
            }),
            "agent_tool" => Ok(FlowStep::AgentTool {
                agent_name: config
                    .agent
                    .clone()
                    .ok_or_else(|| "agent_tool 缺少 agent 字段".to_string())?,
                input: config.input.clone().unwrap_or(serde_json::json!({})),
            }),
            "tool_call" => Ok(FlowStep::ToolCall {
                tool_name: config
                    .tool
                    .clone()
                    .ok_or_else(|| "tool_call 缺少 tool 字段".to_string())?,
                input: config.input.clone().unwrap_or(serde_json::json!({})),
            }),
            "quality_check" => Ok(FlowStep::QualityCheck {
                criteria: config.criteria.clone().unwrap_or_default(),
            }),
            "human_approval" => Ok(FlowStep::HumanApproval {
                title: config
                    .title
                    .clone()
                    .ok_or_else(|| "human_approval 缺少 title 字段".to_string())?,
                description: config.description.clone().unwrap_or_default(),
            }),
            "code_block" => Ok(FlowStep::CodeBlock {
                language: config
                    .language
                    .clone()
                    .unwrap_or_else(|| "python".to_string()),
                code: config
                    .code
                    .clone()
                    .ok_or_else(|| "code_block 缺少 code 字段".to_string())?,
            }),
            other => Err(format!("未知步骤类型: {other}")),
        }
    }

    fn parse_node(config: &ConfigNode) -> Result<FlowNode, String> {
        let step = match config.step_type.as_str() {
            "agent_call" => FlowStep::AgentCall {
                agent_name: config
                    .agent
                    .clone()
                    .ok_or_else(|| "agent_call 缺少 agent 字段".to_string())?,
                prompt: config
                    .prompt
                    .clone()
                    .ok_or_else(|| "agent_call 缺少 prompt 字段".to_string())?,
            },
            "agent_tool" => FlowStep::AgentTool {
                agent_name: config
                    .agent
                    .clone()
                    .ok_or_else(|| "agent_tool 缺少 agent 字段".to_string())?,
                input: config.input.clone().unwrap_or(serde_json::json!({})),
            },
            "tool_call" => FlowStep::ToolCall {
                tool_name: config
                    .tool
                    .clone()
                    .ok_or_else(|| "tool_call 缺少 tool 字段".to_string())?,
                input: config.input.clone().unwrap_or(serde_json::json!({})),
            },
            "quality_check" => FlowStep::QualityCheck {
                criteria: config.criteria.clone().unwrap_or_default(),
            },
            "human_approval" => FlowStep::HumanApproval {
                title: config
                    .title
                    .clone()
                    .ok_or_else(|| "human_approval 缺少 title 字段".to_string())?,
                description: config.description.clone().unwrap_or_default(),
            },
            "code_block" => FlowStep::CodeBlock {
                language: config
                    .language
                    .clone()
                    .unwrap_or_else(|| "python".to_string()),
                code: config
                    .code
                    .clone()
                    .ok_or_else(|| "code_block 缺少 code 字段".to_string())?,
            },
            other => return Err(format!("未知节点类型: {other}")),
        };

        Ok(FlowNode {
            id: config.id.clone(),
            step,
            label: config.label.clone(),
        })
    }

    fn parse_condition(s: &str) -> Condition {
        match s {
            "on_success" => Condition::OnSuccess,
            "on_failure" => Condition::OnFailure,
            _ => Condition::Always,
        }
    }

    fn to_flow(&self) -> Result<Flow, String> {
        let config = &self.flow;
        let steps: Result<Vec<FlowStep>, String> =
            config.steps.iter().map(Self::parse_step).collect();

        Ok(Flow {
            id: config.id.clone(),
            name: config.name.clone(),
            steps: steps?,
            retry_on_failure: config.retry_on_failure,
        })
    }

    fn to_graph(&self) -> Result<FlowGraph, String> {
        let config = &self.flow;
        let nodes: Result<Vec<FlowNode>, String> =
            config.nodes.iter().map(Self::parse_node).collect();
        let edges: Vec<FlowEdge> = config
            .edges
            .iter()
            .map(|e| FlowEdge {
                from: e.from.clone(),
                to: e.to.clone(),
                condition: Self::parse_condition(&e.condition),
            })
            .collect();

        Ok(FlowGraph {
            id: config.id.clone(),
            name: config.name.clone(),
            entry_node: config.entry_node.clone(),
            nodes: nodes?,
            edges,
            retry_on_failure: config.retry_on_failure,
        })
    }
}

pub fn load_flow_from_toml(path: &Path) -> Result<Flow, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("读取文件失败 {}: {e}", path.display()))?;
    let config: FlowConfigFile =
        toml::from_str(&content).map_err(|e| format!("解析 TOML 失败: {e}"))?;
    config.to_flow()
}

pub fn load_graph_from_toml(path: &Path) -> Result<FlowGraph, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("读取文件失败 {}: {e}", path.display()))?;
    let config: FlowConfigFile =
        toml::from_str(&content).map_err(|e| format!("解析 TOML 失败: {e}"))?;
    config.to_graph()
}

pub fn parse_flow_from_str(toml_str: &str) -> Result<Flow, String> {
    let config: FlowConfigFile =
        toml::from_str(toml_str).map_err(|e| format!("解析 TOML 失败: {e}"))?;
    config.to_flow()
}

pub fn parse_graph_from_str(toml_str: &str) -> Result<FlowGraph, String> {
    let config: FlowConfigFile =
        toml::from_str(toml_str).map_err(|e| format!("解析 TOML 失败: {e}"))?;
    config.to_graph()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINEAR_TOML: &str = r#"
[flow]
id = "patent-drafting"
name = "专利申请撰写流程"
retry_on_failure = 3
mode = "linear"

[[flow.steps]]
type = "agent_call"
agent = "invention-understander"
prompt = "理解技术交底书"

[[flow.steps]]
type = "agent_call"
agent = "prior-art-searcher"
prompt = "检索现有技术"

[[flow.steps]]
type = "quality_check"
criteria = ["新颖性", "创造性"]

[[flow.steps]]
type = "human_approval"
title = "最终审批"
description = "请确认专利申请草案"
"#;

    #[test]
    fn test_parse_linear_flow() {
        let flow = parse_flow_from_str(LINEAR_TOML).unwrap();
        assert_eq!(flow.id, "patent-drafting");
        assert_eq!(flow.name, "专利申请撰写流程");
        assert_eq!(flow.steps.len(), 4);
        assert_eq!(flow.retry_on_failure, Some(3));

        match &flow.steps[0] {
            FlowStep::AgentCall {
                agent_name, prompt, ..
            } => {
                assert_eq!(agent_name, "invention-understander");
                assert_eq!(prompt, "理解技术交底书");
            }
            _ => panic!("预期 AgentCall"),
        }
    }

    const GRAPH_TOML: &str = r#"
[flow]
id = "patent-analysis-graph"
name = "专利分析并行图"
mode = "graph"
entry_node = "start"

[[flow.nodes]]
id = "start"
type = "agent_call"
agent = "coordinator"
prompt = "分析用户需求"

[[flow.nodes]]
id = "novelty_search"
type = "agent_call"
agent = "novelty-searcher"
prompt = "并行检索新颖性"
label = "新颖性检索"

[[flow.nodes]]
id = "infringement_check"
type = "agent_call"
agent = "infringement-checker"
prompt = "并行检索侵权风险"
label = "侵权检查"

[[flow.nodes]]
id = "synthesize"
type = "agent_call"
agent = "analyst"
prompt = "综合检索结果"
label = "综合分析"

[[flow.nodes]]
id = "human_review"
type = "human_approval"
title = "审核"
description = "请审核分析报告"

[[flow.edges]]
from = "start"
to = "novelty_search"
condition = "always"

[[flow.edges]]
from = "start"
to = "infringement_check"
condition = "always"

[[flow.edges]]
from = "novelty_search"
to = "synthesize"
condition = "on_success"

[[flow.edges]]
from = "infringement_check"
to = "synthesize"
condition = "on_success"

[[flow.edges]]
from = "synthesize"
to = "human_review"
condition = "always"
"#;

    #[test]
    fn test_parse_graph_flow() {
        let graph = parse_graph_from_str(GRAPH_TOML).unwrap();
        assert_eq!(graph.id, "patent-analysis-graph");
        assert_eq!(graph.nodes.len(), 5);
        assert_eq!(graph.edges.len(), 5);
        assert_eq!(graph.entry_node.as_deref(), Some("start"));

        let levels = graph.topological_levels().unwrap();
        assert_eq!(levels.len(), 4, "应有 4 层：start → 并行检索 → 综合 → 审批");
        assert_eq!(levels[0], vec!["start"]);
        assert_eq!(levels[1].len(), 2); // novelty_search + infringement_check 并行
        assert_eq!(levels[2], vec!["synthesize"]);
        assert_eq!(levels[3], vec!["human_review"]);
    }

    #[test]
    fn test_parse_mixed_graph_with_agent_tool() {
        let toml_str = r#"
[flow]
id = "delegation-flow"
name = "委托模式测试"
mode = "graph"

[[flow.nodes]]
id = "main"
type = "agent_call"
agent = "orchestrator"
prompt = "主任务"

[[flow.nodes]]
id = "delegate"
type = "agent_tool"
agent = "specialist"
input = { task = "子任务" }

[[flow.edges]]
from = "main"
to = "delegate"
"#;
        let graph = parse_graph_from_str(toml_str).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        match &graph.nodes[1].step {
            FlowStep::AgentTool { agent_name, .. } => {
                assert_eq!(agent_name, "specialist");
            }
            _ => panic!("预期 AgentTool"),
        }
    }
}
