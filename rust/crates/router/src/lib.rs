//! 云熙智能体 - 专业领域路由引擎
//!
//! 检测用户输入的专业领域（专利/商标/版权/法律/通用），
//! 评估任务复杂度，并推荐合适的工作流和工具。

pub mod complexity;
pub mod config;
pub mod detector;
pub mod hebbian_hint;
pub mod types;
pub mod workflow_router;

pub use hebbian_hint::HebbianPathHint;
pub use types::{Complexity, Domain, RoutingDecision, WorkflowType};
