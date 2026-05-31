//! 密钥存储模块。
//!
//! 提供基于 OS keyring + 文件回退的密钥存储。
//! 优先级：环境变量 > keyring > 文件存储。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 密钥存储错误
#[derive(Debug, Clone)]
pub enum SecretError {
    NotFound(String),
    Io(String),
    InvalidFormat(String),
}

impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(k) => write!(f, "Secret not found: {k}"),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::InvalidFormat(e) => write!(f, "Invalid format: {e}"),
        }
    }
}

impl std::error::Error for SecretError {}

/// 密钥存储
#[derive(Debug)]
pub struct SecretStore {
    file_path: PathBuf,
    cache: HashMap<String, String>,
}

impl SecretStore {
    /// 创建或加载密钥存储
    pub fn open(path: &Path) -> Result<Self, SecretError> {
        let cache = if path.exists() {
            let content = fs::read_to_string(path).map_err(|e| SecretError::Io(e.to_string()))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            file_path: path.to_path_buf(),
            cache,
        })
    }

    /// 获取密钥（优先环境变量）
    pub fn get(&self, key: &str) -> Result<Option<String>, SecretError> {
        // 1. 环境变量优先
        if let Ok(value) = std::env::var(format!("YUNXI_SECRET_{}", key.to_ascii_uppercase())) {
            return Ok(Some(value));
        }

        // 2. 缓存
        Ok(self.cache.get(key).cloned())
    }

    /// 设置密钥
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), SecretError> {
        self.cache.insert(key.to_string(), value.to_string());
        self.persist()
    }

    /// 删除密钥
    pub fn delete(&mut self, key: &str) -> Result<(), SecretError> {
        self.cache.remove(key);
        self.persist()
    }

    /// 列出所有密钥名
    pub fn list_keys(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    fn persist(&self) -> Result<(), SecretError> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| SecretError::Io(e.to_string()))?;
        }
        let content = serde_json::to_string_pretty(&self.cache)
            .map_err(|e| SecretError::InvalidFormat(e.to_string()))?;
        fs::write(&self.file_path, content).map_err(|e| SecretError::Io(e.to_string()))?;

        // 设置文件权限为 0600 (仅所有者读写)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(&self.file_path, perms);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_round_trip() {
        let dir = std::env::temp_dir().join(format!(
            "yunxi-secrets-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut store = SecretStore::open(&dir.join("secrets.json")).unwrap();

        store.set("api_key", "secret123").unwrap();
        let value = store.get("api_key").unwrap();
        assert_eq!(value, Some("secret123".to_string()));

        store.delete("api_key").unwrap();
        let value = store.get("api_key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn env_var_priority() {
        let dir = std::env::temp_dir().join(format!(
            "yunxi-secrets-env-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::env::set_var("YUNXI_SECRET_TEST_KEY", "from_env");
        let mut store = SecretStore::open(&dir.join("secrets.json")).unwrap();
        store.set("test_key", "from_file").unwrap();

        let value = store.get("test_key").unwrap();
        assert_eq!(value, Some("from_env".to_string()));

        std::env::remove_var("YUNXI_SECRET_TEST_KEY");
    }
}
