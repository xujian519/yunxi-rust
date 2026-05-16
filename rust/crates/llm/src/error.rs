use std::fmt;

#[derive(Debug)]
pub enum LlmError {
    Config(String),
    Auth(String),
    Http(String),
    Stream(String),
}

impl LlmError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    pub fn http(message: impl Into<String>) -> Self {
        Self::Http(message.into())
    }

    pub fn stream(message: impl Into<String>) -> Self {
        Self::Stream(message.into())
    }
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) => write!(f, "llm config error: {msg}"),
            Self::Auth(msg) => write!(f, "llm auth error: {msg}"),
            Self::Http(msg) => write!(f, "llm http error: {msg}"),
            Self::Stream(msg) => write!(f, "llm stream error: {msg}"),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<runtime::RuntimeError> for LlmError {
    fn from(value: runtime::RuntimeError) -> Self {
        Self::Stream(value.to_string())
    }
}
