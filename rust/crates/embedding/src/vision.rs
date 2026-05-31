//! 本地 oMLX / OpenAI 兼容多模态服务（默认 :8009）图片文字识别。

use std::fs;
use std::path::Path;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use reqwest::blocking::Client;
use serde_json::{json, Value};

use crate::config::{load_vision_config, ResolvedVisionHttp};

const MAX_IMAGE_BYTES: u64 = 24 * 1024 * 1024;

/// 多模态 OCR 结果。
#[derive(Debug, Clone)]
pub struct VisionOcrResult {
    pub text: String,
    pub model: String,
    pub base_url: String,
}

/// 是否已配置为使用本地 vision（settings 或环境变量）。
#[must_use]
pub fn vision_ocr_configured() -> bool {
    let v = load_vision_config();
    v.enabled || v.http.model.is_some() || env_truthy("YUNXI_VISION_ENABLED")
}

/// 从图片路径调用本地多模态模型做 OCR/文字转录。
///
/// # Errors
///
/// 服务不可用、鉴权失败、响应无法解析或文件过大时返回错误信息字符串。
pub fn ocr_image_from_path(
    path: &Path,
    prompt_override: Option<&str>,
) -> Result<VisionOcrResult, String> {
    ocr_image_from_path_impl(path, prompt_override)
}

/// 异步版本：在 `spawn_blocking` 中执行 OCR，避免阻塞 tokio 运行时。
pub async fn ocr_image_from_path_async(
    path: &Path,
    prompt_override: Option<&str>,
) -> Result<VisionOcrResult, String> {
    let path = path.to_path_buf();
    let prompt = prompt_override.map(str::to_string);
    tokio::task::spawn_blocking(move || ocr_image_from_path_impl(&path, prompt.as_deref()))
        .await
        .map_err(|e| format!("spawn_blocking failed: {e}"))?
}

fn ocr_image_from_path_impl(
    path: &Path,
    prompt_override: Option<&str>,
) -> Result<VisionOcrResult, String> {
    let resolved = ResolvedVisionHttp::resolve()?;
    let meta = fs::metadata(path).map_err(|e| format!("无法读取图片: {e}"))?;
    if meta.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "图片超过 {} MiB 上限",
            MAX_IMAGE_BYTES / 1024 / 1024
        ));
    }

    let bytes = fs::read(path).map_err(|e| format!("读取图片失败: {e}"))?;
    let mime = guess_image_mime(path);
    let data_url = format!("data:{mime};base64,{}", B64.encode(bytes));

    let prompt = prompt_override
        .map(str::to_string)
        .unwrap_or_else(|| resolved.ocr_prompt.clone());

    let body = json!({
        "model": resolved.model,
        "messages": [{
            "role": "user",
            "content": [
                { "type": "text", "text": prompt },
                { "type": "image_url", "image_url": { "url": data_url } }
            ]
        }],
        "max_tokens": resolved.max_tokens,
        "temperature": 0.1
    });

    let url = format!(
        "{}{}",
        resolved.base_url.trim_end_matches('/'),
        resolved.chat_path
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(resolved.timeout_secs.max(30)))
        .build()
        .map_err(|e| format!("HTTP 客户端: {e}"))?;

    let mut req = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body);
    if let Some(ref key) = resolved.api_key {
        req = req.bearer_auth(key);
    }

    let response = req
        .send()
        .map_err(|e| format!("Vision OCR 请求失败 ({url}): {e}"))?;

    let status = response.status();
    let payload: Value = response
        .json()
        .map_err(|e| format!("Vision OCR 响应非 JSON (HTTP {status}): {e}"))?;

    if !status.is_success() {
        let fallback = payload.to_string();
        let msg = payload
            .pointer("/error/message")
            .and_then(|m| m.as_str())
            .unwrap_or(fallback.as_str());
        return Err(format!("Vision OCR HTTP {status}: {msg}"));
    }

    let text = extract_message_content(&payload)
        .ok_or_else(|| format!("无法解析模型输出: {}", truncate(&payload.to_string(), 240)))?;

    Ok(VisionOcrResult {
        text: text.trim().to_string(),
        model: resolved.model,
        base_url: resolved.base_url,
    })
}

fn extract_message_content(payload: &Value) -> Option<String> {
    if let Some(content) = payload
        .pointer("/choices/0/message/content")
        .and_then(|c| c.as_str())
    {
        return Some(content.to_string());
    }
    // 部分服务把正文放在 text 字段
    payload
        .pointer("/choices/0/message")
        .and_then(|m| m.get("text"))
        .and_then(|t| t.as_str())
        .map(str::to_string)
}

fn guess_image_mime(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tif" | "tiff") => "image/tiff",
        _ => "image/png",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    format!("{}…", s.chars().take(max).collect::<String>())
}

fn env_truthy(key: &str) -> bool {
    std::env::var(key).ok().is_some_and(|v| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chat_completion_content() {
        let payload = json!({
            "choices": [{ "message": { "content": "权利要求 1 …" } }]
        });
        assert_eq!(
            extract_message_content(&payload).as_deref(),
            Some("权利要求 1 …")
        );
    }
}
