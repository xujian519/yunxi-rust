use super::Workspace;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct WorkspaceManager {
    workspaces: HashMap<String, Workspace>,
    current_workspace: Option<String>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: HashMap::new(),
            current_workspace: None,
        }
    }

    pub fn create(&mut self, name: String, path: PathBuf) -> io::Result<&Workspace> {
        let workspace = Workspace::new(name.clone(), path);

        if let Some(workspace_file) = workspace.workspace_file().parent() {
            fs::create_dir_all(workspace_file)?;
        }

        self.workspaces.insert(name.clone(), workspace);
        Ok(&self.workspaces[&name])
    }

    pub fn open(&mut self, path: &Path) -> io::Result<Workspace> {
        let workspace_path = path.join(".yunxi-workspace.json");
        let content = fs::read_to_string(&workspace_path)?;
        let workspace: Workspace = serde_json::from_str(&content)?;

        let name = workspace.name.clone();
        self.workspaces.insert(name, workspace.clone());
        Ok(workspace)
    }

    pub fn close(&mut self, name: &str) -> io::Result<()> {
        if let Some(workspace) = self.workspaces.remove(name) {
            if workspace.workspace_file().exists() {
                fs::remove_file(workspace.workspace_file())?;
            }
        }
        Ok(())
    }

    pub fn save(&self, name: &str) -> io::Result<()> {
        if let Some(workspace) = self.workspaces.get(name) {
            let workspace_file = workspace.workspace_file();
            let content = serde_json::to_string_pretty(workspace)?;
            fs::write(&workspace_file, content)?;
        }
        Ok(())
    }

    pub fn save_all(&self) -> io::Result<()> {
        for name in self.workspaces.keys() {
            self.save(name)?;
        }
        Ok(())
    }

    pub fn switch(&mut self, name: &str) -> Option<&Workspace> {
        if self.workspaces.contains_key(name) {
            self.current_workspace = Some(name.to_string());
            self.workspaces.get(name)
        } else {
            None
        }
    }

    pub fn delete(&mut self, name: &str) -> io::Result<()> {
        if let Some(workspace) = self.workspaces.remove(name) {
            if workspace.path.exists() {
                fs::remove_dir_all(&workspace.path)?;
            }
        }
        if self.current_workspace.as_deref() == Some(name) {
            self.current_workspace = None;
        }
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Workspace> {
        self.workspaces.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Workspace> {
        self.workspaces.get_mut(name)
    }

    pub fn current(&self) -> Option<&Workspace> {
        self.current_workspace
            .as_ref()
            .and_then(|name| self.workspaces.get(name))
    }

    pub fn current_mut(&mut self) -> Option<&mut Workspace> {
        self.current_workspace
            .as_ref()
            .and_then(|name| self.workspaces.get_mut(name))
    }

    pub fn list(&self) -> Vec<&Workspace> {
        self.workspaces.values().collect()
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::new();
        let workspace = manager
            .create("test".to_string(), temp_dir.path().to_path_buf())
            .unwrap();

        assert_eq!(workspace.name, "test");
        assert_eq!(workspace.path, temp_dir.path());
    }

    #[test]
    #[ignore]
    fn test_save_and_load_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::new();
        manager
            .create("test".to_string(), temp_dir.path().to_path_buf())
            .unwrap();

        let workspace = manager.get_mut("test").unwrap();
        workspace.open_files.push(PathBuf::from("file1.txt"));
        workspace.open_files.push(PathBuf::from("file2.txt"));

        manager.save("test").unwrap();

        let mut manager2 = WorkspaceManager::new();
        let loaded = manager2.open(temp_dir.path()).unwrap();
        assert_eq!(loaded.open_files.len(), 2);
    }

    #[test]
    fn test_switch_workspace() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        let mut manager = WorkspaceManager::new();

        manager
            .create("workspace1".to_string(), temp_dir1.path().to_path_buf())
            .unwrap();
        manager
            .create("workspace2".to_string(), temp_dir2.path().to_path_buf())
            .unwrap();

        let current = manager.switch("workspace1").unwrap();
        assert_eq!(current.name, "workspace1");

        let current = manager.switch("workspace2").unwrap();
        assert_eq!(current.name, "workspace2");
    }
}
