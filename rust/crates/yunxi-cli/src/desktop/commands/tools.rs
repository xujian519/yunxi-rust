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
pub fn patent_compare(
    target_title: String,
    target_claims: Vec<String>,
    prior_title: String,
    prior_claims: Vec<String>,
) -> Result<String, String> {
    execute_tool(
        "PatentCompare",
        &json!({
            "mode": "diff",
            "target": {
                "title": target_title,
                "claims": target_claims,
            },
            "priorArt": {
                "title": prior_title,
                "claims": prior_claims,
            },
        }),
    )
}

#[tauri::command]
pub fn knowledge_search(query: String) -> Result<String, String> {
    execute_tool(
        "KnowledgeSearch",
        &json!({
            "query": query,
            "limit": 8,
        }),
    )
}

#[tauri::command]
pub fn memory_search(query: String, limit: Option<usize>) -> Result<String, String> {
    yunxi_cli::memory_bridge::search_memory_report(&query, limit.unwrap_or(10))
}

#[tauri::command]
pub fn oa_parse(content: String, application_number: Option<String>) -> Result<String, String> {
    execute_tool(
        "OaParse",
        &json!({
            "content": content,
            "application_number": application_number,
            "document_type": "cn",
        }),
    )
}
