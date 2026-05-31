//! 元认知监控
//!
//! 跟踪推理进度、检测循环、管理预算。

use crate::pipeline::ReasoningPhase;

/// 推理预算
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReasoningBudget {
    pub max_iterations: usize,
    pub current_iteration: usize,
    pub max_tokens: usize,
    pub tokens_used: usize,
}

/// 元认知监控器
pub struct MetaCognitiveMonitor {
    budget: ReasoningBudget,
    completed_phases: Vec<ReasoningPhase>,
    hypothesis_generation_counts: std::collections::HashMap<String, usize>,
}

impl MetaCognitiveMonitor {
    pub fn new(budget: ReasoningBudget) -> Self {
        Self {
            budget,
            completed_phases: Vec::new(),
            hypothesis_generation_counts: std::collections::HashMap::new(),
        }
    }

    /// 记录完成的阶段
    pub fn record_phase(&mut self, phase: ReasoningPhase) {
        self.completed_phases.push(phase);
    }

    /// 记录假设生成（用于循环检测）
    pub fn record_hypothesis(&mut self, claim: &str) -> bool {
        let count = self
            .hypothesis_generation_counts
            .entry(claim.to_string())
            .or_insert(0);
        *count += 1;
        // 同一假设生成超过 2 次，视为循环
        *count > 2
    }

    /// 检查是否还在预算内
    pub fn within_budget(&self) -> bool {
        self.budget.current_iteration < self.budget.max_iterations
            && self.budget.tokens_used < self.budget.max_tokens
    }

    /// 增加迭代次数
    pub fn increment_iteration(&mut self) {
        self.budget.current_iteration += 1;
    }

    /// 增加 token 消耗
    pub fn add_tokens(&mut self, tokens: usize) {
        self.budget.tokens_used += tokens;
    }

    /// 获取进度摘要
    pub fn progress_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "completed_phases": self.completed_phases.len(),
            "total_phases": 6,
            "iteration": format!("{}/{}", self.budget.current_iteration, self.budget.max_iterations),
            "tokens": format!("{}/{}", self.budget.tokens_used, self.budget.max_tokens),
            "within_budget": self.within_budget(),
            "unique_hypotheses": self.hypothesis_generation_counts.len(),
        })
    }

    /// 获取预算状态
    pub fn budget(&self) -> &ReasoningBudget {
        &self.budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_tracking() {
        let budget = ReasoningBudget {
            max_iterations: 3,
            current_iteration: 0,
            max_tokens: 10000,
            tokens_used: 0,
        };
        let mut monitor = MetaCognitiveMonitor::new(budget);

        assert!(monitor.within_budget());
        monitor.increment_iteration();
        monitor.increment_iteration();
        monitor.add_tokens(5000);
        assert!(monitor.within_budget());

        monitor.increment_iteration();
        assert!(!monitor.within_budget());
    }

    #[test]
    fn test_loop_detection() {
        let budget = ReasoningBudget {
            max_iterations: 10,
            current_iteration: 0,
            max_tokens: 100000,
            tokens_used: 0,
        };
        let mut monitor = MetaCognitiveMonitor::new(budget);

        assert!(!monitor.record_hypothesis("新颖"));
        assert!(!monitor.record_hypothesis("新颖"));
        assert!(monitor.record_hypothesis("新颖")); // 第 3 次 = 循环
        assert!(!monitor.record_hypothesis("创造性"));
    }

    #[test]
    fn test_progress_summary() {
        let budget = ReasoningBudget {
            max_iterations: 5,
            current_iteration: 2,
            max_tokens: 10000,
            tokens_used: 3000,
        };
        let mut monitor = MetaCognitiveMonitor::new(budget);
        monitor.record_phase(ReasoningPhase::Engagement);
        monitor.record_phase(ReasoningPhase::Analysis);

        let summary = monitor.progress_summary();
        assert_eq!(summary["completed_phases"], 2);
        assert_eq!(summary["iteration"], "2/5");
    }
}
