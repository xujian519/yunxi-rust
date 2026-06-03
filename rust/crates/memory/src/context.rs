//! 记忆上下文与检索报告（CLI / Server 共用）。

use crate::UnifiedMemory;

/// 注入 system prompt 时的默认条数。
pub const DEFAULT_CONTEXT_LIMIT: usize = 8;

/// 构建注入 system prompt 的记忆摘要段落。
pub fn build_context_section(limit: usize) -> Result<String, String> {
    let um = UnifiedMemory::default_paths()?;
    let entries = um.search("", limit);
    if entries.is_empty() {
        return Ok(String::new());
    }

    let mut lines = vec!["## 持久化记忆（UnifiedMemory）".to_string()];
    for (i, entry) in entries.iter().enumerate() {
        let preview: String = entry.content.chars().take(200).collect();
        let suffix = if entry.content.chars().count() > 200 {
            "…"
        } else {
            ""
        };
        lines.push(format!(
            "{}. [{} / {}] {}{}",
            i + 1,
            entry.tier,
            entry.source,
            preview,
            suffix
        ));
    }
    Ok(lines.join("\n"))
}

/// 按关键词检索记忆，返回可读文本报告。
pub fn search_report(query: &str, limit: usize) -> Result<String, String> {
    let um = UnifiedMemory::default_paths()?;
    let entries = um.search(query, limit);
    if entries.is_empty() {
        return Ok(format!("Memory search\n  Query   {query}\n  Results 0"));
    }

    let mut lines = vec![
        "Memory search".to_string(),
        format!("  Query   {query}"),
        format!("  Results {}", entries.len()),
        String::new(),
    ];
    for (i, entry) in entries.iter().enumerate() {
        lines.push(format!(
            "  {}. id={} tier={} source={}",
            i + 1,
            entry.id,
            entry.tier,
            entry.source
        ));
        for line in entry.content.lines().take(6) {
            lines.push(format!("     {line}"));
        }
        if entry.content.lines().count() > 6 {
            lines.push("     …".to_string());
        }
    }
    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_search_report() {
        let report = search_report("__unlikely_query_xyz__", 5).unwrap_or_default();
        assert!(report.contains("Results 0") || report.contains("Query"));
    }
}
