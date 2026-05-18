use std::collections::BTreeMap;

/// 权限模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionMode {
    /// 只读模式
    ReadOnly,
    /// 工作区写入模式
    WorkspaceWrite,
    /// 危险完全访问模式
    DangerFullAccess,
    /// 提示模式
    Prompt,
    /// 允许所有模式
    Allow,
}

impl PermissionMode {
    /// 转换为字符串
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
            Self::Prompt => "prompt",
            Self::Allow => "allow",
        }
    }
}

/// 权限请求
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRequest {
    /// 工具名称
    pub tool_name: String,
    /// 工具输入
    pub input: String,
    /// 当前模式
    pub current_mode: PermissionMode,
    /// 所需模式
    pub required_mode: PermissionMode,
}

/// 权限提示决策
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionPromptDecision {
    /// 允许
    Allow,
    /// 拒绝
    Deny { reason: String },
}

/// 权限提示器
pub trait PermissionPrompter {
    /// 决定是否允许权限
    ///
    /// # 参数
    /// - `request`: 权限请求
    ///
    /// # 返回
    /// 权限提示决策
    fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision;
}

/// 权限结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionOutcome {
    /// 允许
    Allow,
    /// 拒绝
    Deny { reason: String },
}

/// 权限策略
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionPolicy {
    active_mode: PermissionMode,
    tool_requirements: BTreeMap<String, PermissionMode>,
}

impl PermissionPolicy {
    /// 创建新的权限策略
    ///
    /// # 参数
    /// - `active_mode`: 激活的权限模式
    ///
    /// # 返回
    /// 新的权限策略
    #[must_use]
    pub fn new(active_mode: PermissionMode) -> Self {
        Self {
            active_mode,
            tool_requirements: BTreeMap::new(),
        }
    }

    /// 设置工具所需权限
    ///
    /// # 参数
    /// - `tool_name`: 工具名称
    /// - `required_mode`: 所需权限模式
    ///
    /// # 返回
    /// 修改后的权限策略
    #[must_use]
    pub fn with_tool_requirement(
        mut self,
        tool_name: impl Into<String>,
        required_mode: PermissionMode,
    ) -> Self {
        self.tool_requirements
            .insert(tool_name.into(), required_mode);
        self
    }

    /// 获取当前激活的权限模式
    ///
    /// # 返回
    /// 当前权限模式
    #[must_use]
    pub fn active_mode(&self) -> PermissionMode {
        self.active_mode
    }

    /// 获取工具所需权限模式
    ///
    /// # 参数
    /// - `tool_name`: 工具名称
    ///
    /// # 返回
    /// 工具所需权限模式，默认为 `DangerFullAccess`
    #[must_use]
    pub fn required_mode_for(&self, tool_name: &str) -> PermissionMode {
        self.tool_requirements
            .get(tool_name)
            .copied()
            .unwrap_or(PermissionMode::DangerFullAccess)
    }

    /// 授权工具使用
    ///
    /// # 参数
    /// - `tool_name`: 工具名称
    /// - `input`: 工具输入
    /// - `prompter`: 可选的权限提示器
    ///
    /// # 返回
    /// 权限结果
    #[must_use]
    pub fn authorize(
        &self,
        tool_name: &str,
        input: &str,
        mut prompter: Option<&mut dyn PermissionPrompter>,
    ) -> PermissionOutcome {
        let current_mode = self.active_mode();
        let required_mode = self.required_mode_for(tool_name);
        if current_mode == PermissionMode::Allow || current_mode >= required_mode {
            return PermissionOutcome::Allow;
        }

        let request = PermissionRequest {
            tool_name: tool_name.to_string(),
            input: input.to_string(),
            current_mode,
            required_mode,
        };

        if current_mode == PermissionMode::Prompt
            || (current_mode == PermissionMode::WorkspaceWrite
                && required_mode == PermissionMode::DangerFullAccess)
        {
            return match prompter.as_mut() {
                Some(prompter) => match prompter.decide(&request) {
                    PermissionPromptDecision::Allow => PermissionOutcome::Allow,
                    PermissionPromptDecision::Deny { reason } => PermissionOutcome::Deny { reason },
                },
                None => PermissionOutcome::Deny {
                    reason: format!(
                        "tool '{tool_name}' requires approval to escalate from {} to {}",
                        current_mode.as_str(),
                        required_mode.as_str()
                    ),
                },
            };
        }

        PermissionOutcome::Deny {
            reason: format!(
                "tool '{tool_name}' requires {} permission; current mode is {}",
                required_mode.as_str(),
                current_mode.as_str()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PermissionMode, PermissionOutcome, PermissionPolicy, PermissionPromptDecision,
        PermissionPrompter, PermissionRequest,
    };

    struct RecordingPrompter {
        seen: Vec<PermissionRequest>,
        allow: bool,
    }

    impl PermissionPrompter for RecordingPrompter {
        fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision {
            self.seen.push(request.clone());
            if self.allow {
                PermissionPromptDecision::Allow
            } else {
                PermissionPromptDecision::Deny {
                    reason: "not now".to_string(),
                }
            }
        }
    }

    #[test]
    fn allows_tools_when_active_mode_meets_requirement() {
        let policy = PermissionPolicy::new(PermissionMode::WorkspaceWrite)
            .with_tool_requirement("read_file", PermissionMode::ReadOnly)
            .with_tool_requirement("write_file", PermissionMode::WorkspaceWrite);

        assert_eq!(
            policy.authorize("read_file", "{}", None),
            PermissionOutcome::Allow
        );
        assert_eq!(
            policy.authorize("write_file", "{}", None),
            PermissionOutcome::Allow
        );
    }

    #[test]
    fn denies_read_only_escalations_without_prompt() {
        let policy = PermissionPolicy::new(PermissionMode::ReadOnly)
            .with_tool_requirement("write_file", PermissionMode::WorkspaceWrite)
            .with_tool_requirement("bash", PermissionMode::DangerFullAccess);

        assert!(matches!(
            policy.authorize("write_file", "{}", None),
            PermissionOutcome::Deny { reason } if reason.contains("requires workspace-write permission")
        ));
        assert!(matches!(
            policy.authorize("bash", "{}", None),
            PermissionOutcome::Deny { reason } if reason.contains("requires danger-full-access permission")
        ));
    }

    #[test]
    fn prompts_for_workspace_write_to_danger_full_access_escalation() {
        let policy = PermissionPolicy::new(PermissionMode::WorkspaceWrite)
            .with_tool_requirement("bash", PermissionMode::DangerFullAccess);
        let mut prompter = RecordingPrompter {
            seen: Vec::new(),
            allow: true,
        };

        let outcome = policy.authorize("bash", "echo hi", Some(&mut prompter));

        assert_eq!(outcome, PermissionOutcome::Allow);
        assert_eq!(prompter.seen.len(), 1);
        assert_eq!(prompter.seen[0].tool_name, "bash");
        assert_eq!(
            prompter.seen[0].current_mode,
            PermissionMode::WorkspaceWrite
        );
        assert_eq!(
            prompter.seen[0].required_mode,
            PermissionMode::DangerFullAccess
        );
    }

    #[test]
    fn honors_prompt_rejection_reason() {
        let policy = PermissionPolicy::new(PermissionMode::WorkspaceWrite)
            .with_tool_requirement("bash", PermissionMode::DangerFullAccess);
        let mut prompter = RecordingPrompter {
            seen: Vec::new(),
            allow: false,
        };

        assert!(matches!(
            policy.authorize("bash", "echo hi", Some(&mut prompter)),
            PermissionOutcome::Deny { reason } if reason == "not now"
        ));
    }
}
