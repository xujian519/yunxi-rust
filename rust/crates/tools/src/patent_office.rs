//! 办公格式扩展：MarkItDown 桥接 + oMLX 多模态 OCR + Tesseract 回退。

use std::path::{Path, PathBuf};
use std::process::Command;

use embedding::vision::{ocr_image_from_path, vision_ocr_configured};
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_FILE_SIZE_BYTES: u64 = 80 * 1024 * 1024;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkItDownConvertInput {
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalOcrInput {
    pub file_path: String,
    #[serde(default = "default_ocr_lang")]
    pub lang: String,
    /// `auto`：优先 oMLX :8009 多模态，失败则 Tesseract；`tesseract`：仅本地 Tesseract；`omlx`：仅多模态。
    #[serde(default = "default_ocr_backend")]
    pub backend: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionOcrInput {
    pub file_path: String,
    /// 覆盖默认 OCR 提示词（可选）。
    pub prompt: Option<String>,
}

fn default_ocr_lang() -> String {
    std::env::var("YUNXI_OCR_LANG").unwrap_or_else(|_| "chi_sim+eng".to_string())
}

fn default_ocr_backend() -> String {
    "auto".to_string()
}

/// MarkItDown 转 Markdown（Python 子进程）。
pub fn run_markitdown_convert(input: &MarkItDownConvertInput) -> Result<String, String> {
    let path = Path::new(&input.file_path);
    check_size(path)?;

    let script = resolve_markitdown_script();
    let python = std::env::var("YUNXI_MARKITDOWN_PYTHON").unwrap_or_else(|_| "python3".to_string());

    let output = Command::new(&python)
        .arg(&script)
        .arg(path)
        .output()
        .map_err(|e| format!("无法启动 MarkItDown 脚本 ({python}): {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("MarkItDownConvert 失败: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value =
        serde_json::from_str(stdout.trim()).map_err(|e| format!("MarkItDown 输出非 JSON: {e}"))?;
    serde_json::to_string(&value).map_err(|e| e.to_string())
}

/// oMLX / OpenAI 兼容多模态 OCR（默认 `http://127.0.0.1:8009/v1/chat/completions`）。
pub fn run_vision_ocr(input: &VisionOcrInput) -> Result<String, String> {
    let path = Path::new(&input.file_path);
    check_size(path)?;

    let result = ocr_image_from_path(path, input.prompt.as_deref())?;
    let payload = json!({
        "file_path": path.display().to_string(),
        "engine": "omlx-vision",
        "model": result.model,
        "base_url": result.base_url,
        "text": result.text,
        "char_count": result.text.chars().count(),
    });
    serde_json::to_string(&payload).map_err(|e| e.to_string())
}

/// 本地 OCR：`auto` 优先多模态（:8009），失败回退 Tesseract。
pub fn run_local_ocr(input: &LocalOcrInput) -> Result<String, String> {
    let path = Path::new(&input.file_path);
    check_size(path)?;

    let backend = input.backend.trim().to_ascii_lowercase();
    match backend.as_str() {
        "omlx" | "vision" | "mlx" => {
            return run_vision_ocr(&VisionOcrInput {
                file_path: input.file_path.clone(),
                prompt: None,
            })
        }
        "tesseract" | "local" => {}
        "auto" | "" => {
            if vision_ocr_configured() || omlx_likely_available() {
                if let Ok(out) = run_vision_ocr(&VisionOcrInput {
                    file_path: input.file_path.clone(),
                    prompt: None,
                }) {
                    return Ok(out);
                }
            }
        }
        other => {
            return Err(format!(
                "未知 backend '{other}'，请使用 auto | omlx | tesseract"
            ));
        }
    }

    run_tesseract_ocr(path, &input.lang)
}

fn omlx_likely_available() -> bool {
    if vision_ocr_configured() {
        return true;
    }
    let semantic = embedding::load_semantic_config();
    if semantic.enabled && semantic.http.base_url.contains(":8009") {
        return true;
    }
    let vision = embedding::load_vision_config();
    if vision.enabled {
        return true;
    }
    if vision
        .http
        .base_url
        .as_ref()
        .is_some_and(|u| u.contains(":8009"))
    {
        return true;
    }
    std::env::var("OMLX_API_KEY")
        .or_else(|_| std::env::var("EMBEDDING_API_KEY"))
        .is_ok_and(|k| !k.is_empty())
}

fn run_tesseract_ocr(path: &Path, lang: &str) -> Result<String, String> {
    let tesseract =
        std::env::var("YUNXI_TESSERACT_CMD").unwrap_or_else(|_| "tesseract".to_string());

    let output = Command::new(&tesseract)
        .arg(path)
        .arg("stdout")
        .arg("-l")
        .arg(lang)
        .output()
        .map_err(|e| {
            format!(
                "无法调用 tesseract: {e}。请安装：macOS `brew install tesseract tesseract-lang`"
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("LocalOcr 失败: {stderr}"));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let result = json!({
        "file_path": path.display().to_string(),
        "engine": "tesseract",
        "lang": lang,
        "text": text,
        "char_count": text.chars().count(),
    });
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

fn check_size(path: &Path) -> Result<(), String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("无法读取文件: {e}"))?;
    if meta.len() > MAX_FILE_SIZE_BYTES {
        return Err(format!(
            "文件超过 {} MiB 上限",
            MAX_FILE_SIZE_BYTES / 1024 / 1024
        ));
    }
    Ok(())
}

fn resolve_markitdown_script() -> PathBuf {
    if let Ok(path) = std::env::var("YUNXI_MARKITDOWN_SCRIPT") {
        return PathBuf::from(path);
    }
    // 开发时从 tools crate 定位仓库 scripts/patent
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest.join("../../scripts/patent/markitdown_convert.py");
    if candidate.is_file() {
        return candidate;
    }
    PathBuf::from("scripts/patent/markitdown_convert.py")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_markitdown_script_path() {
        let p = resolve_markitdown_script();
        assert!(p.to_string_lossy().contains("markitdown_convert.py"));
    }
}
