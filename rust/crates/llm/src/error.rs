use std::fmt;

/// Errors that can occur during LLM interactions.
#[derive(Debug)]
pub enum LlmError {
    /// Invalid or missing configuration (e.g. model name, endpoint URL).
    Config(String),
    /// Authentication failure (e.g. invalid API key).
    Auth(String),
    /// HTTP transport error when calling the LLM endpoint.
    Http(String),
    /// Error while processing the streaming response.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_constructor() {
        let err = LlmError::config("missing key");
        match &err {
            LlmError::Config(msg) => assert_eq!(msg, "missing key"),
            _ => panic!("expected Config variant"),
        }
    }

    #[test]
    fn auth_constructor() {
        let err = LlmError::auth("forbidden");
        match &err {
            LlmError::Auth(msg) => assert_eq!(msg, "forbidden"),
            _ => panic!("expected Auth variant"),
        }
    }

    #[test]
    fn http_constructor() {
        let err = LlmError::http("timeout");
        match &err {
            LlmError::Http(msg) => assert_eq!(msg, "timeout"),
            _ => panic!("expected Http variant"),
        }
    }

    #[test]
    fn stream_constructor() {
        let err = LlmError::stream("broken pipe");
        match &err {
            LlmError::Stream(msg) => assert_eq!(msg, "broken pipe"),
            _ => panic!("expected Stream variant"),
        }
    }

    #[test]
    fn display_config() {
        let msg = LlmError::Config(String::from("bad cfg")).to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("config"));
    }

    #[test]
    fn display_auth() {
        let msg = LlmError::Auth(String::from("denied")).to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("auth"));
    }

    #[test]
    fn display_http() {
        let msg = LlmError::Http(String::from("503")).to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("http"));
    }

    #[test]
    fn display_stream() {
        let msg = LlmError::Stream(String::from("eof")).to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("stream"));
    }
}
