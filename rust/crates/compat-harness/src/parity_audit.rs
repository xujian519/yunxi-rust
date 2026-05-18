use crate::snapshot::SnapshotLoader;

/// Result of comparing the Rust workspace against the archived TypeScript snapshot.
#[derive(Debug, Clone)]
pub struct ParityAudit {
    pub snapshot_available: bool,
    pub ts_commands: usize,
    pub ts_tools: usize,
    pub ts_subsystems: usize,
    pub ts_total_files: u32,
    /// Number of Rust crates in the workspace.
    pub rust_crates: usize,
    /// Currently mapped tools in the Rust MVP tool registry.
    pub rust_tools: usize,
    /// Currently mapped slash commands in Rust.
    pub rust_commands: usize,
    /// Subsystem-equivalent directories present in the snapshot.
    pub rust_subsystem_coverage: usize,
}

impl ParityAudit {
    /// Run a parity audit comparing the Rust workspace against the TS snapshot.
    pub fn run(workspace_root: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let loader = SnapshotLoader::default();
        let snapshot_available = loader.is_available();

        let ts_commands = loader.load_commands().map(|cmds| cmds.len()).unwrap_or(0);
        let ts_tools = loader.load_tools().map(|ts| ts.len()).unwrap_or(0);
        let ts_subsystems = loader.load_subsystems().map(|subs| subs.len()).unwrap_or(0);
        let ts_total_files = loader
            .load_archive_surface()
            .map(|s| s.total_ts_like_files)
            .unwrap_or(0);

        // Count Rust workspace crates
        let crates_dir = workspace_root.as_ref().join("crates");
        let mut rust_crates = 0;
        if crates_dir.is_dir() {
            for entry in std::fs::read_dir(&crates_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    rust_crates += 1;
                }
            }
        }

        // Count current Rust built-in tools
        let rust_tools = tools::mvp_tool_specs().len();

        // Count current Rust slash commands
        let rust_commands = commands::slash_command_specs().len();

        // Check how many TS subsystem names have a corresponding Rust crate
        let subsystems = loader.load_subsystems().unwrap_or_default();
        let rust_subsystem_coverage = subsystems
            .iter()
            .filter(|sub| {
                let crate_name = sub.archive_name.replace('-', "_");
                crates_dir.join(&crate_name).is_dir() || crates_dir.join(&sub.archive_name).is_dir()
            })
            .count();

        Ok(Self {
            snapshot_available,
            ts_commands,
            ts_tools,
            ts_subsystems,
            ts_total_files,
            rust_crates,
            rust_tools,
            rust_commands,
            rust_subsystem_coverage,
        })
    }

    /// Render the audit as Markdown.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            String::from("# Parity Audit — Rust vs TypeScript Snapshot"),
            String::new(),
        ];

        if !self.snapshot_available {
            lines.push(String::from(
                "> ⚠️ Snapshot data not available. Run from the repository root.",
            ));
            return lines.join("\n");
        }

        // Summary table
        lines.extend([
            String::from("## Summary"),
            String::new(),
            String::from("| Metric | TS Snapshot | Rust | Coverage |"),
            String::from("|--------|------------|------|----------|"),
            format!(
                "| Commands | {} | {} | {}% |",
                self.ts_commands,
                self.rust_commands,
                Self::pct(self.rust_commands, self.ts_commands)
            ),
            format!(
                "| Tools | {} | {} | {}% |",
                self.ts_tools,
                self.rust_tools,
                Self::pct(self.rust_tools, self.ts_tools)
            ),
            format!(
                "| Subsystems | {} | {} | {}% |",
                self.ts_subsystems,
                self.rust_subsystem_coverage,
                Self::pct(self.rust_subsystem_coverage, self.ts_subsystems)
            ),
            format!(
                "| TS-like files | {} | {} (crates) | — |",
                self.ts_total_files, self.rust_crates
            ),
            String::new(),
        ]);

        // Notes
        lines.extend([
            String::from("## Notes"),
            String::new(),
            format!(
                "- TS snapshot contains **{}** command entries, **{}** tool entries, and **{}** subsystems.",
                self.ts_commands, self.ts_tools, self.ts_subsystems
            ),
            format!(
                "- Rust workspace has **{}** crates with **{}** built-in tools and **{}** slash commands.",
                self.rust_crates, self.rust_tools, self.rust_commands
            ),
            String::from(
                "- 'Subsystem coverage' counts TS subsystem names that have a corresponding Rust crate.",
            ),
            String::from(
                "- The TS snapshot represents the original Claude Code source; the Rust workspace is the YunXi port.",
            ),
            String::new(),
        ]);

        lines.join("\n")
    }

    #[allow(clippy::cast_precision_loss)]
    fn pct(numerator: usize, denominator: usize) -> String {
        if denominator == 0 {
            return String::from("—");
        }
        let pct = (numerator as f64 / denominator as f64) * 100.0;
        format!("{pct:.0}")
    }
}
