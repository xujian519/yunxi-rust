//! 任务管理
//!
//! 基于文件的任务生命周期管理：创建、查询、更新、删除。

use crate::types::{Task, TaskState};
use std::path::PathBuf;

/// 任务管理器
pub struct TaskManager {
    task_dir: PathBuf,
}

impl TaskManager {
    pub fn new(task_dir: &str) -> Self {
        Self {
            task_dir: PathBuf::from(task_dir),
        }
    }

    /// 使用默认路径 ~/.yunxi/tasks/
    pub fn default_path() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Self::new(&format!("{home}/.yunxi/tasks"))
    }

    /// 确保目录存在
    pub fn ensure_dir(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.task_dir).map_err(|e| format!("创建任务目录失败: {e}"))?;
        Ok(())
    }

    /// 创建新任务
    pub fn create(&self, title: &str, description: &str) -> Result<Task, String> {
        self.ensure_dir()?;
        let id = format!("task-{}", chrono_now_ts());
        let task = Task {
            id: id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            state: TaskState::Pending,
            owner: None,
            created_at: chrono_now(),
            completed_at: None,
        };
        self.save(&task)?;
        Ok(task)
    }

    /// 获取任务
    pub fn get(&self, id: &str) -> Option<Task> {
        let path = self.task_dir.join(format!("{id}.json"));
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// 列出所有任务
    pub fn list(&self) -> Vec<Task> {
        let mut tasks = Vec::new();
        if !self.task_dir.exists() {
            return tasks;
        }
        if let Ok(entries) = std::fs::read_dir(&self.task_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "json") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(task) = serde_json::from_str::<Task>(&content) {
                            tasks.push(task);
                        }
                    }
                }
            }
        }
        tasks
    }

    /// 更新任务状态
    pub fn update_state(&self, id: &str, new_state: TaskState) -> Result<Task, String> {
        let mut task = self.get(id).ok_or_else(|| format!("任务不存在: {id}"))?;
        task.state = new_state;
        if task.state == TaskState::Completed || task.state == TaskState::Failed {
            task.completed_at = Some(chrono_now());
        }
        self.save(&task)?;
        Ok(task)
    }

    /// 分配任务给某人
    pub fn assign(&self, id: &str, owner: &str) -> Result<Task, String> {
        let mut task = self.get(id).ok_or_else(|| format!("任务不存在: {id}"))?;
        task.owner = Some(owner.to_string());
        self.save(&task)?;
        Ok(task)
    }

    /// 删除任务
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let path = self.task_dir.join(format!("{id}.json"));
        if path.exists() {
            std::fs::remove_file(path).map_err(|e| format!("删除任务失败: {e}"))?;
        }
        Ok(())
    }

    fn save(&self, task: &Task) -> Result<(), String> {
        self.ensure_dir()?;
        let path = self.task_dir.join(format!("{}.json", task.id));
        let content =
            serde_json::to_string_pretty(task).map_err(|e| format!("序列化任务失败: {e}"))?;
        std::fs::write(&path, content).map_err(|e| format!("写入任务文件失败: {e}"))?;
        Ok(())
    }
}

fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "unknown".into(),
    }
}

fn chrono_now_ts() -> String {
    let output = std::process::Command::new("date").args(["+%s%N"]).output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => format!(
            "{}",
            std::time::SystemTime::now()
                .elapsed()
                .unwrap_or_default()
                .as_nanos()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_task_lifecycle() {
        let tmp = format!("/tmp/yunxi-test-task-{}", std::process::id());
        let _ = fs::remove_dir_all(&tmp);
        let mgr = TaskManager::new(&tmp);

        let task = mgr.create("测试任务", "这是一个测试").unwrap();
        assert_eq!(task.state, TaskState::Pending);

        let updated = mgr.update_state(&task.id, TaskState::InProgress).unwrap();
        assert_eq!(updated.state, TaskState::InProgress);

        let completed = mgr.update_state(&task.id, TaskState::Completed).unwrap();
        assert_eq!(completed.state, TaskState::Completed);
        assert!(completed.completed_at.is_some());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_list_and_delete() {
        let tmp = format!("/tmp/yunxi-test-task-list-{}", std::process::id());
        let _ = fs::remove_dir_all(&tmp);
        let mgr = TaskManager::new(&tmp);

        mgr.create("任务1", "描述1").unwrap();
        mgr.create("任务2", "描述2").unwrap();

        let tasks = mgr.list();
        assert_eq!(tasks.len(), 2);

        mgr.delete(&tasks[0].id).unwrap();
        assert_eq!(mgr.list().len(), 1);

        let _ = fs::remove_dir_all(&tmp);
    }
}
