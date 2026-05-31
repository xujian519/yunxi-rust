//! 语义嵌入配置（读取 `.yunxi/settings*.json` 与环境变量）

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// 顶层 `semantic` 配置块
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticConfig {
    /// 是否启用语义能力（嵌入检索、语义对比等）；默认关闭
    #[serde(default)]
    pub enabled: bool,
    /// `http` | `onnx`；默认 `http`
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default)]
    pub http: HttpEmbeddingConfig,
    #[serde(default)]
    pub onnx: OnnxEmbeddingConfig,
    #[serde(default)]
    pub defaults: SemanticDefaults,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpEmbeddingConfig {
    #[serde(default = "default_http_base")]
    pub base_url: String,
    #[serde(default = "default_embed_path")]
    pub embed_path: String,
    /// `openai` | `tei` | `simple`
    #[serde(default = "default_api_style")]
    pub api_style: String,
    #[serde(default)]
    pub model: Option<String>,
    /// oMLX 等服务的 Bearer Token（也可设环境变量 EMBEDDING_API_KEY / OMLX_API_KEY）
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnnxEmbeddingConfig {
    /// 模型目录（含 model.onnx、tokenizer.json）；空则使用默认候选路径
    pub model_dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticDefaults {
    /// 知识检索默认模式：`text` | `semantic` | `hybrid`
    #[serde(default = "default_knowledge_mode")]
    pub knowledge_search_mode: String,
    /// `SemanticCompare` 在 `auto` 模式下是否优先使用嵌入
    #[serde(default)]
    pub semantic_compare_auto: bool,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: default_backend(),
            http: HttpEmbeddingConfig::default(),
            onnx: OnnxEmbeddingConfig::default(),
            defaults: SemanticDefaults::default(),
        }
    }
}

impl Default for HttpEmbeddingConfig {
    fn default() -> Self {
        Self {
            base_url: default_http_base(),
            embed_path: default_embed_path(),
            api_style: default_api_style(),
            model: Some("bge-m3-mlx-8bit".into()),
            api_key: None,
            timeout_secs: default_timeout_secs(),
        }
    }
}

impl Default for OnnxEmbeddingConfig {
    fn default() -> Self {
        Self { model_dir: None }
    }
}

impl Default for SemanticDefaults {
    fn default() -> Self {
        Self {
            knowledge_search_mode: default_knowledge_mode(),
            semantic_compare_auto: true,
        }
    }
}

fn default_backend() -> String {
    "http".into()
}

fn default_http_base() -> String {
    // 与 ~/.omlx/settings.json 及 yunpat 索引默认一致（oMLX 嵌入，非 8766 FlagEmbedding）
    "http://127.0.0.1:8009".into()
}

fn default_embed_path() -> String {
    "/v1/embeddings".into()
}

fn default_api_style() -> String {
    "openai".into()
}

fn default_timeout_secs() -> u64 {
    120
}

fn default_knowledge_mode() -> String {
    "hybrid".into()
}

/// 多模态 / 图片 OCR（oMLX :8009；默认模型 `gemma-4-e2b-it-4bit`，API Key 与 `semantic.http` 共用）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub http: VisionHttpConfig,
    #[serde(default)]
    pub defaults: VisionDefaults,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionHttpConfig {
    pub base_url: Option<String>,
    pub chat_path: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    #[serde(default = "default_vision_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_vision_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisionDefaults {
    #[serde(default = "default_vision_ocr_prompt")]
    pub ocr_prompt: String,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            http: VisionHttpConfig::default(),
            defaults: VisionDefaults::default(),
        }
    }
}

/// oMLX 多模态默认模型（与 `/health` 的 `default_model` 一致）。
#[must_use]
pub fn default_vision_model() -> String {
    "gemma-4-e2b-it-4bit".into()
}

impl Default for VisionHttpConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            chat_path: None,
            model: Some(default_vision_model()),
            api_key: None,
            timeout_secs: default_vision_timeout_secs(),
            max_tokens: default_vision_max_tokens(),
        }
    }
}

impl Default for VisionDefaults {
    fn default() -> Self {
        Self {
            ocr_prompt: default_vision_ocr_prompt(),
        }
    }
}

fn default_vision_timeout_secs() -> u64 {
    180
}

fn default_vision_max_tokens() -> u32 {
    8192
}

fn default_vision_chat_path() -> String {
    "/v1/chat/completions".into()
}

fn default_vision_ocr_prompt() -> String {
    "你是专利案件材料 OCR 助手。请完整、逐行转录图片中的全部可见文字（中文与英文、数字、标点），保留段落换行；不要总结、不要省略、不要添加解释。若无法辨认处用 [无法辨认] 标注。".into()
}

/// 解析后的 Vision HTTP 参数（供 `vision::ocr_image_from_path` 使用）。
#[derive(Debug, Clone)]
pub struct ResolvedVisionHttp {
    pub base_url: String,
    pub chat_path: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub max_tokens: u32,
    pub ocr_prompt: String,
}

impl ResolvedVisionHttp {
    /// 合并 `vision` 与 `semantic` 配置及环境变量。
    ///
    /// # Errors
    ///
    /// 合并 `vision`、`semantic` 与环境变量；`api_key` 优先取自 `semantic.http`（与 BGE-M3 一致）。
    pub fn resolve() -> Result<Self, String> {
        let vision = load_vision_config();
        let semantic = load_semantic_config();

        let base_url = vision
            .http
            .base_url
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| semantic.http.base_url.clone());

        let chat_path = vision
            .http
            .chat_path
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(default_vision_chat_path);

        let api_key = resolve_api_key(&vision, &semantic);

        let model = env::var("YUNXI_VISION_MODEL")
            .ok()
            .filter(|m| !m.is_empty())
            .or_else(|| vision.http.model.clone())
            .unwrap_or_else(default_vision_model);

        let timeout_secs = if vision.http.timeout_secs != default_vision_timeout_secs() {
            vision.http.timeout_secs
        } else {
            semantic
                .http
                .timeout_secs
                .max(default_vision_timeout_secs())
        };

        Ok(Self {
            base_url,
            chat_path,
            model,
            api_key,
            timeout_secs,
            max_tokens: vision.http.max_tokens,
            ocr_prompt: vision.defaults.ocr_prompt,
        })
    }
}

fn resolve_api_key(vision: &VisionConfig, semantic: &SemanticConfig) -> Option<String> {
    // 与 BGE-M3 嵌入共用同一 oMLX 密钥；`vision.http.apiKey` 可省略
    semantic
        .http
        .api_key
        .clone()
        .filter(|k| !k.is_empty())
        .or_else(|| vision.http.api_key.clone().filter(|k| !k.is_empty()))
        .or_else(|| env::var("OMLX_API_KEY").ok().filter(|k| !k.is_empty()))
        .or_else(|| env::var("EMBEDDING_API_KEY").ok().filter(|k| !k.is_empty()))
}

/// 加载合并后的多模态 OCR 配置。
#[must_use]
pub fn load_vision_config() -> VisionConfig {
    let mut cfg = discover_and_merge_vision_json();
    apply_vision_env_overrides(&mut cfg);
    cfg
}

fn discover_and_merge_vision_json() -> VisionConfig {
    let mut merged = VisionConfig::default();
    for path in settings_json_paths() {
        if let Some(partial) = read_vision_from_file(&path) {
            merge_vision(&mut merged, partial);
        }
    }
    merged
}

fn read_vision_from_file(path: &Path) -> Option<VisionConfig> {
    let text = fs::read_to_string(path).ok()?;
    let root: serde_json::Value = serde_json::from_str(&text).ok()?;
    let vision = root.get("vision")?;
    serde_json::from_value(vision.clone()).ok()
}

fn merge_vision(base: &mut VisionConfig, overlay: VisionConfig) {
    if overlay.enabled {
        base.enabled = true;
    }
    if let Some(url) = overlay.http.base_url.filter(|s| !s.is_empty()) {
        base.http.base_url = Some(url);
    }
    if let Some(path) = overlay.http.chat_path.filter(|s| !s.is_empty()) {
        base.http.chat_path = Some(path);
    }
    if overlay.http.model.is_some() {
        base.http.model = overlay.http.model;
    }
    if overlay.http.api_key.is_some() {
        base.http.api_key = overlay.http.api_key;
    }
    if overlay.http.timeout_secs != default_vision_timeout_secs() {
        base.http.timeout_secs = overlay.http.timeout_secs;
    }
    if overlay.http.max_tokens != default_vision_max_tokens() {
        base.http.max_tokens = overlay.http.max_tokens;
    }
    if overlay.defaults.ocr_prompt != default_vision_ocr_prompt() {
        base.defaults.ocr_prompt = overlay.defaults.ocr_prompt;
    }
}

/// 探测 oMLX `/health` 的 `default_model` 字段。
#[must_use]
pub fn fetch_omlx_default_model(base_url: &str) -> Option<String> {
    let url = format!("{}/health", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;
    let payload: serde_json::Value = client.get(&url).send().ok()?.json().ok()?;
    payload
        .get("default_model")
        .and_then(|m| m.as_str())
        .map(str::to_string)
}

/// 异步版本：探测 oMLX `/health` 的 `default_model` 字段。
pub async fn fetch_omlx_default_model_async(base_url: &str) -> Option<String> {
    let url = format!("{}/health", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;
    let payload: serde_json::Value = client.get(&url).send().await.ok()?.json().await.ok()?;
    payload
        .get("default_model")
        .and_then(|m| m.as_str())
        .map(str::to_string)
}

fn apply_vision_env_overrides(cfg: &mut VisionConfig) {
    if env_truthy("YUNXI_VISION_ENABLED") {
        cfg.enabled = true;
    }
    if let Ok(url) = env::var("YUNXI_VISION_URL") {
        if !url.is_empty() {
            cfg.enabled = true;
            cfg.http.base_url = Some(url);
        }
    }
    if let Ok(model) = env::var("YUNXI_VISION_MODEL") {
        if !model.is_empty() {
            cfg.http.model = Some(model);
        }
    }
}

fn settings_json_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".yunxi/settings.json"));
        paths.push(home.join(".yunxi/settings.local.json"));
    }
    if let Ok(cwd) = env::current_dir() {
        paths.push(cwd.join(".yunxi/settings.json"));
        paths.push(cwd.join(".yunxi/settings.local.json"));
    }
    paths
}

/// 加载合并后的语义配置（用户 → 项目 → 本地 → 环境变量覆盖）
#[must_use]
pub fn load_semantic_config() -> SemanticConfig {
    let mut cfg = discover_and_merge_semantic_json();
    apply_env_overrides(&mut cfg);
    cfg
}

/// 语义能力是否可用（用户显式开启，或本地 oMLX 服务自动检测到）
#[must_use]
pub fn semantic_enabled() -> bool {
    // 环境变量可强制关闭（优先于配置文件）
    if let Ok(v) = std::env::var("YUNXI_SEMANTIC_ENABLED") {
        let v = v.trim();
        if v.eq_ignore_ascii_case("0") || v.eq_ignore_ascii_case("false") {
            return false;
        }
    }
    let cfg = load_semantic_config();
    if cfg.enabled {
        return true;
    }
    if env_truthy("YUNXI_SEMANTIC_AUTODETECT_DISABLED") {
        return false;
    }
    detect_local_embedding_service()
}

/// 探测本地 oMLX 嵌入服务是否可达
fn detect_local_embedding_service() -> bool {
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    static CACHE: OnceLock<(bool, Instant)> = OnceLock::new();
    let ttl = Duration::from_secs(300);

    if let Some((result, ts)) = CACHE.get() {
        if ts.elapsed() < ttl {
            return *result;
        }
    }

    let port = parse_port_from_url(&default_http_base()).unwrap_or(8009);
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .unwrap_or_else(|_| "127.0.0.1:8009".parse().unwrap());
    let result = std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok();
    let _ = CACHE.set((result, Instant::now()));
    result
}

fn parse_port_from_url(url: &str) -> Option<u16> {
    let host_port = url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/');
    host_port
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse::<u16>().ok())
}

fn discover_and_merge_semantic_json() -> SemanticConfig {
    let mut merged = SemanticConfig::default();

    for path in settings_json_paths() {
        if let Some(partial) = read_semantic_from_file(&path) {
            merge_semantic(&mut merged, partial);
        }
    }

    merged
}

fn read_semantic_from_file(path: &Path) -> Option<SemanticConfig> {
    let text = fs::read_to_string(path).ok()?;
    let root: serde_json::Value = serde_json::from_str(&text).ok()?;
    let semantic = root.get("semantic")?;
    serde_json::from_value(semantic.clone()).ok()
}

fn merge_semantic(base: &mut SemanticConfig, overlay: SemanticConfig) {
    if overlay.enabled {
        base.enabled = true;
    }
    if !overlay.backend.is_empty() {
        base.backend = overlay.backend;
    }
    merge_http(&mut base.http, overlay.http);
    if overlay.onnx.model_dir.is_some() {
        base.onnx = overlay.onnx;
    }
    merge_defaults(&mut base.defaults, overlay.defaults);
}

fn merge_http(base: &mut HttpEmbeddingConfig, overlay: HttpEmbeddingConfig) {
    if overlay.base_url != default_http_base() {
        base.base_url = overlay.base_url;
    }
    if overlay.embed_path != default_embed_path() {
        base.embed_path = overlay.embed_path;
    }
    if overlay.api_style != default_api_style() {
        base.api_style = overlay.api_style;
    }
    if overlay.model.is_some() {
        base.model = overlay.model;
    }
    if overlay.api_key.is_some() {
        base.api_key = overlay.api_key;
    }
    if overlay.timeout_secs != default_timeout_secs() {
        base.timeout_secs = overlay.timeout_secs;
    }
}

fn merge_defaults(base: &mut SemanticDefaults, overlay: SemanticDefaults) {
    if overlay.knowledge_search_mode != default_knowledge_mode() {
        base.knowledge_search_mode = overlay.knowledge_search_mode;
    }
    base.semantic_compare_auto = overlay.semantic_compare_auto;
}

fn apply_env_overrides(cfg: &mut SemanticConfig) {
    if env_truthy("YUNXI_SEMANTIC_ENABLED") {
        cfg.enabled = true;
    }
    if let Ok(url) = env::var("YUNXI_EMBEDDING_URL").or_else(|_| env::var("EMBEDDING_BASE_URL")) {
        if !url.is_empty() {
            cfg.enabled = true;
            cfg.backend = "http".into();
            // 支持完整 URL（…/v1/embeddings）或仅 host:port
            if url.contains("/v1/") {
                if let Some((base, path)) = url.split_once("/v1/") {
                    cfg.http.base_url = base.to_string();
                    cfg.http.embed_path = format!("/v1/{path}");
                }
            } else {
                cfg.http.base_url = url;
            }
        }
    }
    if let Ok(model) = env::var("EMBEDDING_MODEL") {
        if !model.is_empty() {
            cfg.http.model = Some(model);
        }
    }
    if let Ok(key) = env::var("EMBEDDING_API_KEY").or_else(|_| env::var("OMLX_API_KEY")) {
        if !key.is_empty() {
            cfg.http.api_key = Some(key);
        }
    }
    if let Ok(style) = env::var("YUNXI_EMBEDDING_API_STYLE") {
        if !style.is_empty() {
            cfg.http.api_style = style;
        }
    }
}

fn env_truthy(key: &str) -> bool {
    match env::var(key) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

impl SemanticConfig {
    #[must_use]
    pub fn onnx_model_dir(&self) -> PathBuf {
        self.onnx
            .model_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("assets/models/bge-m3"))
    }

    #[must_use]
    pub fn parse_knowledge_search_mode(&self) -> KnowledgeSearchMode {
        KnowledgeSearchMode::parse(&self.defaults.knowledge_search_mode)
    }
}

/// 知识检索模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeSearchMode {
    Text,
    Semantic,
    Hybrid,
}

impl KnowledgeSearchMode {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "semantic" | "vector" => Self::Semantic,
            "hybrid" | "mixed" => Self::Hybrid,
            _ => Self::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_disabled() {
        let cfg = SemanticConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.http.base_url, "http://127.0.0.1:8009");
        assert_eq!(cfg.http.model.as_deref(), Some("bge-m3-mlx-8bit"));
    }

    #[test]
    fn parses_knowledge_mode() {
        assert_eq!(
            KnowledgeSearchMode::parse("hybrid"),
            KnowledgeSearchMode::Hybrid
        );
    }

    #[test]
    fn vision_defaults_to_omlx_gemma_model() {
        assert_eq!(default_vision_model(), "gemma-4-e2b-it-4bit");
        let resolved = ResolvedVisionHttp::resolve().expect("resolve");
        assert_eq!(resolved.model, "gemma-4-e2b-it-4bit");
        assert_eq!(resolved.base_url, "http://127.0.0.1:8009");
    }
}
