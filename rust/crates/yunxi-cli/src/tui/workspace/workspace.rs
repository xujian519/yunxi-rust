use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub path: PathBuf,
    pub open_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub layout: WorkspaceLayout,
    pub settings: WorkspaceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceLayout {
    SplitHorizontal,
    SplitVertical,
    Grid,
    Custom(String),
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self::SplitHorizontal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSettings {
    pub theme: String,
    pub font_size: u8,
    pub line_numbers: bool,
    pub word_wrap: bool,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, String>,
}

impl Default for WorkspaceSettings {
    fn default() -> Self {
        Self {
            theme: "default".to_string(),
            font_size: 12,
            line_numbers: true,
            word_wrap: false,
            custom: HashMap::new(),
        }
    }
}

impl Workspace {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            open_files: Vec::new(),
            active_file: None,
            layout: WorkspaceLayout::default(),
            settings: WorkspaceSettings::default(),
        }
    }

    pub fn workspace_file(&self) -> PathBuf {
        self.path.join(".yunxi-workspace.json")
    }
}
