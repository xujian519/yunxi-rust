//! Core types for tool specification and discovery.

pub use runtime::PermissionMode;
use serde_json::Value;

/// A named tool entry with its availability source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolManifestEntry {
    pub name: String,
    pub source: ToolSource,
}

/// Whether a tool is always available or conditionally enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSource {
    /// Always available in the tool set.
    Base,
    /// Only available under specific runtime conditions.
    Conditional,
}

/// A registry of [`ToolManifestEntry`] values for tool discovery and enumeration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolRegistry {
    entries: Vec<ToolManifestEntry>,
}

impl ToolRegistry {
    #[must_use]
    pub fn new(entries: Vec<ToolManifestEntry>) -> Self {
        Self { entries }
    }

    #[must_use]
    pub fn entries(&self) -> &[ToolManifestEntry] {
        &self.entries
    }
}

/// Describes a tool's name, description, JSON input schema, and required permission level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub required_permission: PermissionMode,
}
