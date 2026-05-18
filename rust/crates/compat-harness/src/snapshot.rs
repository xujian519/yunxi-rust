use std::path::PathBuf;

use serde::Deserialize;

// --- Snapshot JSON types ---

#[derive(Debug, Clone, Deserialize)]
pub struct SnapshotEntry {
    pub name: String,
    pub source_hint: String,
    pub responsibility: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveSurface {
    pub archive_root: String,
    pub root_files: Vec<String>,
    pub root_dirs: Vec<String>,
    pub total_ts_like_files: u32,
    pub command_entry_count: u32,
    pub tool_entry_count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubsystemSnapshot {
    pub archive_name: String,
    pub package_name: String,
    pub module_count: u32,
    pub sample_files: Vec<String>,
}

/// Loader for the JSON snapshot files under `reference_data/`.
pub struct SnapshotLoader {
    data_dir: PathBuf,
}

impl SnapshotLoader {
    #[must_use]
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    /// Discover the `reference_data` directory relative to the crate root.
    #[must_use]
    pub fn from_crate_dir() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        // Crate is at rust/crates/compat-harness; reference_data is at rust/reference_data/
        let data_dir = manifest_dir.join("../../reference_data");
        Self { data_dir }
    }

    fn read_json<T: for<'de> Deserialize<'de>>(&self, relative_path: &str) -> std::io::Result<T> {
        let path = self.data_dir.join(relative_path);
        let text = std::fs::read_to_string(&path)?;
        serde_json::from_str(&text)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    // --- Command snapshot ---

    pub fn load_commands(&self) -> std::io::Result<Vec<SnapshotEntry>> {
        self.read_json("commands_snapshot.json")
    }

    // --- Tool snapshot ---

    pub fn load_tools(&self) -> std::io::Result<Vec<SnapshotEntry>> {
        self.read_json("tools_snapshot.json")
    }

    // --- Archive surface ---

    pub fn load_archive_surface(&self) -> std::io::Result<ArchiveSurface> {
        self.read_json("archive_surface_snapshot.json")
    }

    // --- Subsystem snapshots ---

    pub fn load_subsystems(&self) -> std::io::Result<Vec<SubsystemSnapshot>> {
        let subsystems_dir = self.data_dir.join("subsystems");
        let mut entries: Vec<SubsystemSnapshot> = Vec::new();

        if !subsystems_dir.is_dir() {
            return Ok(entries);
        }

        for entry in std::fs::read_dir(&subsystems_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                let text = std::fs::read_to_string(&path)?;
                let subsystem: SubsystemSnapshot = serde_json::from_str(&text)
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
                entries.push(subsystem);
            }
        }
        entries.sort_by(|a, b| a.archive_name.cmp(&b.archive_name));
        Ok(entries)
    }

    /// True when the snapshot data directory exists and is populated.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.data_dir
            .join("archive_surface_snapshot.json")
            .is_file()
    }
}

impl Default for SnapshotLoader {
    fn default() -> Self {
        Self::from_crate_dir()
    }
}
