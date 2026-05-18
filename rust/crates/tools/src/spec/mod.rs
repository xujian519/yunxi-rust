//! Tool specification types and the full MVP tool manifest.

mod core_specs;
mod patent_specs;
mod session_specs;
mod types;

pub use types::{ToolManifestEntry, ToolRegistry, ToolSource, ToolSpec};

/// Returns the full set of MVP tool specifications supported by `YunXi`.
#[must_use]
pub fn mvp_tool_specs() -> Vec<ToolSpec> {
    let mut specs = Vec::new();
    specs.extend(core_specs::core_tool_specs());
    specs.extend(session_specs::session_tool_specs());
    specs.extend(patent_specs::patent_tool_specs());
    specs
}
