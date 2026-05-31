//! 编排流工具（对接 `workflow` crate）

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use workflow::{generate_run_id, CheckpointStore, Flow, FlowExecutor, FlowStatus, FlowStep};

/// HITL 覆盖层展示用元数据（可从检查点补全）。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FlowHitlDisplayInfo {
    pub flow_name: Option<String>,
    pub current_step: Option<usize>,
    pub step_title: Option<String>,
    pub step_description: Option<String>,
}

/// 编排流输入
#[derive(Debug, Deserialize)]
pub struct FlowToolsInput {
    pub operation: String,
    #[serde(default)]
    pub flow_definition: Option<FlowDefinition>,
    #[serde(default, alias = "flowId")]
    pub flow_id: Option<String>,
    #[serde(default, alias = "runId")]
    pub run_id: Option<String>,
    #[serde(default)]
    pub input_data: Option<Value>,
    /// `resume_flow` 时是否批准 HITL（默认 true）
    #[serde(default)]
    pub approved: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FlowDefinition {
    pub name: String,
    pub steps: Vec<StepDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct StepDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default, alias = "stepType", alias = "step_type")]
    pub step_type: Option<String>,
    /// 自定义输入参数（覆盖默认值）
    #[serde(default)]
    pub input_data: Option<Value>,
}

static FLOWS: Mutex<Option<HashMap<String, Flow>>> = Mutex::new(None);

fn with_flows<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce(&mut HashMap<String, Flow>) -> Result<T, String>,
{
    let mut guard = FLOWS.lock().map_err(|e| e.to_string())?;
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    f(guard.as_mut().expect("flows map"))
}

pub fn checkpoint_db_path() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".yunxi/workflows/checkpoints.db"))
        .unwrap_or_else(|_| PathBuf::from(".yunxi/workflows/checkpoints.db"))
}

fn open_checkpoint_store() -> Result<CheckpointStore, String> {
    CheckpointStore::open(&checkpoint_db_path())
}

fn tool_executor() -> workflow::ToolExecutor {
    Box::new(|tool_name: &str, input: &Value| crate::execute_tool(tool_name, input))
}

fn step_from_def(s: StepDefinition) -> FlowStep {
    match s.step_type.as_deref() {
        Some("human_approval" | "hitl" | "approval") => FlowStep::HumanApproval {
            title: s.name.clone(),
            description: s.id,
        },
        Some("quality" | "quality_check") => FlowStep::QualityCheck {
            criteria: vec![s.name],
        },
        Some("agent" | "agent_call") => FlowStep::AgentCall {
            agent_name: s.tool.unwrap_or_else(|| "patent-analysis-agent".into()),
            prompt: s.name,
        },
        _ => {
            let tool = s.tool.unwrap_or_else(|| "KnowledgeSearch".into());
            let input = s
                .input_data
                .unwrap_or_else(|| json!({ "query": s.name, "stepId": s.id }));
            FlowStep::ToolCall {
                tool_name: tool,
                input,
            }
        }
    }
}

fn hitl_fields_from_flow(flow: &Flow, step_index: usize) -> FlowHitlDisplayInfo {
    let mut info = FlowHitlDisplayInfo {
        flow_name: Some(flow.name.clone()),
        current_step: Some(step_index),
        ..Default::default()
    };
    if let Some(FlowStep::HumanApproval { title, description }) = flow.steps.get(step_index) {
        info.step_title = Some(title.clone());
        info.step_description = Some(description.clone());
    }
    info
}

fn execution_json(flow: &Flow, exec: workflow::FlowExecution) -> Value {
    let suspended = exec.result.status == FlowStatus::Suspended;
    let step_index = exec.result.current_step;
    let mut body = json!({
        "flow_id": flow.id,
        "run_id": exec.run_id,
        "status": format!("{:?}", exec.result.status),
        "steps_completed": exec.result.step_results.len(),
        "suspended": suspended,
        "current_step": step_index,
        "step_results": exec.result.step_results,
        "resume_hint": if suspended {
            "调用 FlowTool resume_flow 并传入 run_id；或 /flow resume <flow_id> <run_id>"
        } else {
            ""
        },
    });
    if suspended {
        let hitl = hitl_fields_from_flow(flow, step_index);
        if let Some(name) = hitl.flow_name {
            body["flow_name"] = json!(name);
        }
        if let Some(title) = hitl.step_title {
            body["hitl_title"] = json!(title);
        }
        if let Some(desc) = hitl.step_description {
            body["hitl_description"] = json!(desc);
        }
    }
    body
}

/// 从检查点 + 流程注册表补全 HITL 展示信息（会话恢复时用）。
#[must_use]
pub fn lookup_flow_hitl_display(flow_id: &str, run_id: &str) -> FlowHitlDisplayInfo {
    let Ok(store) = open_checkpoint_store() else {
        return FlowHitlDisplayInfo::default();
    };
    let Ok(Some(cp)) = store.load_latest(run_id) else {
        return FlowHitlDisplayInfo::default();
    };
    if cp.state.status != FlowStatus::Suspended {
        return FlowHitlDisplayInfo::default();
    }
    let step_index = cp.state.current_step;
    with_flows(|registry| {
        let Some(flow) = registry.get(flow_id) else {
            return Ok(FlowHitlDisplayInfo {
                current_step: Some(step_index),
                ..Default::default()
            });
        };
        Ok(hitl_fields_from_flow(flow, step_index))
    })
    .unwrap_or_default()
}

/// 执行编排流操作
pub fn flow_tool(input: FlowToolsInput) -> Result<Value, String> {
    match input.operation.as_str() {
        "create_flow" => {
            let flow_def = input
                .flow_definition
                .ok_or("flow_definition required for create_flow")?;
            let flow_id = input
                .flow_id
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| format!("flow-{}", uuid::Uuid::new_v4()));
            let steps: Vec<FlowStep> = flow_def.steps.into_iter().map(step_from_def).collect();
            let flow = Flow {
                id: flow_id.clone(),
                name: flow_def.name,
                steps,
                retry_on_failure: Some(1),
            };
            with_flows(|registry| {
                registry.insert(flow_id.clone(), flow);
                Ok(json!({
                    "flow_id": flow_id,
                    "message": "工作流已创建"
                }))
            })
        }
        "execute_flow" => {
            let flow_id = input.flow_id.ok_or("flow_id required for execute_flow")?;
            with_flows(|registry| {
                let flow = registry
                    .get(&flow_id)
                    .cloned()
                    .ok_or_else(|| format!("Flow not found: {flow_id}"))?;
                let store = open_checkpoint_store()?;
                let mut executor = FlowExecutor::new(store).with_tool_executor(tool_executor());
                let exec = executor.execute(&flow, input.input_data)?;
                Ok(execution_json(&flow, exec))
            })
        }
        "resume_flow" => {
            let flow_id = input.flow_id.ok_or("flow_id required for resume_flow")?;
            let run_id = input
                .run_id
                .filter(|s| !s.trim().is_empty())
                .ok_or("run_id required for resume_flow（来自 execute_flow 返回）")?;
            let approved = input.approved.unwrap_or(true);
            with_flows(|registry| {
                let flow = registry
                    .get(&flow_id)
                    .cloned()
                    .ok_or_else(|| format!("Flow not found: {flow_id}"))?;
                let store = open_checkpoint_store()?;
                let mut executor = FlowExecutor::new(store).with_tool_executor(tool_executor());
                let exec = executor.resume_with_approval(&flow, &run_id, approved)?;
                Ok(execution_json(&flow, exec))
            })
        }
        "list_suspended" => with_flows(|registry| {
            let store = open_checkpoint_store()?;
            let mut items = Vec::new();
            for flow in registry.values() {
                for cp in store.list_by_flow(&flow.id)? {
                    if cp.state.status == FlowStatus::Suspended {
                        let hitl = hitl_fields_from_flow(flow, cp.state.current_step);
                        items.push(json!({
                            "flow_id": flow.id,
                            "flow_name": hitl.flow_name,
                            "run_id": cp.run_id,
                            "step_index": cp.state.current_step,
                            "hitl_title": hitl.step_title,
                            "hitl_description": hitl.step_description,
                            "created_at": cp.created_at,
                        }));
                    }
                }
            }
            Ok(json!({ "suspended": items, "count": items.len() }))
        }),
        "get_flow_status" => {
            let flow_id = input
                .flow_id
                .ok_or("flow_id required for get_flow_status")?;
            with_flows(|registry| {
                let flow = registry
                    .get(&flow_id)
                    .ok_or_else(|| format!("Flow not found: {flow_id}"))?;
                Ok(json!({
                    "flow_id": flow_id,
                    "name": flow.name,
                    "steps": flow.steps.len(),
                    "retry_on_failure": flow.retry_on_failure,
                }))
            })
        }
        "list_flows" => with_flows(|registry| {
            let flows: Vec<_> = registry
                .values()
                .map(|f| {
                    json!({
                        "flow_id": f.id,
                        "name": f.name,
                        "steps": f.steps.len(),
                    })
                })
                .collect();
            Ok(json!({ "flows": flows, "total": flows.len() }))
        }),
        "create_checkpoint" => {
            let flow_id = input.flow_id.ok_or("flow_id required")?;
            with_flows(|registry| {
                if !registry.contains_key(&flow_id) {
                    return Err(format!("Flow not found: {flow_id}"));
                }
                Ok(json!({
                    "run_id": generate_run_id(),
                    "flow_id": flow_id,
                    "checkpoint_db": checkpoint_db_path().display().to_string(),
                    "message": "execute_flow 会自动写入检查点"
                }))
            })
        }
        _ => Err(format!("Unknown flow operation: {}", input.operation)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_json_includes_hitl_when_suspended() {
        let flow = Flow {
            id: "f1".into(),
            name: "专利答复".into(),
            steps: vec![FlowStep::HumanApproval {
                title: "确认检索结果".into(),
                description: "step-2".into(),
            }],
            retry_on_failure: None,
        };
        let exec = workflow::FlowExecution {
            run_id: "run-1".into(),
            result: workflow::FlowResult {
                flow_id: "f1".into(),
                status: FlowStatus::Suspended,
                step_results: vec![],
                current_step: 0,
            },
        };
        let v = execution_json(&flow, exec);
        assert_eq!(v["suspended"], true);
        assert_eq!(v["hitl_title"], "确认检索结果");
        assert_eq!(v["flow_name"], "专利答复");
    }
}
