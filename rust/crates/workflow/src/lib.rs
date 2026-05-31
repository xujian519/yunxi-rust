//! 云熙智能体 - 工作流引擎
//!
//! 提供任务生命周期管理、线性/DAG 工作流编排、
//! 子 Agent 委托、Code-First 执行、声明式 TOML 配置和编排器闭环。

pub mod agent_bridge;
pub mod checkpoint;
pub mod code_exec;
pub mod config;
pub mod executor;
pub mod flow;
pub mod graph;
pub mod graph_executor;
pub mod orchestrator;
pub mod plan;
pub mod scheduler;
pub mod task;
pub mod types;

pub use agent_bridge::{
    AgentExecutionResult, AgentExecutor, AgentFallbackFn, MultiAgentExecutor, NoopAgentExecutor,
};
pub use checkpoint::{generate_run_id, Checkpoint, CheckpointStore};
pub use code_exec::{BuiltinPatentChecker, CodeExecutionResult, CodeExecutor, NoopCodeExecutor};
pub use config::{
    load_flow_from_toml, load_graph_from_toml, parse_flow_from_str, parse_graph_from_str,
};
pub use executor::{
    FlowExecution, FlowExecutor, HitlPort, HitlRequest, HitlResponse, ToolExecutor,
};
pub use flow::{Flow, FlowResult, FlowStatus, FlowStep, StepResult};
pub use graph::{Condition, FlowEdge, FlowGraph, FlowNode, GraphNodeResult};
pub use graph_executor::{GraphExecution, GraphExecutor};
pub use orchestrator::{OrchestrationResult, OrchestrationStatus, Orchestrator};
pub use plan::{
    ExecutionPlan, NoopPlanGenerator, PlanGenerator, PlanStep, PlanStepStatus, RoutingHint,
    WorkflowType,
};
pub use scheduler::{now_unix, parse_schedule_interval_secs, ScheduleRegistry, ScheduledJob};
