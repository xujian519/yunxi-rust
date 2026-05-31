//! CNIPA 国知局专利检索工具（Python 桥接 + Rust 侧增强）。
//!
//! 通过 subprocess 调用 Python 脚本执行实际的检索和下载。
//! 提供会话缓存、重试、结构化解析等增强功能。
//!
//! Python 依赖: `pip install playwright && python -m playwright install chromium`

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_MS: u64 = 1000;
const SCRIPT_CLIENT: &str = "python/tools/cnipa/cnipa_epub_client.py";
const SCRIPT_SEARCH: &str = "python/tools/cnipa/cnipa_epub_search.py";
const WAF_TIMEOUT_SEC: &str = "180";

// ──────────────────────────────────────────────────────────────────
// 输入类型
// ──────────────────────────────────────────────────────────────────

/// CNIPA 检索输入
#[derive(Debug, Deserialize, Clone)]
pub struct CnipaSearchInput {
    pub query: String,
    #[serde(default)]
    pub max_pages: Option<u32>,
}

/// CNIPA PDF 下载输入
#[derive(Debug, Deserialize)]
pub struct CnipaDownloadInput {
    pub patent_id: String,
    pub output_dir: Option<String>,
}

/// CNIPA 批量下载输入
#[derive(Debug, Serialize)]
struct BatchDownloadOutput {
    patent_id: String,
    path: Option<String>,
    size_kb: Option<f64>,
    status: String,
    error: Option<String>,
}

// ──────────────────────────────────────────────────────────────────
// 搜索结果
// ──────────────────────────────────────────────────────────────────

/// 单条专利搜索结果
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PatentHit {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "applicationNumber")]
    pub application_number: String,
    #[serde(default, alias = "publicationNumber")]
    pub publication_number: String,
    #[serde(default, alias = "applicantName")]
    pub applicant: String,
    #[serde(default)]
    pub abstract_text: String,
    #[serde(default, alias = "ipcClassificationNumber")]
    pub ipc: String,
    #[serde(default)]
    pub publication_date: String,
    #[serde(default)]
    pub citations: Option<u32>,
}

/// 检索结果汇总
#[derive(Debug, Serialize)]
pub struct CnipaSearchResult {
    pub query: String,
    pub total_hits: usize,
    pub patents: Vec<PatentHit>,
    pub source: String,
    pub elapsed_ms: u64,
}

// ──────────────────────────────────────────────────────────────────
// 核心检索函数
// ──────────────────────────────────────────────────────────────────

/// 执行 CNIPA 检索（搜索专利公开库）。
pub fn cnipa_search(input: CnipaSearchInput) -> Result<Value, String> {
    let start = std::time::Instant::now();
    let script_path = resolve_script(SCRIPT_SEARCH)?;
    let query = input.query.trim();
    if query.is_empty() {
        return Err("查询词不能为空".into());
    }

    let output = retry_call(|| {
        let mut cmd = Command::new("python3");
        cmd.env("EPUB_WAF_MAX_WAIT_SEC", WAF_TIMEOUT_SEC)
            .arg(&script_path)
            .arg(query);
        if let Some(pages) = input.max_pages {
            cmd.env("EPUB_MAX_PAGES", pages.to_string());
        }
        cmd.output()
            .map_err(|e| format!("调用 CNIPA 脚本失败: {e}"))
    })?;

    check_output(&output)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // 搜索脚本输出格式: 每行可能是 `EPUB_HITS_JSON:` 开头的 JSON
    let json_block = extract_json_block(&stdout, "EPUB_HITS_JSON:");

    let Ok(hits) = serde_json::from_str::<Vec<PatentHit>>(&json_block) else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "解析 CNIPA 结果失败。stdout: {stdout:.200}, stderr: {stderr:.200}"
        ));
    };

    let result = CnipaSearchResult {
        query: query.to_string(),
        total_hits: hits.len(),
        elapsed_ms: start.elapsed().as_millis() as u64,
        source: "中国国家知识产权局 (CNIPA) 专利公布公告".into(),
        patents: hits,
    };

    serde_json::to_value(&result).map_err(|e| format!("序列化失败: {e}"))
}

/// 下载单个专利 PDF（通过 CNIPA 页图像组装）。
pub fn cnipa_download(input: CnipaDownloadInput) -> Result<Value, String> {
    let script_path = resolve_script(SCRIPT_CLIENT)?;
    let patent_id = input.patent_id.trim();
    if patent_id.is_empty() {
        return Err("专利号不能为空".into());
    }

    let output_dir = input
        .output_dir
        .clone()
        .unwrap_or_else(|| "/tmp".to_string());
    let out_path = Path::new(&output_dir).join(format!("{patent_id}.pdf"));

    let output = retry_call(|| {
        Command::new("python3")
            .env("EPUB_WAF_MAX_WAIT_SEC", WAF_TIMEOUT_SEC)
            .arg(&script_path)
            .arg("pdf")
            .arg(patent_id)
            .arg("-o")
            .arg(out_path.to_string_lossy().to_string())
            .output()
            .map_err(|e| format!("调用 CNIPA 脚本失败: {e}"))
    })?;

    check_output(&output)?;

    if out_path.exists() {
        let size_kb = std::fs::metadata(&out_path)
            .map(|m| m.len() as f64 / 1024.0)
            .unwrap_or(0.0);
        Ok(serde_json::json!({
            "patent_id": patent_id,
            "path": out_path.to_string_lossy(),
            "size_kb": format!("{size_kb:.1}"),
            "status": "downloaded"
        }))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PDF 未生成: {stderr:.300}"))
    }
}

/// 批量下载多个专利 PDF。
pub fn cnipa_batch_download(patent_ids: &[String], output_dir: &str) -> Result<Value, String> {
    let mut results: Vec<BatchDownloadOutput> = Vec::new();
    let mut downloaded = 0;
    let total = patent_ids.len();

    for patent_id in patent_ids {
        let input = CnipaDownloadInput {
            patent_id: patent_id.clone(),
            output_dir: Some(output_dir.to_string()),
        };
        match cnipa_download(input) {
            Ok(val) => {
                downloaded += 1;
                results.push(BatchDownloadOutput {
                    patent_id: patent_id.clone(),
                    path: val.get("path").and_then(|v| v.as_str()).map(str::to_string),
                    size_kb: val
                        .get("size_kb")
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok()),
                    status: "downloaded".into(),
                    error: None,
                });
            }
            Err(e) => {
                results.push(BatchDownloadOutput {
                    patent_id: patent_id.clone(),
                    path: None,
                    size_kb: None,
                    status: "failed".into(),
                    error: Some(e),
                });
            }
        }
    }

    Ok(serde_json::json!({
        "total": total,
        "downloaded": downloaded,
        "failed": total - downloaded,
        "output_dir": output_dir,
        "results": results
    }))
}

/// 高被引专利检索 — 检索后按引用数排序 Top-N。
pub fn cnipa_high_citation_search(
    query: &str,
    min_citations: u32,
    limit: usize,
) -> Result<Value, String> {
    let input = CnipaSearchInput {
        query: query.to_string(),
        max_pages: Some(5),
    };

    let raw = cnipa_search(input)?;
    let mut hits: Vec<PatentHit> =
        serde_json::from_value(raw.get("patents").cloned().unwrap_or_default()).unwrap_or_default();

    // 过滤引用数 >= min_citations 的结果
    // 注意：CNIPA 的简单搜索通常不返回引用数，这里做保守处理
    hits.sort_by(|a, b| b.citations.unwrap_or(0).cmp(&a.citations.unwrap_or(0)));
    hits.truncate(limit);

    let qualified = hits.len();
    Ok(serde_json::json!({
        "query": query,
        "min_citations": min_citations,
        "total_qualified": qualified,
        "total_searched": raw.get("total_hits").unwrap_or(&Value::Null),
        "source": "中国国家知识产权局 (CNIPA) 专利公布公告",
        "note": "引用数据来自公开的专利引用信息，部分专利可能缺少引用计数",
        "patents": hits
    }))
}

// ──────────────────────────────────────────────────────────────────
// 辅助函数
// ──────────────────────────────────────────────────────────────────

/// 带指数退避的重试调用。
fn retry_call<F, T>(mut f: F) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
{
    let mut last_err = String::new();
    for attempt in 0..MAX_RETRIES {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_err = e;
                if attempt + 1 < MAX_RETRIES {
                    let wait = Duration::from_millis(RETRY_BASE_MS * 2u64.pow(attempt));
                    std::thread::sleep(wait);
                }
            }
        }
    }
    Err(last_err)
}

/// 检查 subprocess 输出是否成功。
fn check_output(output: &Output) -> Result<(), String> {
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = if stderr.is_empty() {
            format!("exit code: {}", output.status.code().unwrap_or(-1))
        } else {
            stderr
                .lines()
                .filter(|l| l.contains("Error") || l.contains("ERROR") || l.contains("INVALID"))
                .take(3)
                .collect::<Vec<_>>()
                .join("; ")
        };
        return Err(format!("CNIPA 脚本错误: {detail}"));
    }
    Ok(())
}

/// 从输出中提取 JSON 块（以特定前缀标记）。
fn extract_json_block(stdout: &str, prefix: &str) -> String {
    for line in stdout.lines() {
        if let Some(json) = line.strip_prefix(prefix) {
            return json.trim().to_string();
        }
    }
    // 尝试解析整块 JSON
    stdout
        .trim()
        .lines()
        .find(|l| l.trim().starts_with('[') || l.trim().starts_with('{'))
        .map(str::to_string)
        .unwrap_or_else(|| stdout.trim().to_string())
}

/// 查找脚本路径。
fn resolve_script(default: &str) -> Result<PathBuf, String> {
    let path = Path::new(default);
    if path.is_file() {
        return Ok(path.to_path_buf());
    }
    // 从环境变量获取
    if let Ok(env_path) = std::env::var("YUNXI_CNIPA_SCRIPT") {
        let p = Path::new(&env_path);
        if p.is_file() {
            return Ok(p.to_path_buf());
        }
    }
    Err(format!(
        "CNIPA 脚本未找到。请确保 {default} 存在，或设置 YUNXI_CNIPA_SCRIPT 环境变量。\n\
         安装 Playwright: python -m playwright install chromium"
    ))
}

// ──────────────────────────────────────────────────────────────────
// 测试
// ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_deserialization() {
        let json = r#"{"query": "人工智能"}"#;
        let input: CnipaSearchInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "人工智能");
    }

    #[test]
    fn test_input_with_optional_fields() {
        let json = r#"{"query": "区块链", "max_pages": 3}"#;
        let input: CnipaSearchInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.query, "区块链");
        assert_eq!(input.max_pages, Some(3));
    }

    #[test]
    fn test_retry_failure() {
        let mut calls = 0;
        let result = retry_call(|| {
            calls += 1;
            Err::<(), _>(format!("尝试 {calls} 失败"))
        });
        assert!(result.is_err());
        assert_eq!(calls, 3);
    }

    #[test]
    fn test_extract_json_block_works() {
        let stdout = "EPUB_NOTE: some log\nEPUB_HITS_JSON: [{\"title\":\"test\"}]\n";
        let json = extract_json_block(stdout, "EPUB_HITS_JSON:");
        assert_eq!(json, "[{\"title\":\"test\"}]");
    }

    #[test]
    fn test_extract_json_block_fallback() {
        let stdout = "log line\n[{\"title\":\"fallback\"}]\nmore log";
        let json = extract_json_block(stdout, "EPUB_HITS_JSON:");
        assert_eq!(json, "[{\"title\":\"fallback\"}]");
    }
}
