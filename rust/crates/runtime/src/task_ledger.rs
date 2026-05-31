//! 任务跟踪与检查点
//!
//! 借鉴 LangGraph 的检查点和 Magentic-One 的任务进度簿，
//! 为多步骤专利流程提供进度跟踪和故障恢复能力。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::agent_protocol::{
    generate_message_id, now_timestamp, TaskStep, TaskStepResult, TaskStepStatus,
};

/// 任务检查点（可序列化的执行快照）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// 检查点 ID
    pub id: String,
    /// 创建时间
    pub created_at: String,
    /// 当前步骤索引
    pub current_step: usize,
    /// 已完成的步骤数量
    pub completed_count: usize,
}

/// 任务进度簿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLedger {
    /// 任务 ID
    pub task_id: String,
    /// 任务描述
    pub description: String,
    /// 执行计划（步骤列表）
    pub plan: Vec<TaskStep>,
    /// 已完成的步骤结果
    pub completed: Vec<TaskStepResult>,
    /// 当前步骤索引
    pub current_step: usize,
    /// 最新检查点
    pub checkpoint: Option<Checkpoint>,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

impl TaskLedger {
    /// 创建新的任务进度簿
    pub fn new(task_id: impl Into<String>, description: impl Into<String>) -> Self {
        let now = now_timestamp();
        Self {
            task_id: task_id.into(),
            description: description.into(),
            plan: Vec::new(),
            completed: Vec::new(),
            current_step: 0,
            checkpoint: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 添加执行步骤
    pub fn add_step(
        &mut self,
        description: impl Into<String>,
        assigned_agent: impl Into<String>,
    ) -> &mut Self {
        let step_id = format!("step-{}", self.plan.len());
        self.plan.push(TaskStep {
            step_id,
            description: description.into(),
            assigned_agent: assigned_agent.into(),
            status: TaskStepStatus::Pending,
        });
        self
    }

    /// 标记当前步骤为进行中
    pub fn start_current_step(&mut self) -> Result<(), String> {
        let step = self
            .plan
            .get_mut(self.current_step)
            .ok_or_else(|| "no current step".to_string())?;
        step.status = TaskStepStatus::InProgress;
        self.touch();
        Ok(())
    }

    /// 完成当前步骤并推进
    pub fn complete_current_step(
        &mut self,
        output: Option<serde_json::Value>,
    ) -> Result<(), String> {
        let step = self
            .plan
            .get(self.current_step)
            .ok_or_else(|| "no current step".to_string())?;

        let result = TaskStepResult {
            step_id: step.step_id.clone(),
            status: TaskStepStatus::Completed,
            output,
            error: None,
            completed_at: now_timestamp(),
        };

        // 标记步骤完成
        self.plan[self.current_step].status = TaskStepStatus::Completed;
        self.completed.push(result);
        self.current_step += 1;
        self.save_checkpoint();
        self.touch();
        Ok(())
    }

    /// 标记当前步骤失败
    pub fn fail_current_step(&mut self, error: impl Into<String>) -> Result<(), String> {
        let step = self
            .plan
            .get(self.current_step)
            .ok_or_else(|| "no current step".to_string())?;

        let result = TaskStepResult {
            step_id: step.step_id.clone(),
            status: TaskStepStatus::Failed,
            output: None,
            error: Some(error.into()),
            completed_at: now_timestamp(),
        };

        self.plan[self.current_step].status = TaskStepStatus::Failed;
        self.completed.push(result);
        self.touch();
        Ok(())
    }

    /// 获取当前步骤
    #[must_use]
    pub fn current_step(&self) -> Option<&TaskStep> {
        self.plan.get(self.current_step)
    }

    /// 是否所有步骤已完成
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.plan.len()
    }

    /// 完成进度百分比
    #[must_use]
    pub fn progress(&self) -> f64 {
        if self.plan.is_empty() {
            return 1.0;
        }
        let completed = self
            .plan
            .iter()
            .filter(|s| s.status == TaskStepStatus::Completed)
            .count();
        completed as f64 / self.plan.len() as f64
    }

    /// 保存检查点
    pub fn save_checkpoint(&mut self) {
        self.checkpoint = Some(Checkpoint {
            id: generate_message_id(),
            created_at: now_timestamp(),
            current_step: self.current_step,
            completed_count: self.completed.len(),
        });
    }

    /// 从最新检查点恢复
    pub fn restore_from_checkpoint(&mut self) -> Result<(), String> {
        let cp = self
            .checkpoint
            .as_ref()
            .ok_or_else(|| "no checkpoint available".to_string())?;

        // 移除检查点之后的结果
        self.completed.truncate(cp.completed_count);
        self.current_step = cp.current_step;

        // 重置从检查点开始的步骤状态
        for step in &mut self.plan {
            if step.status == TaskStepStatus::InProgress {
                step.status = TaskStepStatus::Pending;
            }
        }
        self.touch();
        Ok(())
    }

    /// 持久化到文件
    pub fn save_to_file(&self, dir: &PathBuf) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{}.json", self.task_id));
        let body = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, body).map_err(|e| e.to_string())
    }

    /// 从文件加载
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    /// 获取摘要信息
    #[must_use]
    pub fn summary(&self) -> TaskLedgerSummary {
        let total = self.plan.len();
        let completed = self
            .plan
            .iter()
            .filter(|s| s.status == TaskStepStatus::Completed)
            .count();
        let failed = self
            .plan
            .iter()
            .filter(|s| s.status == TaskStepStatus::Failed)
            .count();
        TaskLedgerSummary {
            task_id: self.task_id.clone(),
            description: self.description.clone(),
            total_steps: total,
            completed_steps: completed,
            failed_steps: failed,
            current_step_index: self.current_step,
            progress: self.progress(),
            has_checkpoint: self.checkpoint.is_some(),
        }
    }

    fn touch(&mut self) {
        self.updated_at = now_timestamp();
    }
}

/// 任务进度摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLedgerSummary {
    pub task_id: String,
    pub description: String,
    pub total_steps: usize,
    pub completed_steps: usize,
    pub failed_steps: usize,
    pub current_step_index: usize,
    pub progress: f64,
    pub has_checkpoint: bool,
}

/// 查找 ledger 存储目录
pub fn ledger_store_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("YUNXI_AGENT_STORE") {
        return PathBuf::from(dir).join("ledgers");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(format!("{home}/.yunxi/agents/ledgers"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_lifecycle() {
        let mut ledger = TaskLedger::new("task-1", "撰写专利申请");
        ledger.add_step("检索相关专利", "Retriever");
        ledger.add_step("分析技术特征", "Analyzer");
        ledger.add_step("撰写权利要求", "Writer");

        assert!(!ledger.is_complete());
        assert_eq!(ledger.progress(), 0.0);

        // 步骤 1
        ledger.start_current_step().unwrap();
        ledger
            .complete_current_step(Some(serde_json::json!({"found": 10})))
            .unwrap();
        assert_eq!(ledger.current_step, 1);
        assert!(ledger.checkpoint.is_some());

        // 步骤 2
        ledger.start_current_step().unwrap();
        ledger
            .complete_current_step(Some(serde_json::json!({"features": 5})))
            .unwrap();
        assert_eq!(ledger.current_step, 2);

        // 步骤 3
        ledger.start_current_step().unwrap();
        ledger
            .complete_current_step(Some(serde_json::json!({"claims": 10})))
            .unwrap();

        assert!(ledger.is_complete());
        assert_eq!(ledger.progress(), 1.0);
    }

    #[test]
    fn checkpoint_restore() {
        let mut ledger = TaskLedger::new("task-2", "恢复测试");
        ledger.add_step("步骤 A", "Retriever");
        ledger.add_step("步骤 B", "Analyzer");

        ledger.start_current_step().unwrap();
        ledger
            .complete_current_step(Some(serde_json::json!("ok")))
            .unwrap();
        assert!(ledger.checkpoint.is_some());

        // 模拟步骤 2 失败
        ledger.start_current_step().unwrap();
        ledger.fail_current_step("timeout").unwrap();

        assert_eq!(ledger.completed.len(), 2);
        assert_eq!(ledger.current_step, 1); // fail_current_step 不推进 current_step

        // 从检查点恢复
        ledger.restore_from_checkpoint().unwrap();
        assert_eq!(ledger.current_step, 1);
        assert_eq!(ledger.completed.len(), 1);
    }

    #[test]
    fn summary_reflects_state() {
        let mut ledger = TaskLedger::new("task-3", "摘要测试");
        ledger.add_step("A", "Retriever");
        ledger.add_step("B", "Writer");

        let summary = ledger.summary();
        assert_eq!(summary.total_steps, 2);
        assert_eq!(summary.completed_steps, 0);
        assert!(!summary.has_checkpoint);

        ledger.start_current_step().unwrap();
        ledger.complete_current_step(None).unwrap();

        let summary = ledger.summary();
        assert_eq!(summary.completed_steps, 1);
        assert!(summary.has_checkpoint);
    }

    #[test]
    fn file_persistence() {
        let dir = std::env::temp_dir().join(format!(
            "yunxi-ledger-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        let mut ledger = TaskLedger::new("persist-1", "持久化测试");
        ledger.add_step("步骤 1", "Retriever");
        ledger.start_current_step().unwrap();
        ledger
            .complete_current_step(Some(serde_json::json!("done")))
            .unwrap();

        ledger.save_to_file(&dir).unwrap();

        let loaded = TaskLedger::load_from_file(&dir.join("persist-1.json")).unwrap();
        assert_eq!(loaded.task_id, "persist-1");
        assert_eq!(loaded.completed.len(), 1);
        assert_eq!(loaded.current_step, 1);

        let _ = std::fs::remove_dir_all(dir);
    }
}
