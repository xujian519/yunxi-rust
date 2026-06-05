//! Tool dispatch: maps tool names to their runner functions.
//!
//! Dispatch is backed by the global [`ToolRegistry`] populated in
//! `tool_registry.rs`. The public API `execute_tool()` delegates to
//! `GLOBAL_REGISTRY.get(name)` — no more 40+-arm match.

use serde_json::Value;

use crate::tool_registry::GLOBAL_REGISTRY;

/// Dispatches tool execution by name via the global tool registry.
///
/// Returns an error string if the tool name is unknown or execution fails.
pub fn execute_tool(name: &str, input: &Value) -> Result<String, String> {
    match GLOBAL_REGISTRY.get(name) {
        Some(runner) => runner(input),
        None => Err(format!("unsupported tool: {name}")),
    }
}
