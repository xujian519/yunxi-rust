use std::collections::VecDeque;
use std::time::Duration;

use crate::error::LlmError;
use crate::openai::types::{ChatCompletionChunk, ChatCompletionRequest};

const DEFAULT_MAX_RETRIES: u32 = 2;
const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_millis(200);
const DEFAULT_MAX_BACKOFF: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
pub struct OpenAiClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
}

impl OpenAiClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(30));
        if std::env::var("YUNXI_NO_PROXY").as_deref() == Ok("1")
            || std::env::var("NO_PROXY").as_deref() == Ok("*")
        {
            builder = builder.no_proxy();
        }
        let http = builder
            .build()
            .expect("reqwest client build should not fail with standard timeouts");
        Self {
            http,
            base_url: base_url.into(),
            api_key: api_key.into(),
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff: DEFAULT_INITIAL_BACKOFF,
            max_backoff: DEFAULT_MAX_BACKOFF,
        }
    }

    #[must_use]
    pub fn with_backoff(
        mut self,
        max_retries: u32,
        initial_backoff: Duration,
        max_backoff: Duration,
    ) -> Self {
        self.max_retries = max_retries;
        self.initial_backoff = initial_backoff;
        self.max_backoff = max_backoff;
        self
    }

    /// 流式聊天完成
    ///
    /// # Errors
    ///
    /// - 如果请求发送失败,返回 Llm 错误
    /// - 如果重试耗尽,返回 Llm 错误
    pub async fn stream_chat(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<OpenAiStream, LlmError> {
        let response = self.send_with_retry(request).await?;
        Ok(OpenAiStream {
            response,
            buffer: String::new(),
            pending: VecDeque::new(),
            done: false,
        })
    }

    async fn send_with_retry(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<reqwest::Response, LlmError> {
        let mut attempts = 0;
        let mut last_error: Option<LlmError>;

        loop {
            attempts += 1;
            match self.send_raw(request).await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    }
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    let retryable = is_retryable_status(status);
                    let error = LlmError::http(format!("HTTP {status}: {body}"));
                    if retryable && attempts <= self.max_retries + 1 {
                        last_error = Some(error);
                    } else {
                        return Err(error);
                    }
                }
                Err(error) => {
                    let retryable = error.is_request() || error.is_connect() || error.is_timeout();
                    let llm_error = LlmError::http(error.to_string());
                    if retryable && attempts <= self.max_retries + 1 {
                        last_error = Some(llm_error);
                    } else {
                        return Err(llm_error);
                    }
                }
            }

            if attempts > self.max_retries {
                break;
            }

            let delay = self.backoff_for_attempt(attempts);
            tokio::time::sleep(delay).await;
        }

        Err(
            #[allow(clippy::expect_used)]
            last_error.expect("retry loop must capture an error"),
        )
    }

    async fn send_raw(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let base = self.base_url.trim_end_matches('/');
        let url = format!("{base}/chat/completions");

        self.http
            .post(&url)
            .header("authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(request)
            .send()
            .await
    }

    fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        let multiplier = 1_u32
            .checked_shl(attempt.saturating_sub(1))
            .unwrap_or(u32::MAX);
        self.initial_backoff
            .checked_mul(multiplier)
            .map_or(self.max_backoff, |delay| delay.min(self.max_backoff))
    }
}

const fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 409 | 429 | 500 | 502 | 503 | 504)
}

#[derive(Debug)]
pub struct OpenAiStream {
    response: reqwest::Response,
    buffer: String,
    pending: VecDeque<ChatCompletionChunk>,
    done: bool,
}

impl OpenAiStream {
    /// 获取下一个块
    ///
    /// # Errors
    ///
    /// - 如果读取失败,返回 Llm 错误
    /// - 如果解析失败,返回 Llm 错误
    pub async fn next_chunk(&mut self) -> Result<Option<ChatCompletionChunk>, LlmError> {
        loop {
            if let Some(chunk) = self.pending.pop_front() {
                return Ok(Some(chunk));
            }

            if self.done {
                return Ok(None);
            }

            match self.response.chunk().await {
                Ok(Some(bytes)) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&bytes));
                    self.parse_buffer();
                }
                Ok(None) => {
                    self.done = true;
                    // 处理 buffer 中剩余数据
                    if !self.buffer.trim().is_empty() {
                        self.parse_buffer();
                    }
                }
                Err(error) => {
                    return Err(LlmError::stream(error.to_string()));
                }
            }
        }
    }

    fn parse_buffer(&mut self) {
        let lines: Vec<&str> = self.buffer.split('\n').collect();
        let mut remaining = String::new();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with(':') {
                continue;
            }

            if trimmed == "data: [DONE]" {
                self.done = true;
                continue;
            }

            if let Some(data) = trimmed.strip_prefix("data: ") {
                if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(data) {
                    self.pending.push_back(chunk);
                }
                continue;
            }

            // 不完整的行，保留到下次处理
            if !trimmed.is_empty() && !trimmed.starts_with("data:") {
                remaining.push_str(line);
                remaining.push('\n');
            }
        }

        self.buffer = remaining;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_client_with_base_url() {
        let client = OpenAiClient::new("https://api.deepseek.com/v1", "test-key");
        assert_eq!(client.base_url, "https://api.deepseek.com/v1");
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn backoff_doubles_until_max() {
        let client = OpenAiClient::new("http://test", "key").with_backoff(
            3,
            Duration::from_millis(10),
            Duration::from_millis(25),
        );
        assert_eq!(client.backoff_for_attempt(1), Duration::from_millis(10));
        assert_eq!(client.backoff_for_attempt(2), Duration::from_millis(20));
        assert_eq!(client.backoff_for_attempt(3), Duration::from_millis(25));
    }

    #[test]
    fn detects_retryable_statuses() {
        assert!(is_retryable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable_status(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(!is_retryable_status(reqwest::StatusCode::UNAUTHORIZED));
    }
}
