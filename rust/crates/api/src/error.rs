use std::env::VarError;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Debug)]
pub enum ApiError {
    MissingApiKey,
    ExpiredOAuthToken,
    Auth(String),
    InvalidApiKeyEnv(VarError),
    Http(reqwest::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
    Api {
        status: reqwest::StatusCode,
        error_type: Option<String>,
        message: Option<String>,
        body: String,
        retryable: bool,
    },
    RetriesExhausted {
        attempts: u32,
        last_error: Box<ApiError>,
    },
    InvalidSseFrame(&'static str),
    BackoffOverflow {
        attempt: u32,
        base_delay: Duration,
    },
}

impl ApiError {
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http(error) => error.is_connect() || error.is_timeout() || error.is_request(),
            Self::Api { retryable, .. } => *retryable,
            Self::RetriesExhausted { last_error, .. } => last_error.is_retryable(),
            Self::MissingApiKey
            | Self::ExpiredOAuthToken
            | Self::Auth(_)
            | Self::InvalidApiKeyEnv(_)
            | Self::Io(_)
            | Self::Json(_)
            | Self::InvalidSseFrame(_)
            | Self::BackoffOverflow { .. } => false,
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingApiKey => {
                write!(
                    f,
                    "ANTHROPIC_AUTH_TOKEN or ANTHROPIC_API_KEY is not set; export one before calling the Anthropic API"
                )
            }
            Self::ExpiredOAuthToken => {
                write!(
                    f,
                    "saved OAuth token is expired and no refresh token is available"
                )
            }
            Self::Auth(message) => write!(f, "auth error: {message}"),
            Self::InvalidApiKeyEnv(error) => {
                write!(
                    f,
                    "failed to read ANTHROPIC_AUTH_TOKEN / ANTHROPIC_API_KEY: {error}"
                )
            }
            Self::Http(error) => write!(f, "http error: {error}"),
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::Api {
                status,
                error_type,
                message,
                body,
                ..
            } => match (error_type, message) {
                (Some(error_type), Some(message)) => {
                    write!(
                        f,
                        "anthropic api returned {status} ({error_type}): {message}"
                    )
                }
                _ => write!(f, "anthropic api returned {status}: {body}"),
            },
            Self::RetriesExhausted {
                attempts,
                last_error,
            } => write!(
                f,
                "anthropic api failed after {attempts} attempts: {last_error}"
            ),
            Self::InvalidSseFrame(message) => write!(f, "invalid sse frame: {message}"),
            Self::BackoffOverflow {
                attempt,
                base_delay,
            } => write!(
                f,
                "retry backoff overflowed on attempt {attempt} with base delay {base_delay:?}"
            ),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<reqwest::Error> for ApiError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<VarError> for ApiError {
    fn from(value: VarError) -> Self {
        Self::InvalidApiKeyEnv(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_retryable_missing_api_key() {
        assert!(!ApiError::MissingApiKey.is_retryable());
    }

    #[test]
    fn is_retryable_expired_oauth_token() {
        assert!(!ApiError::ExpiredOAuthToken.is_retryable());
    }

    #[test]
    fn is_retryable_auth() {
        assert!(!ApiError::Auth(String::from("bad token")).is_retryable());
    }

    #[test]
    fn is_retryable_invalid_api_key_env() {
        assert!(!ApiError::InvalidApiKeyEnv(VarError::NotPresent).is_retryable());
    }

    #[test]
    fn is_retryable_io() {
        assert!(!ApiError::Io(std::io::Error::other("disk")).is_retryable());
    }

    #[test]
    fn is_retryable_json() {
        let err = serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err();
        assert!(!ApiError::Json(err).is_retryable());
    }

    #[test]
    fn is_retryable_api_retryable_true() {
        let err = ApiError::Api {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            error_type: None,
            message: None,
            body: String::from("boom"),
            retryable: true,
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn is_retryable_api_retryable_false() {
        let err = ApiError::Api {
            status: reqwest::StatusCode::BAD_REQUEST,
            error_type: None,
            message: None,
            body: String::from("bad"),
            retryable: false,
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn is_retryable_retries_exhausted_inner_retryable() {
        let err = ApiError::RetriesExhausted {
            attempts: 3,
            last_error: Box::new(ApiError::Api {
                status: reqwest::StatusCode::SERVICE_UNAVAILABLE,
                error_type: None,
                message: None,
                body: String::new(),
                retryable: true,
            }),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn is_retryable_retries_exhausted_inner_not_retryable() {
        let err = ApiError::RetriesExhausted {
            attempts: 3,
            last_error: Box::new(ApiError::MissingApiKey),
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn is_retryable_invalid_sse_frame() {
        assert!(!ApiError::InvalidSseFrame("bad frame").is_retryable());
    }

    #[test]
    fn is_retryable_backoff_overflow() {
        assert!(!ApiError::BackoffOverflow {
            attempt: 50,
            base_delay: Duration::from_secs(1),
        }
        .is_retryable());
    }

    #[test]
    fn display_missing_api_key() {
        let msg = ApiError::MissingApiKey.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("ANTHROPIC"));
    }

    #[test]
    fn display_expired_oauth_token() {
        let msg = ApiError::ExpiredOAuthToken.to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("expired"));
    }

    #[test]
    fn display_auth() {
        let msg = ApiError::Auth(String::from("denied")).to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("denied"));
    }

    #[test]
    fn display_invalid_api_key_env() {
        let msg = ApiError::InvalidApiKeyEnv(VarError::NotPresent).to_string();
        assert!(!msg.is_empty());
    }

    #[test]
    fn display_io() {
        let msg = ApiError::Io(std::io::Error::other("fail")).to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("io error"));
    }

    #[test]
    fn display_json() {
        let err = serde_json::from_str::<serde_json::Value>("!!!").unwrap_err();
        let msg = ApiError::Json(err).to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("json error"));
    }

    #[test]
    fn display_api_with_type_and_message() {
        let msg = ApiError::Api {
            status: reqwest::StatusCode::BAD_REQUEST,
            error_type: Some(String::from("invalid_request")),
            message: Some(String::from("bad input")),
            body: String::new(),
            retryable: false,
        }
        .to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("400"));
        assert!(msg.contains("invalid_request"));
        assert!(msg.contains("bad input"));
    }

    #[test]
    fn display_api_without_type_and_message() {
        let msg = ApiError::Api {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            error_type: None,
            message: None,
            body: String::from("server error"),
            retryable: true,
        }
        .to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("500"));
        assert!(msg.contains("server error"));
    }

    #[test]
    fn display_retries_exhausted() {
        let msg = ApiError::RetriesExhausted {
            attempts: 5,
            last_error: Box::new(ApiError::MissingApiKey),
        }
        .to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("5"));
    }

    #[test]
    fn display_invalid_sse_frame() {
        let msg = ApiError::InvalidSseFrame("truncated").to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("truncated"));
    }

    #[test]
    fn display_backoff_overflow() {
        let msg = ApiError::BackoffOverflow {
            attempt: 99,
            base_delay: Duration::from_millis(200),
        }
        .to_string();
        assert!(!msg.is_empty());
        assert!(msg.contains("99"));
    }
}
