//! Hebbian 路径提示 trait（解耦 router 与 memory crate）。
//!
//! 由 memory crate 的 `HebbianOptimizer` 实现，供 `WorkflowRouter` 使用。

/// Hebbian 学习路径提示（由 memory crate 的 HebbianOptimizer 实现）。
pub trait HebbianPathHint: Send + Sync {
    /// 获取从 source 工具出发的最优路径建议。
    ///
    /// 返回 `(tool_id, strength)` 列表，按 strength 降序排列。
    fn optimal_path(&self, source: &str) -> Option<Vec<(String, f64)>>;
}
