/// CLI 执行过程中可能发生的错误。
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// 命令行参数解析失败。
    #[error("argument error: {0}")]
    Args(String),
    /// 运行时错误。
    #[error("runtime error: {0}")]
    Runtime(String),
    /// I/O 操作失败。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON 序列化/反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl CliError {
    pub fn args(message: impl Into<String>) -> Self {
        Self::Args(message.into())
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into())
    }
}
