use std::path::PathBuf;

/// 工具执行过程中可能发生的错误。
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// JSON 序列化/反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// I/O 操作失败。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// HTTP 请求失败。
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    /// 配置错误。
    #[error("config error: {0}")]
    Config(String),
    /// 专利领域操作失败。
    #[error("patent operation failed: {0}")]
    Patent(String),
    /// 文件路径解析失败。
    #[error("invalid file path: {0}")]
    Path(PathBuf),
    /// 通用执行错误。
    #[error("{0}")]
    Execution(String),
}

impl ToolError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn patent(message: impl Into<String>) -> Self {
        Self::Patent(message.into())
    }

    pub fn execution(message: impl Into<String>) -> Self {
        Self::Execution(message.into())
    }
}
