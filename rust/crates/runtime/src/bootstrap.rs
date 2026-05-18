/// 启动阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapPhase {
    /// CLI 入口
    CliEntry,
    /// 快速路径版本检查
    FastPathVersion,
    /// 启动分析器
    StartupProfiler,
    /// 系统提示词快速路径
    SystemPromptFastPath,
    /// Chrome MCP 快速路径
    ChromeMcpFastPath,
    /// 守护进程工作器快速路径
    DaemonWorkerFastPath,
    /// 桥接快速路径
    BridgeFastPath,
    /// 守护进程快速路径
    DaemonFastPath,
    /// 后台会话快速路径
    BackgroundSessionFastPath,
    /// 模板快速路径
    TemplateFastPath,
    /// 环境运行器快速路径
    EnvironmentRunnerFastPath,
    /// 主运行时
    MainRuntime,
}

/// 启动计划
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapPlan {
    phases: Vec<BootstrapPhase>,
}

impl BootstrapPlan {
    /// Claude Code 默认启动计划
    #[must_use]
    pub fn claude_code_default() -> Self {
        Self::yunxi_default()
    }

    /// 云熙智能体默认启动计划（推荐使用此方法）
    #[must_use]
    pub fn yunxi_default() -> Self {
        Self::from_phases(vec![
            BootstrapPhase::CliEntry,
            BootstrapPhase::FastPathVersion,
            BootstrapPhase::StartupProfiler,
            BootstrapPhase::SystemPromptFastPath,
            BootstrapPhase::ChromeMcpFastPath,
            BootstrapPhase::DaemonWorkerFastPath,
            BootstrapPhase::BridgeFastPath,
            BootstrapPhase::DaemonFastPath,
            BootstrapPhase::BackgroundSessionFastPath,
            BootstrapPhase::TemplateFastPath,
            BootstrapPhase::EnvironmentRunnerFastPath,
            BootstrapPhase::MainRuntime,
        ])
    }

    /// 从阶段列表创建启动计划
    ///
    /// # 参数
    /// - `phases`: 启动阶段列表
    ///
    /// # 返回
    /// 启动计划
    #[must_use]
    pub fn from_phases(phases: Vec<BootstrapPhase>) -> Self {
        let mut deduped = Vec::new();
        for phase in phases {
            if !deduped.contains(&phase) {
                deduped.push(phase);
            }
        }
        Self { phases: deduped }
    }

    /// 获取启动阶段列表
    ///
    /// # 返回
    /// 启动阶段列表的引用
    #[must_use]
    pub fn phases(&self) -> &[BootstrapPhase] {
        &self.phases
    }
}
