//! HTTP Server 会话磁盘存储（`{workspace}/.yunxi/server-sessions/`）。

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use runtime::Session;
use serde::Serialize;

use crate::agent_bridge;

#[derive(Debug, Clone, Serialize)]
pub struct SessionMeta {
    pub id: String,
    pub message_count: usize,
    pub modified_at: u64,
}

pub struct SessionStore {
    dir: PathBuf,
}

impl SessionStore {
    pub fn new() -> Self {
        let dir = agent_bridge::workspace_root()
            .join(".yunxi")
            .join("server-sessions");
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    #[cfg(test)]
    pub fn with_dir(dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    fn session_path(&self, id: &str) -> Result<PathBuf, String> {
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("invalid session id".into());
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    pub fn list(&self) -> Result<Vec<SessionMeta>, String> {
        let mut metas = Vec::new();
        let entries = fs::read_dir(&self.dir).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let modified_at = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let message_count = Session::load_from_path(&path)
                .map(|s| s.messages.len())
                .unwrap_or(0);
            metas.push(SessionMeta {
                id: stem.to_string(),
                message_count,
                modified_at,
            });
        }
        metas.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
        Ok(metas)
    }

    pub fn load(&self, id: &str) -> Result<Session, String> {
        let path = self.session_path(id)?;
        Session::load_from_path(path).map_err(|e| e.to_string())
    }

    pub fn save(&self, id: &str, session: &Session) -> Result<(), String> {
        let path = self.session_path(id)?;
        session.save_to_path(path).map_err(|e| e.to_string())
    }

    pub fn create(&self, title: &str) -> Result<String, String> {
        let id = new_session_id();
        let mut session = Session::new();
        if !title.is_empty() {
            session
                .messages
                .push(runtime::ConversationMessage::user_text(format!(
                    "[session title: {title}]"
                )));
        }
        self.save(&id, &session)?;
        Ok(id)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let path = self.session_path(id)?;
        if path.exists() {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn exists(&self, id: &str) -> bool {
        self.session_path(id)
            .ok()
            .is_some_and(|p| p.exists())
    }
}

fn new_session_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("session-{ms}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_load_roundtrip() {
        let dir = std::env::temp_dir().join(format!("yunxi-sess-test-{}", std::process::id()));
        let store = SessionStore::with_dir(dir.clone());
        let id = store.create("测试").unwrap();
        let session = store.load(&id).unwrap();
        assert_eq!(session.messages.len(), 1);
        let metas = store.list().unwrap();
        assert!(metas.iter().any(|m| m.id == id));
        store.delete(&id).unwrap();
        let _ = fs::remove_dir_all(dir);
    }
}
