//! 统一记忆系统桥接：为 system prompt 与 IPC 提供记忆上下文。

pub use memory::{build_context_section, search_report, DEFAULT_CONTEXT_LIMIT};

/// 构建注入 system prompt 的记忆摘要段落。
pub fn build_memory_context_section() -> Result<String, Box<dyn std::error::Error>> {
    build_context_section(DEFAULT_CONTEXT_LIMIT).map_err(|e| e.into())
}

/// 按关键词检索记忆，返回可读文本报告。
pub fn search_memory_report(query: &str, limit: usize) -> Result<String, String> {
    search_report(query, limit)
}
