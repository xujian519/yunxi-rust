use super::Session;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    active_session: Option<String>,
    sessions_dir: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        let sessions_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".yunxi")
            .join("sessions");

        fs::create_dir_all(&sessions_dir).ok();

        Self {
            sessions: HashMap::new(),
            active_session: None,
            sessions_dir,
        }
    }

    pub fn with_sessions_dir(mut self, dir: PathBuf) -> Self {
        self.sessions_dir = dir;
        fs::create_dir_all(&self.sessions_dir).ok();
        self
    }

    pub fn start(&mut self, id: Option<String>) -> &Session {
        let session_id = id.unwrap_or_else(|| {
            format!(
                "session-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            )
        });

        let mut session = Session::new(session_id.clone());
        session.start();

        self.sessions.insert(session_id.clone(), session);
        self.active_session = Some(session_id);

        self.sessions
            .get(self.active_session.as_deref().unwrap())
            .unwrap()
    }

    pub fn end(&mut self, id: &str) -> io::Result<()> {
        if let Some(session) = self.sessions.get_mut(id) {
            session.end();
            self.save(id)?;
        }
        if self.active_session.as_deref() == Some(id) {
            self.active_session = None;
        }
        Ok(())
    }

    pub fn end_active(&mut self) -> io::Result<()> {
        if let Some(id) = self.active_session.clone() {
            self.end(&id)
        } else {
            Ok(())
        }
    }

    pub fn record_command(&mut self, input: String, output: String, duration_ms: Option<u64>) {
        if let Some(id) = &self.active_session {
            if let Some(session) = self.sessions.get_mut(id) {
                session.add_command(input, output, duration_ms);
            }
        }
    }

    pub fn record_file(
        &mut self,
        path: String,
        action: crate::tui::session::session_data::FileAction,
    ) {
        if let Some(id) = &self.active_session {
            if let Some(session) = self.sessions.get_mut(id) {
                session.add_file_record(path, action);
            }
        }
    }

    pub fn save(&self, id: &str) -> io::Result<()> {
        if let Some(session) = self.sessions.get(id) {
            let content = serde_json::to_string_pretty(session)?;
            let session_file = self.sessions_dir.join(format!("{}.json", id));
            fs::write(session_file, content)?;
        }
        Ok(())
    }

    pub fn save_all(&self) -> io::Result<()> {
        for id in self.sessions.keys() {
            self.save(id)?;
        }
        Ok(())
    }

    pub fn load(&mut self, id: &str) -> io::Result<()> {
        let session_file = self.sessions_dir.join(format!("{}.json", id));
        let content = fs::read_to_string(&session_file)?;
        let session: Session = serde_json::from_str(&content)?;

        self.sessions.insert(id.to_string(), session);
        Ok(())
    }

    pub fn load_all(&mut self) -> io::Result<()> {
        if !self.sessions_dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    if let Some(id) = stem.to_str() {
                        self.load(id)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn history(&self) -> Vec<&Session> {
        let mut sessions: Vec<_> = self.sessions.values().collect();
        sessions.sort_by_key(|s| s.timestamp);
        sessions
    }

    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn active(&self) -> Option<&Session> {
        self.active_session
            .as_ref()
            .and_then(|id| self.sessions.get(id))
    }

    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.keys().cloned().collect()
    }

    pub fn delete(&mut self, id: &str) -> io::Result<()> {
        self.sessions.remove(id);

        let session_file = self.sessions_dir.join(format!("{}.json", id));
        if session_file.exists() {
            fs::remove_file(session_file)?;
        }

        if self.active_session.as_deref() == Some(id) {
            self.active_session = None;
        }

        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_session() {
        let mut manager = SessionManager::new();
        let session = manager.start(Some("test-session".to_string()));

        assert_eq!(session.id, "test-session");
        assert!(session.is_active());
    }

    #[test]
    fn test_record_command() {
        let mut manager = SessionManager::new();
        manager.start(Some("test".to_string()));

        manager.record_command(
            "test input".to_string(),
            "test output".to_string(),
            Some(100),
        );

        let session = manager.active().unwrap();
        assert_eq!(session.commands.len(), 1);
        assert_eq!(session.commands[0].input, "test input");
    }

    #[test]
    fn test_save_and_load_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SessionManager::new().with_sessions_dir(temp_dir.path().to_path_buf());

        manager.start(Some("test".to_string()));
        manager.record_command("cmd".to_string(), "output".to_string(), None);
        manager.end("test").unwrap();

        let mut manager2 = SessionManager::new().with_sessions_dir(temp_dir.path().to_path_buf());
        manager2.load_all().unwrap();

        let loaded = manager2.get_session("test").unwrap();
        assert_eq!(loaded.commands.len(), 1);
    }
}
