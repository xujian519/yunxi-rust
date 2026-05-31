//! 远程 HTTP 嵌入服务（如本地 8766 端口的 BGE-M3 服务）

use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::{json, Value};

use crate::config::HttpEmbeddingConfig;
use crate::service::{Embedding, EmbeddingError, EMBEDDING_DIM};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApiStyle {
    OpenAi,
    Tei,
    Simple,
}

impl ApiStyle {
    fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "tei" | "huggingface" => Self::Tei,
            "simple" | "encode" => Self::Simple,
            _ => Self::OpenAi,
        }
    }
}

/// 通过 HTTP 调用远程嵌入 API
pub struct HttpEmbeddingBackend {
    client: Client,
    url: String,
    style: ApiStyle,
    model: Option<String>,
    api_key: Option<String>,
}

impl HttpEmbeddingBackend {
    pub fn new(cfg: &HttpEmbeddingConfig) -> Result<Self, EmbeddingError> {
        let base = cfg.base_url.trim_end_matches('/');
        let path = if cfg.embed_path.starts_with('/') {
            cfg.embed_path.clone()
        } else {
            format!("/{}", cfg.embed_path)
        };
        let url = format!("{base}{path}");

        let client = Client::builder()
            .timeout(Duration::from_secs(cfg.timeout_secs.max(1)))
            .build()
            .map_err(|e| EmbeddingError::Http(e.to_string()))?;

        let api_key = cfg
            .api_key
            .clone()
            .filter(|k| !k.is_empty())
            .or_else(|| {
                std::env::var("EMBEDDING_API_KEY")
                    .ok()
                    .filter(|k| !k.is_empty())
            })
            .or_else(|| std::env::var("OMLX_API_KEY").ok().filter(|k| !k.is_empty()));

        Ok(Self {
            client,
            url,
            style: ApiStyle::parse(&cfg.api_style),
            model: cfg.model.clone(),
            api_key,
        })
    }

    pub fn encode(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let batch = self.encode_batch(&[text])?;
        batch
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::Http("empty embedding response".into()))
    }

    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let body = self.build_request_body(texts);
        let mut req = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&body);
        if let Some(ref key) = self.api_key {
            req = req.bearer_auth(key);
        }
        let response = req
            .send()
            .map_err(|e| EmbeddingError::Http(format!("request failed: {e}")))?;

        let status = response.status();
        let payload: Value = response
            .json()
            .map_err(|e| EmbeddingError::Http(format!("invalid JSON (HTTP {status}): {e}")))?;

        if !status.is_success() {
            return Err(EmbeddingError::Http(format!(
                "HTTP {status}: {}",
                payload
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or(&payload.to_string())
            )));
        }

        parse_embeddings_response(&payload, texts.len())
    }

    /// 异步版本：在 `spawn_blocking` 中执行 HTTP 请求，避免阻塞 tokio 运行时。
    pub async fn encode_async(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        let batch = self.encode_batch_async(&[text]).await?;
        batch
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::Http("empty embedding response".into()))
    }

    /// 异步版本：在 `spawn_blocking` 中执行 HTTP 请求，避免阻塞 tokio 运行时。
    pub async fn encode_batch_async(
        &self,
        texts: &[&str],
    ) -> Result<Vec<Embedding>, EmbeddingError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        let this = self.clone();
        let texts_owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        tokio::task::spawn_blocking(move || {
            let texts_ref: Vec<&str> = texts_owned.iter().map(|s| s.as_str()).collect();
            this.encode_batch(&texts_ref)
        })
        .await
        .map_err(|e| EmbeddingError::Http(format!("spawn_blocking failed: {e}")))?
    }
}

impl Clone for HttpEmbeddingBackend {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            url: self.url.clone(),
            style: self.style,
            model: self.model.clone(),
            api_key: self.api_key.clone(),
        }
    }
}

impl HttpEmbeddingBackend {
    fn build_request_body(&self, texts: &[&str]) -> Value {
        match self.style {
            ApiStyle::OpenAi => {
                let input = if texts.len() == 1 {
                    json!(texts[0])
                } else {
                    json!(texts)
                };
                let mut body = json!({ "input": input });
                if let Some(model) = &self.model {
                    body["model"] = json!(model);
                }
                body
            }
            ApiStyle::Tei => {
                let inputs = if texts.len() == 1 {
                    json!(texts[0])
                } else {
                    json!(texts)
                };
                json!({ "inputs": inputs })
            }
            ApiStyle::Simple => {
                if texts.len() == 1 {
                    json!({ "text": texts[0] })
                } else {
                    json!({ "texts": texts })
                }
            }
        }
    }
}

fn parse_embeddings_response(
    payload: &Value,
    expected: usize,
) -> Result<Vec<Embedding>, EmbeddingError> {
    // OpenAI: { "data": [ { "embedding": [...] }, ... ] }
    if let Some(data) = payload.get("data").and_then(|d| d.as_array()) {
        let mut out = Vec::with_capacity(data.len());
        for item in data {
            let emb = item
                .get("embedding")
                .and_then(|e| e.as_array())
                .ok_or_else(|| EmbeddingError::Http("missing embedding in data[]".into()))?;
            out.push(parse_f32_array(emb)?);
        }
        return normalize_batch(out, expected);
    }

    // TEI / 部分服务: 直接返回二维数组 [[...], ...]
    if let Some(arr) = payload.as_array() {
        if arr.first().and_then(|x| x.as_array()).is_some() {
            let mut out = Vec::with_capacity(arr.len());
            for row in arr {
                let emb = row
                    .as_array()
                    .ok_or_else(|| EmbeddingError::Http("expected array of vectors".into()))?;
                out.push(parse_f32_array(emb)?);
            }
            return normalize_batch(out, expected);
        }
    }

    // { "embeddings": [[...]] } 或 { "embedding": [...] }
    if let Some(embeddings) = payload.get("embeddings").and_then(|e| e.as_array()) {
        let mut out = Vec::with_capacity(embeddings.len());
        for row in embeddings {
            let emb = row
                .as_array()
                .ok_or_else(|| EmbeddingError::Http("embeddings[] must be arrays".into()))?;
            out.push(parse_f32_array(emb)?);
        }
        return normalize_batch(out, expected);
    }

    if let Some(single) = payload.get("embedding").and_then(|e| e.as_array()) {
        return normalize_batch(vec![parse_f32_array(single)?], expected);
    }

    // { "vector": [...] }
    if let Some(single) = payload.get("vector").and_then(|e| e.as_array()) {
        return normalize_batch(vec![parse_f32_array(single)?], expected);
    }

    Err(EmbeddingError::Http(format!(
        "unrecognized embedding response shape: {}",
        truncate_json(payload)
    )))
}

fn parse_f32_array(values: &[Value]) -> Result<Embedding, EmbeddingError> {
    let mut vec: Vec<f32> = values
        .iter()
        .map(|v| {
            v.as_f64()
                .ok_or_else(|| EmbeddingError::Http("non-numeric embedding value".into()))
                .map(|f| f as f32)
        })
        .collect::<Result<_, _>>()?;
    l2_normalize(&mut vec);
    Ok(vec)
}

fn normalize_batch(
    mut batch: Vec<Embedding>,
    expected: usize,
) -> Result<Vec<Embedding>, EmbeddingError> {
    if batch.len() != expected {
        return Err(EmbeddingError::Http(format!(
            "expected {expected} embeddings, got {}",
            batch.len()
        )));
    }
    for v in &mut batch {
        if v.len() != EMBEDDING_DIM {
            v.resize(EMBEDDING_DIM, 0.0);
            l2_normalize(v);
        }
    }
    Ok(batch)
}

fn l2_normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v {
            *x /= norm;
        }
    }
}

fn truncate_json(v: &Value) -> String {
    let s = v.to_string();
    if s.len() > 200 {
        format!("{}…", &s[..200])
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_shape() {
        let payload = json!({
            "data": [
                { "embedding": [1.0, 0.0, 0.0] },
                { "embedding": [0.0, 1.0, 0.0] }
            ]
        });
        let vecs = parse_embeddings_response(&payload, 2).unwrap();
        assert_eq!(vecs.len(), 2);
    }
}
