/// 命令解析与执行过程中可能发生的错误。
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    /// 命令解析失败。
    #[error("parse error: {0}")]
    Parse(String),
    /// JSON 序列化/反序列化失败。
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    /// I/O 操作失败。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// 运行时错误。
    #[error("runtime error: {0}")]
    Runtime(String),
}

impl CommandError {
    pub fn parse(message: impl Into<String>) -> Self {
        Self::Parse(message.into())
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into())
    }
}
