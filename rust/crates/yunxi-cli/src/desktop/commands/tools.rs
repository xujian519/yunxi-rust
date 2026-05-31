//! 专利工具 IPC（供前端 slash 命令等直接调用）。

use serde_json::json;
use tools::execute_tool;

#[tauri::command]
pub fn patent_search(query: String, limit: Option<usize>) -> Result<String, String> {
    execute_tool(
        "PatentSearch",
        &json!({
            "query": query,
            "limit": limit.unwrap_or(10),
        }),
    )
}

#[tauri::command]
pub fn knowledge_search(query: String) -> Result<String, String> {
    execute_tool(
        "KnowledgeGraphQuery",
        &json!({
            "query": query,
            "source": "all",
            "limit": 8,
        }),
    )
}
