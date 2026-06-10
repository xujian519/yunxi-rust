//! Tool dispatch: maps tool names to their runner functions.
//!
//! Dispatch is backed by the global `ToolRegistry` populated in
//! `tool_registry.rs`. The public API `execute_tool()` delegates to
//! `GLOBAL_REGISTRY.get(name)` — no more 40+-arm match.
//!
//! `execute_tool_output()` returns a `ToolOutput` and is the preferred API
//! for new code; `execute_tool()` is retained for backward compatibility
//! during the gradual migration (P1-4).

use serde_json::Value;

use crate::tool_registry::GLOBAL_REGISTRY;
use crate::tool_trait::ToolOutput;

/// Dispatches tool execution by name via the global tool registry.
///
/// Returns a typed `ToolOutput` result, preferred for new integrations.
pub fn execute_tool_output(name: &str, input: &Value) -> Result<ToolOutput, String> {
    match GLOBAL_REGISTRY.get(name) {
        Some(runner) => {
            let start = std::time::Instant::now();
            let result = runner(input);
            let elapsed = start.elapsed();
            match result {
                Ok(content) => {
                    Ok(ToolOutput::ok(content).with_duration(elapsed.as_millis() as u64))
                }
                Err(content) => {
                    Ok(ToolOutput::err(content).with_duration(elapsed.as_millis() as u64))
                }
            }
        }
        None => Ok(ToolOutput::err(format!("unsupported tool: {name}"))),
    }
}

/// Dispatches tool execution by name via the global tool registry.
///
/// Returns an error string if the tool name is unknown or execution fails.
/// Prefer `execute_tool_output()` for new code.
pub fn execute_tool(name: &str, input: &Value) -> Result<String, String> {
    match GLOBAL_REGISTRY.get(name) {
        Some(runner) => runner(input),
        None => Err(format!("unsupported tool: {name}")),
    }
}
