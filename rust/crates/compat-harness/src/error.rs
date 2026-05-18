use std::path::PathBuf;

/// 兼容性检查过程中可能发生的错误。
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    /// I/O 操作失败。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON 序列化/反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// 清单文件解析错误。
    #[error("manifest error at {path}: {message}")]
    Manifest {
        path: PathBuf,
        message: String,
    },
    /// 快照比较错误。
    #[error("snapshot error: {0}")]
    Snapshot(String),
}

impl HarnessError {
    pub fn manifest(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Manifest {
            path: path.into(),
            message: message.into(),
        }
    }

    pub fn snapshot(message: impl Into<String>) -> Self {
        Self::Snapshot(message.into())
    }
}
