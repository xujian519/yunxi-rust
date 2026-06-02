use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub timestamp: u64,
    pub commands: Vec<CommandRecord>,
    pub files: Vec<FileRecord>,
    pub context: SessionContext,
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub input: String,
    pub output: String,
    pub timestamp: u64,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub path: String,
    pub action: FileAction,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileAction {
    Open,
    Edit,
    Close,
    Create,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub workspace: Option<String>,
    pub active_files: Vec<String>,
    pub theme: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metadata: Vec<(String, String)>,
}

impl Default for SessionContext {
    fn default() -> Self {
        Self {
            workspace: None,
            active_files: Vec::new(),
            theme: "default".to_string(),
            metadata: Vec::new(),
        }
    }
}

impl Session {
    pub fn new(id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            timestamp,
            commands: Vec::new(),
            files: Vec::new(),
            context: SessionContext::default(),
            duration: None,
        }
    }

    pub fn start(&mut self) {
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn end(&mut self) {
        if let Some(start) = self.duration {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.duration = Some(now - start);
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.duration = Some(now - self.timestamp);
        }
    }

    pub fn add_command(&mut self, input: String, output: String, duration_ms: Option<u64>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.commands.push(CommandRecord {
            input,
            output,
            timestamp,
            duration_ms,
        });
    }

    pub fn add_file_record(&mut self, path: String, action: FileAction) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.files.push(FileRecord {
            path,
            action,
            timestamp,
        });
    }

    pub fn duration_seconds(&self) -> Option<u64> {
        self.duration
    }

    pub fn is_active(&self) -> bool {
        self.duration.is_none()
    }
}
