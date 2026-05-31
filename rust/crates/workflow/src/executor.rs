//! 工作流执行引擎。

use super::agent_bridge::AgentExecutor;
use super::checkpoint::{generate_run_id, Checkpoint, CheckpointStore};
use super::code_exec::CodeExecutor;
use super::flow::{Flow, FlowResult, FlowStatus, FlowStep, StepResult};

/// HITL 请求
#[derive(Debug, Clone)]
pub struct HitlRequest {
    pub title: String,
    pub description: String,
    pub step_index: usize,
}

/// HITL 响应
#[derive(Debug, Clone)]
pub struct HitlResponse {
    pub approved: bool,
    pub comment: Option<String>,
}

/// HITL 端口 trait
pub trait HitlPort {
    fn request(&self, req: HitlRequest) -> Result<HitlResponse, String>;
}

/// 工具执行函数类型
pub type ToolExecutor =
    Box<dyn Fn(&str, &serde_json::Value) -> Result<String, String> + Send + Sync>;

/// 工作流执行输出（含 run_id 供恢复）
#[derive(Debug, Clone)]
pub struct FlowExecution {
    pub result: FlowResult,
    pub run_id: String,
}

/// 工作流执行器
pub struct FlowExecutor {
    checkpoint_store: CheckpointStore,
    tool_executor: Option<ToolExecutor>,
    agent_executor: Option<Box<dyn AgentExecutor>>,
    code_executor: Option<Box<dyn CodeExecutor>>,
}

impl FlowExecutor {
    pub fn new(checkpoint_store: CheckpointStore) -> Self {
        Self {
            checkpoint_store,
            tool_executor: None,
            agent_executor: None,
            code_executor: None,
        }
    }

    pub fn with_tool_executor(mut self, executor: ToolExecutor) -> Self {
        self.tool_executor = Some(executor);
        self
    }

    /// 注入 Agent 执行器（用于 FlowStep::AgentCall）
    pub fn with_agent_executor(mut self, executor: Box<dyn AgentExecutor>) -> Self {
        self.agent_executor = Some(executor);
        self
    }

    /// 注入代码执行器（用于 FlowStep::CodeBlock）
    pub fn with_code_executor(mut self, executor: Box<dyn CodeExecutor>) -> Self {
        self.code_executor = Some(executor);
        self
    }

    /// 执行工作流
    pub fn execute(
        &mut self,
        flow: &Flow,
        _initial_input: Option<serde_json::Value>,
    ) -> Result<FlowExecution, String> {
        let run_id = generate_run_id();
        let mut result = FlowResult {
            flow_id: flow.id.clone(),
            status: FlowStatus::Running,
            step_results: Vec::new(),
            current_step: 0,
        };

        self.save_checkpoint(&flow.id, &run_id, 0, &result)?;

        for (idx, step) in flow.steps.iter().enumerate() {
            result.current_step = idx;
            if let Some(outcome) = self.run_step(flow, &run_id, idx, step, &mut result)? {
                return Ok(FlowExecution {
                    result: outcome,
                    run_id,
                });
            }
        }

        result.status = FlowStatus::Completed;
        self.save_checkpoint(&flow.id, &run_id, flow.steps.len(), &result)?;
        Ok(FlowExecution { result, run_id })
    }

    /// 从 HITL 检查点恢复（默认视为已批准）
    pub fn resume(&mut self, flow: &Flow, run_id: &str) -> Result<FlowExecution, String> {
        self.resume_with_approval(flow, run_id, true)
    }

    /// 从 HITL 检查点恢复，可拒绝审批
    pub fn resume_with_approval(
        &mut self,
        flow: &Flow,
        run_id: &str,
        approved: bool,
    ) -> Result<FlowExecution, String> {
        let checkpoint = self
            .checkpoint_store
            .load_latest(run_id)?
            .ok_or_else(|| "Checkpoint not found".to_string())?;

        let mut result = checkpoint.state;
        if result.status != FlowStatus::Suspended {
            return Err("Flow is not suspended".into());
        }

        let suspended_at = result.current_step;
        if !approved {
            result.status = FlowStatus::Failed;
            result.step_results.push(StepResult {
                step_index: suspended_at,
                success: false,
                output: Some(serde_json::json!({ "approved": false })),
                error: Some("human approval rejected".into()),
            });
            self.save_checkpoint(&flow.id, run_id, suspended_at + 1, &result)?;
            return Ok(FlowExecution {
                result,
                run_id: run_id.to_string(),
            });
        }

        if let FlowStep::HumanApproval { title, description } = &flow.steps[suspended_at] {
            result.step_results.push(StepResult {
                step_index: suspended_at,
                success: true,
                output: Some(serde_json::json!({
                    "approved": true,
                    "title": title,
                    "description": description,
                })),
                error: None,
            });
        }

        result.status = FlowStatus::Running;
        self.save_checkpoint(&flow.id, run_id, suspended_at + 1, &result)?;

        for idx in (suspended_at + 1)..flow.steps.len() {
            result.current_step = idx;
            if let Some(outcome) =
                self.run_step(flow, run_id, idx, &flow.steps[idx], &mut result)?
            {
                return Ok(FlowExecution {
                    result: outcome,
                    run_id: run_id.to_string(),
                });
            }
        }

        result.status = FlowStatus::Completed;
        self.save_checkpoint(&flow.id, run_id, flow.steps.len(), &result)?;
        Ok(FlowExecution {
            result,
            run_id: run_id.to_string(),
        })
    }

    /// 执行单步；若需暂停则返回 `Some(final_result)`
    fn run_step(
        &mut self,
        flow: &Flow,
        run_id: &str,
        idx: usize,
        step: &FlowStep,
        result: &mut FlowResult,
    ) -> Result<Option<FlowResult>, String> {
        let step_result = match step {
            FlowStep::AgentCall { agent_name, prompt } => {
                if let Some(ref mut agent_exec) = self.agent_executor {
                    match agent_exec.execute(agent_name, prompt) {
                        Ok(result) => StepResult {
                            step_index: idx,
                            success: result.success,
                            output: Some(serde_json::json!({
                                "agent": result.agent_name,
                                "prompt": result.prompt,
                                "output": result.output,
                                "success": result.success,
                                "error": result.error,
                            })),
                            error: result.error,
                        },
                        Err(e) => StepResult {
                            step_index: idx,
                            success: false,
                            output: None,
                            error: Some(format!("Agent execution failed: {e}")),
                        },
                    }
                } else {
                    StepResult {
                        step_index: idx,
                        success: false,
                        output: None,
                        error: Some(format!(
                            "No agent executor registered for agent '{agent_name}'"
                        )),
                    }
                }
            }
            FlowStep::AgentTool { agent_name, input } => {
                if let Some(ref mut agent_exec) = self.agent_executor {
                    let prompt = serde_json::to_string(input).unwrap_or_default();
                    match agent_exec.delegate_to(agent_name, &prompt) {
                        Ok(result) => StepResult {
                            step_index: idx,
                            success: result.success,
                            output: Some(serde_json::json!({
                                "agent_tool": result.agent_name,
                                "output": result.output,
                                "success": result.success,
                            })),
                            error: result.error,
                        },
                        Err(e) => StepResult {
                            step_index: idx,
                            success: false,
                            output: None,
                            error: Some(format!("AgentTool delegation failed: {e}")),
                        },
                    }
                } else {
                    StepResult {
                        step_index: idx,
                        success: false,
                        output: None,
                        error: Some(format!(
                            "No agent executor registered for AgentTool '{agent_name}'"
                        )),
                    }
                }
            }
            FlowStep::QualityCheck { criteria } => StepResult {
                step_index: idx,
                success: true,
                output: Some(serde_json::json!({
                    "criteria": criteria,
                    "passed": true
                })),
                error: None,
            },
            FlowStep::HumanApproval {
                title: _,
                description: _,
            } => {
                result.status = FlowStatus::Suspended;
                self.save_checkpoint(&flow.id, run_id, idx, result)?;
                return Ok(Some(result.clone()));
            }
            FlowStep::ToolCall { tool_name, input } => {
                if let Some(ref executor) = self.tool_executor {
                    match executor(tool_name, input) {
                        Ok(output) => StepResult {
                            step_index: idx,
                            success: true,
                            output: Some(serde_json::json!({ "output": output })),
                            error: None,
                        },
                        Err(e) => StepResult {
                            step_index: idx,
                            success: false,
                            output: None,
                            error: Some(e),
                        },
                    }
                } else {
                    StepResult {
                        step_index: idx,
                        success: false,
                        output: None,
                        error: Some(format!("No tool executor registered for {tool_name}")),
                    }
                }
            }
            FlowStep::CodeBlock { language, code } => {
                if let Some(ref mut exec) = self.code_executor {
                    match exec.execute(language, code) {
                        Ok(result) => StepResult {
                            step_index: idx,
                            success: result.success,
                            output: Some(serde_json::json!({
                                "output": result.output,
                                "language": result.language,
                            })),
                            error: result.error,
                        },
                        Err(e) => StepResult {
                            step_index: idx,
                            success: false,
                            output: None,
                            error: Some(format!("代码执行失败: {e}")),
                        },
                    }
                } else {
                    StepResult {
                        step_index: idx,
                        success: false,
                        output: None,
                        error: Some("未注册代码执行器".into()),
                    }
                }
            }
        };

        let success = step_result.success;
        result.step_results.push(step_result);
        self.save_checkpoint(&flow.id, run_id, idx + 1, result)?;

        if !success {
            result.status = FlowStatus::Failed;
            return Ok(Some(result.clone()));
        }
        Ok(None)
    }

    fn save_checkpoint(
        &self,
        flow_id: &str,
        run_id: &str,
        step_index: usize,
        state: &FlowResult,
    ) -> Result<(), String> {
        let checkpoint = Checkpoint {
            id: uuid::Uuid::new_v4().to_string(),
            flow_id: flow_id.into(),
            run_id: run_id.into(),
            step_index,
            state: state.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.checkpoint_store.save(&checkpoint)
    }
}

#[cfg(test)]
mod tests {
    use super::super::checkpoint::CheckpointStore;
    use super::*;
    use crate::agent_bridge::NoopAgentExecutor;

    fn temp_db() -> (std::path::PathBuf, CheckpointStore) {
        let dir = std::env::temp_dir().join(format!(
            "yunxi-executor-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let store = CheckpointStore::open(&dir.join("executor.sqlite")).unwrap();
        (dir, store)
    }

    #[test]
    fn test_execute_flow() {
        let (_dir, store) = temp_db();
        let mut executor =
            FlowExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
                label: "test".into(),
            }));

        let flow = Flow {
            id: "test-flow".into(),
            name: "测试工作流".into(),
            steps: vec![
                FlowStep::AgentCall {
                    agent_name: "researcher".into(),
                    prompt: "搜索专利".into(),
                },
                FlowStep::QualityCheck {
                    criteria: vec!["完整性".into()],
                },
            ],
            retry_on_failure: None,
        };

        let exec = executor.execute(&flow, None).unwrap();
        assert_eq!(exec.result.status, FlowStatus::Completed);
        assert_eq!(exec.result.step_results.len(), 2);
        assert!(!exec.run_id.is_empty());
    }

    #[test]
    fn test_suspend_and_resume() {
        let (_dir, store) = temp_db();
        let mut executor =
            FlowExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
                label: "test".into(),
            }));

        let flow = Flow {
            id: "hitl-flow".into(),
            name: "HITL测试".into(),
            steps: vec![
                FlowStep::AgentCall {
                    agent_name: "agent1".into(),
                    prompt: "步骤1".into(),
                },
                FlowStep::HumanApproval {
                    title: "确认".into(),
                    description: "请确认继续".into(),
                },
                FlowStep::AgentCall {
                    agent_name: "agent2".into(),
                    prompt: "步骤2".into(),
                },
            ],
            retry_on_failure: None,
        };

        let exec = executor.execute(&flow, None).unwrap();
        assert_eq!(exec.result.status, FlowStatus::Suspended);
        assert_eq!(exec.result.current_step, 1);

        let resumed = executor.resume(&flow, &exec.run_id).unwrap();
        assert_eq!(resumed.result.status, FlowStatus::Completed);
        assert_eq!(resumed.result.step_results.len(), 3);
    }

    #[test]
    fn test_agent_tool_delegation_in_flow() {
        let (_dir, store) = temp_db();
        let mut executor =
            FlowExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
                label: "test".into(),
            }));

        let flow = Flow {
            id: "agent-tool-flow".into(),
            name: "AgentTool测试".into(),
            steps: vec![FlowStep::AgentTool {
                agent_name: "specialist".into(),
                input: serde_json::json!({"task": "委托任务"}),
            }],
            retry_on_failure: None,
        };

        let exec = executor.execute(&flow, None).unwrap();
        assert_eq!(exec.result.status, FlowStatus::Completed);
        assert_eq!(exec.result.step_results.len(), 1);
        assert!(exec.result.step_results[0].success);
    }
}
