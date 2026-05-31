//! 定时调度：持久化 `ScheduleConfig` 并判断下次触发时间。
//!
//! 支持：
//! - `@hourly` / `@daily` 别名
//! - `every:N`（N 为秒）间隔

use crate::types::ScheduleConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 带运行状态的调度项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    #[serde(flatten)]
    pub config: ScheduleConfig,
    #[serde(default)]
    pub last_run_unix: Option<u64>,
}

/// 调度注册表（`~/.yunxi/schedules.json`）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleRegistry {
    pub jobs: Vec<ScheduledJob>,
}

impl ScheduleRegistry {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建调度目录失败: {e}"))?;
        }
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化调度配置失败: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("写入调度配置失败: {e}"))
    }

    pub fn register(&mut self, config: ScheduleConfig) {
        if let Some(existing) = self.jobs.iter_mut().find(|j| j.config.cron == config.cron) {
            existing.config = config;
        } else {
            self.jobs.push(ScheduledJob {
                config,
                last_run_unix: None,
            });
        }
    }

    /// 返回当前应执行的作业（并更新 `last_run_unix`）
    pub fn due_jobs(&mut self, now_unix: u64) -> Vec<ScheduleConfig> {
        let mut due = Vec::new();
        for job in &mut self.jobs {
            let interval = parse_schedule_interval_secs(&job.config.cron);
            let should_run = match (job.last_run_unix, interval) {
                (_, None) => false,
                (None, Some(_)) => true,
                (Some(last), Some(secs)) => now_unix.saturating_sub(last) >= secs,
            };
            if should_run {
                job.last_run_unix = Some(now_unix);
                due.push(job.config.clone());
            }
        }
        due
    }
}

impl ScheduledJob {
    pub fn default_store_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(format!("{home}/.yunxi/schedules.json"))
    }
}

/// 解析调度表达式为间隔秒数
#[must_use]
pub fn parse_schedule_interval_secs(cron: &str) -> Option<u64> {
    let trimmed = cron.trim();
    if trimmed == "@hourly" {
        return Some(3600);
    }
    if trimmed == "@daily" {
        return Some(86_400);
    }
    if let Some(rest) = trimmed.strip_prefix("every:") {
        return rest.parse().ok();
    }
    if let Some(rest) = trimmed.strip_prefix("every=") {
        return rest.parse().ok();
    }
    None
}

#[must_use]
pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_interval_aliases() {
        assert_eq!(parse_schedule_interval_secs("@hourly"), Some(3600));
        assert_eq!(parse_schedule_interval_secs("every:120"), Some(120));
    }

    #[test]
    fn due_jobs_respects_interval() {
        let mut reg = ScheduleRegistry::default();
        reg.register(ScheduleConfig {
            cron: "every:60".into(),
            prompt: "tick".into(),
            recurring: true,
        });
        let due = reg.due_jobs(1_000);
        assert_eq!(due.len(), 1);
        let due2 = reg.due_jobs(1_030);
        assert!(due2.is_empty());
        let due3 = reg.due_jobs(1_061);
        assert_eq!(due3.len(), 1);
    }
}
