//! 执行策略与审批模型。
//!
//! 提供 YAML 策略热加载、命令审批、权限控制。

use std::path::Path;

use serde::{Deserialize, Serialize};

/// 执行策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecPolicy {
    pub version: String,
    pub rules: Vec<ExecRule>,
    pub defaults: PolicyDefaults,
}

/// 执行规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecRule {
    pub name: String,
    pub pattern: String,
    pub action: RuleAction,
    pub severity: Severity,
}

/// 规则动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    Allow,
    Deny,
    Prompt,
}

/// 严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// 策略默认值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefaults {
    pub default_action: RuleAction,
    pub require_approval_for: Vec<String>,
}

impl Default for PolicyDefaults {
    fn default() -> Self {
        Self {
            default_action: RuleAction::Allow,
            require_approval_for: vec!["rm".into(), "sudo".into(), "chmod".into()],
        }
    }
}

impl Default for ExecPolicy {
    fn default() -> Self {
        Self {
            version: "1.0".into(),
            rules: vec![
                ExecRule {
                    name: "禁止删除根目录".into(),
                    pattern: "rm.*-rf.*/".into(),
                    action: RuleAction::Deny,
                    severity: Severity::Critical,
                },
                ExecRule {
                    name: "危险命令需确认".into(),
                    pattern: "sudo.*".into(),
                    action: RuleAction::Prompt,
                    severity: Severity::High,
                },
            ],
            defaults: PolicyDefaults::default(),
        }
    }
}

/// 执行策略管理器
#[derive(Debug)]
pub struct PolicyManager {
    policy: ExecPolicy,
}

impl PolicyManager {
    /// 加载策略文件
    pub fn load(path: &Path) -> Result<Self, String> {
        if path.exists() {
            let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            let policy: ExecPolicy =
                serde_yaml::from_str(&content).map_err(|e: serde_yaml::Error| e.to_string())?;
            Ok(Self { policy })
        } else {
            Ok(Self {
                policy: ExecPolicy::default(),
            })
        }
    }

    /// 检查命令是否符合策略
    pub fn check_command(&self, command: &str) -> PolicyCheckResult {
        for rule in &self.policy.rules {
            if Self::matches_pattern(command, &rule.pattern) {
                return PolicyCheckResult {
                    allowed: rule.action == RuleAction::Allow,
                    requires_approval: rule.action == RuleAction::Prompt,
                    rule_name: Some(rule.name.clone()),
                    severity: Some(rule.severity),
                };
            }
        }

        // 检查默认值
        let requires_approval = self
            .policy
            .defaults
            .require_approval_for
            .iter()
            .any(|cmd| command.starts_with(cmd));

        PolicyCheckResult {
            allowed: true,
            requires_approval,
            rule_name: None,
            severity: None,
        }
    }

    fn matches_pattern(command: &str, pattern: &str) -> bool {
        // 简单模式匹配：支持 * 通配符
        let regex_pattern = pattern.replace('*', ".*");
        regex::Regex::new(&regex_pattern)
            .map(|re| re.is_match(command))
            .unwrap_or(false)
    }
}

/// 策略检查结果
#[derive(Debug, Clone)]
pub struct PolicyCheckResult {
    pub allowed: bool,
    pub requires_approval: bool,
    pub rule_name: Option<String>,
    pub severity: Option<Severity>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_blocks_dangerous() {
        let manager = PolicyManager {
            policy: ExecPolicy::default(),
        };

        let result = manager.check_command("rm -rf /");
        assert!(!result.allowed);
        assert_eq!(result.severity, Some(Severity::Critical));

        let result = manager.check_command("sudo apt-get update");
        assert!(result.requires_approval);
    }

    #[test]
    fn allows_safe_commands() {
        let manager = PolicyManager {
            policy: ExecPolicy::default(),
        };

        let result = manager.check_command("echo hello");
        assert!(result.allowed);
        assert!(!result.requires_approval);
    }
}
