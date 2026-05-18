use std::path::PathBuf;

/// A single crate entry in the workspace port manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortCrate {
    pub name: String,
    pub description: String,
    pub rust_files: usize,
    pub lines_of_code: Option<usize>,
}

/// A manifest of the Rust workspace porting progress.
#[derive(Debug, Clone)]
pub struct PortManifest {
    pub workspace_root: PathBuf,
    pub crates: Vec<PortCrate>,
    pub total_rust_files: usize,
    pub total_crates: usize,
}

impl PortManifest {
    /// Build a port manifest by scanning the workspace directory.
    pub fn build(workspace_root: PathBuf) -> std::io::Result<Self> {
        let crates_dir = workspace_root.join("crates");
        let mut crates = Vec::new();

        if crates_dir.is_dir() {
            for entry in std::fs::read_dir(&crates_dir)? {
                let entry = entry?;
                let crate_path = entry.path();
                if !crate_path.is_dir() {
                    continue;
                }

                let crate_name = crate_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let crate_toml_path = crate_path.join("Cargo.toml");
                let description = if crate_toml_path.is_file() {
                    Self::extract_description(&crate_toml_path).unwrap_or_else(|| String::from("-"))
                } else {
                    String::from("-")
                };

                let rust_files = Self::count_rust_files(&crate_path);
                let lines_of_code = Self::count_lines(&crate_path);

                crates.push(PortCrate {
                    name: crate_name,
                    description,
                    rust_files,
                    lines_of_code,
                });
            }
        }

        crates.sort_by(|a, b| a.name.cmp(&b.name));

        let total_rust_files: usize = crates.iter().map(|c| c.rust_files).sum();
        let total_crates = crates.len();

        Ok(Self {
            workspace_root,
            crates,
            total_rust_files,
            total_crates,
        })
    }

    fn extract_description(toml_path: &std::path::Path) -> Option<String> {
        let content = std::fs::read_to_string(toml_path).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("description = \"") {
                if let Some(desc) = rest.strip_suffix('\"') {
                    return Some(desc.to_string());
                }
            }
        }
        None
    }

    fn count_rust_files(dir: &std::path::Path) -> usize {
        let src = dir.join("src");
        if !src.is_dir() {
            return 0;
        }
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(&src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rs") && path.is_file() {
                    count += 1;
                }
            }
        }
        count
    }

    fn count_lines(dir: &std::path::Path) -> Option<usize> {
        let src = dir.join("src");
        if !src.is_dir() {
            return None;
        }
        let mut total = 0;
        if let Ok(entries) = std::fs::read_dir(&src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rs") && path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        total += content.lines().count();
                    }
                }
            }
        }
        Some(total)
    }

    /// Render the manifest as Markdown.
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut lines = vec![
            format!("# YunXi Workspace Port Manifest"),
            String::new(),
            format!("Workspace root: `{}`", self.workspace_root.display()),
            format!("Total crates: **{}**", self.total_crates),
            format!("Total Rust source files: **{}**", self.total_rust_files),
            String::new(),
            String::from("## Crate Inventory"),
            String::new(),
            String::from("| Crate | Description | .rs Files | Lines |"),
            String::from("|-------|-------------|-----------|-------|"),
        ];

        for c in &self.crates {
            let loc = c
                .lines_of_code
                .map_or_else(|| String::from("-"), |n| n.to_string());
            lines.push(format!(
                "| `{}` | {} | {} | {} |",
                c.name, c.description, c.rust_files, loc
            ));
        }

        lines.push(String::new());
        lines.join("\n")
    }
}
