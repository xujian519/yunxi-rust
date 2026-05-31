//! BGE-M3 ONNX 嵌入服务
//!
//! 基于 Athena `core/embedding/bge_embedding_service.py` 重写。
//! 使用 ONNX Runtime 加载 BGE-M3 模型，输出 1024 维 float32 向量。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use indexmap::IndexMap;
use ort::session::Session;
use ort::value::{DynTensorValueType, Tensor};
use tokenizers::Tokenizer;

use crate::config::{load_semantic_config, SemanticConfig};
use crate::http_backend::HttpEmbeddingBackend;
use crate::vector_store::VectorStoreError;

#[derive(Clone)]
enum EmbeddingBackend {
    Onnx(Arc<OnnxBackend>),
    Http(Arc<HttpEmbeddingBackend>),
}

struct OnnxBackend {
    session: Mutex<Session>,
    tokenizer: Mutex<Tokenizer>,
}

/// 嵌入向量维度（BGE-M3 输出 1024 维）
pub const EMBEDDING_DIM: usize = 1024;

/// BGE-M3 最大序列长度
const BGE_M3_MAX_SEQ_LEN: usize = 8192;

/// 默认 LRU 缓存容量
const DEFAULT_CACHE_SIZE: usize = 2048;

/// 嵌入向量类型
pub type Embedding = Vec<f32>;

/// 5 级相似度分类（对应 Athena 的 SimilarityLevel）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SimilarityLevel {
    /// >= 0.95 — 完全相同/等同
    Identical,
    /// >= 0.80 — 高度相似
    HighlySimilar,
    /// >= 0.60 — 中度相似
    ModeratelySimilar,
    /// >= 0.40 — 轻微相似
    SlightlySimilar,
    /// < 0.40 — 不相关
    Unrelated,
}

impl SimilarityLevel {
    pub fn from_score(score: f32) -> Self {
        if score >= 0.95 {
            Self::Identical
        } else if score >= 0.80 {
            Self::HighlySimilar
        } else if score >= 0.60 {
            Self::ModeratelySimilar
        } else if score >= 0.40 {
            Self::SlightlySimilar
        } else {
            Self::Unrelated
        }
    }
}

/// 统一嵌入服务（ONNX 本地或 HTTP 远程）
///
/// 线程安全：后端与 LRU 缓存均可在多线程下使用。
#[derive(Clone)]
pub struct EmbeddingService {
    backend: EmbeddingBackend,
    cache: Arc<Mutex<OrderedLruCache>>,
}

/// 基于 `IndexMap` 的 O(1) LRU 缓存
struct OrderedLruCache {
    entries: IndexMap<String, Embedding>,
    max_size: usize,
}

impl OrderedLruCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: IndexMap::new(),
            max_size,
        }
    }

    fn get(&mut self, key: &str) -> Option<&Embedding> {
        // 移到末尾（最近使用），IndexMap::swap_remove_index + insert 实现 LRU
        if let Some((k, v)) = self.entries.shift_remove_entry(key) {
            self.entries.insert(k, v);
            // insert 后 key 在末尾，返回引用
            self.entries.get(key)
        } else {
            None
        }
    }

    fn put(&mut self, key: String, value: Embedding) {
        // 已存在则先移除（保持插入顺序）
        self.entries.shift_remove(&key);

        // 淘汰最久未使用（队首）
        if self.entries.len() >= self.max_size {
            self.entries.shift_remove_index(0);
        }

        self.entries.insert(key, value);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl EmbeddingService {
    /// 按 `.yunxi/settings.json` 的 `semantic` 块加载（需 `enabled: true`）
    pub fn load_from_config() -> Result<Self, EmbeddingError> {
        let cfg = load_semantic_config();
        if !cfg.enabled {
            return Err(EmbeddingError::Disabled("semantic.enabled 为 false".into()));
        }
        Self::load_with_config(&cfg)
    }

    fn load_with_config(cfg: &SemanticConfig) -> Result<Self, EmbeddingError> {
        let cache = Arc::new(Mutex::new(OrderedLruCache::new(DEFAULT_CACHE_SIZE)));
        let backend = match cfg.backend.trim().to_ascii_lowercase().as_str() {
            "onnx" | "local" => {
                EmbeddingBackend::Onnx(Arc::new(load_onnx_backend(&cfg.onnx_model_dir())?))
            }
            _ => EmbeddingBackend::Http(Arc::new(HttpEmbeddingBackend::new(&cfg.http)?)),
        };
        Ok(Self { backend, cache })
    }

    /// 从指定模型目录加载 ONNX 服务（不检查 semantic.enabled）
    pub fn load_onnx(model_dir: &Path) -> Result<Self, EmbeddingError> {
        Ok(Self {
            backend: EmbeddingBackend::Onnx(Arc::new(load_onnx_backend(model_dir)?)),
            cache: Arc::new(Mutex::new(OrderedLruCache::new(DEFAULT_CACHE_SIZE))),
        })
    }

    /// 兼容旧调用：若已启用语义配置则走配置；否则尝试本地 ONNX
    pub fn load_default() -> Result<Self, EmbeddingError> {
        let cfg = load_semantic_config();
        if cfg.enabled {
            return Self::load_from_config();
        }
        Self::load_onnx_from_default_paths()
    }

    fn load_onnx_from_default_paths() -> Result<Self, EmbeddingError> {
        let candidates = [
            PathBuf::from("assets/models/bge-m3"),
            dirs::home_dir()
                .map(|h| h.join(".yunxi/models/bge-m3"))
                .unwrap_or_default(),
        ];

        for dir in &candidates {
            if dir.join("model.onnx").exists() {
                return Self::load_onnx(dir);
            }
        }

        Err(EmbeddingError::ModelNotFound(
            "assets/models/bge-m3/ or ~/.yunxi/models/bge-m3/".into(),
        ))
    }

    /// 编码单个文本为 1024 维向量
    pub fn encode(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(text) {
                return Ok(cached.clone());
            }
        }

        let embedding = match &self.backend {
            EmbeddingBackend::Onnx(onnx) => onnx.encode_single(text)?,
            EmbeddingBackend::Http(http) => http.encode(text)?,
        };

        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    /// 批量编码
    pub fn encode_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();

        {
            let mut cache = self.cache.lock().unwrap();
            for (i, text) in texts.iter().enumerate() {
                if let Some(cached) = cache.get(text) {
                    results.push((i, cached.clone()));
                } else {
                    uncached_indices.push(i);
                    uncached_texts.push(*text);
                }
            }
        }

        if !uncached_texts.is_empty() {
            tracing::debug!(count = uncached_texts.len(), "encoding uncached texts");
            let new_embeddings = match &self.backend {
                EmbeddingBackend::Onnx(onnx) => onnx.encode_batch_impl(&uncached_texts)?,
                EmbeddingBackend::Http(http) => http.encode_batch(&uncached_texts)?,
            };
            let mut cache = self.cache.lock().unwrap();
            for (i, embedding) in new_embeddings.into_iter().enumerate() {
                let original_idx = uncached_indices[i];
                cache.put(uncached_texts[i].to_string(), embedding.clone());
                results.push((original_idx, embedding));
            }
        }

        results.sort_by_key(|(i, _)| *i);
        Ok(results.into_iter().map(|(_, e)| e).collect())
    }

    /// 异步编码单个文本（HTTP backend 使用 spawn_blocking，避免阻塞 tokio 运行时）
    pub async fn encode_async(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(text) {
                return Ok(cached.clone());
            }
        }

        let embedding = match &self.backend {
            EmbeddingBackend::Onnx(onnx) => onnx.encode_single(text)?,
            EmbeddingBackend::Http(http) => http.encode_async(text).await?,
        };

        {
            let mut cache = self.cache.lock().unwrap();
            cache.put(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    /// 异步批量编码（HTTP backend 使用 spawn_blocking，避免阻塞 tokio 运行时）
    pub async fn encode_batch_async(
        &self,
        texts: &[&str],
    ) -> Result<Vec<Embedding>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());
        let mut uncached_indices = Vec::new();
        let mut uncached_texts = Vec::new();

        {
            let mut cache = self.cache.lock().unwrap();
            for (i, text) in texts.iter().enumerate() {
                if let Some(cached) = cache.get(text) {
                    results.push((i, cached.clone()));
                } else {
                    uncached_indices.push(i);
                    uncached_texts.push(*text);
                }
            }
        }

        if !uncached_texts.is_empty() {
            tracing::debug!(count = uncached_texts.len(), "encoding uncached texts");
            let new_embeddings = match &self.backend {
                EmbeddingBackend::Onnx(onnx) => onnx.encode_batch_impl(&uncached_texts)?,
                EmbeddingBackend::Http(http) => http.encode_batch_async(&uncached_texts).await?,
            };
            let mut cache = self.cache.lock().unwrap();
            for (i, embedding) in new_embeddings.into_iter().enumerate() {
                let original_idx = uncached_indices[i];
                cache.put(uncached_texts[i].to_string(), embedding.clone());
                results.push((original_idx, embedding));
            }
        }

        results.sort_by_key(|(i, _)| *i);
        Ok(results.into_iter().map(|(_, e)| e).collect())
    }

    /// 计算余弦相似度
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a * norm_b)
    }
}

fn load_onnx_backend(model_dir: &Path) -> Result<OnnxBackend, EmbeddingError> {
    let model_path = model_dir.join("model.onnx");
    let tokenizer_path = model_dir.join("tokenizer.json");

    if !model_path.exists() {
        return Err(EmbeddingError::ModelNotFound(model_path));
    }
    if !tokenizer_path.exists() {
        return Err(EmbeddingError::TokenizerNotFound(tokenizer_path));
    }

    tracing::info!(path = %model_path.display(), "loading ONNX embedding model");

    let session = Session::builder()
        .and_then(|mut b| b.commit_from_file(&model_path))
        .map_err(|e| EmbeddingError::OrtLoad(e.to_string()))?;

    let tokenizer = Tokenizer::from_file(&tokenizer_path)
        .map_err(|e| EmbeddingError::TokenizerLoad(e.to_string()))?;

    tracing::info!("ONNX embedding model loaded");

    Ok(OnnxBackend {
        session: Mutex::new(session),
        tokenizer: Mutex::new(tokenizer),
    })
}

impl OnnxBackend {
    fn encode_single(&self, text: &str) -> Result<Embedding, EmbeddingError> {
        self.encode_batch_impl(&[text])?
            .into_iter()
            .next()
            .ok_or_else(|| EmbeddingError::Inference("empty onnx batch".into()))
    }

    /// 批量编码核心实现（ONNX 推理）
    fn encode_batch_impl(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError> {
        let encodings: Vec<_> = {
            let tokenizer = self.tokenizer.lock().unwrap();
            texts
                .iter()
                .map(|text| {
                    tokenizer
                        .encode(*text, true)
                        .map_err(|e| EmbeddingError::Tokenize(e.to_string()))
                })
                .collect::<Result<_, EmbeddingError>>()?
        };

        let batch_size = texts.len();
        let seq_len = encodings
            .iter()
            .map(|e| e.len())
            .max()
            .unwrap_or(1)
            .min(BGE_M3_MAX_SEQ_LEN);

        // 构建输入张量（i64 类型）
        let mut input_ids = vec![0i64; batch_size * seq_len];
        let mut attention_mask = vec![0i64; batch_size * seq_len];
        let token_type_ids = vec![0i64; batch_size * seq_len];

        for (i, encoding) in encodings.iter().enumerate() {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();
            for (j, (&id, &m)) in ids.iter().zip(mask.iter()).enumerate() {
                if j < seq_len {
                    input_ids[i * seq_len + j] = i64::from(id);
                    attention_mask[i * seq_len + j] = i64::from(m);
                }
            }
        }

        // 创建 ONNX tensor
        let input_ids_tensor = Tensor::from_array((
            vec![batch_size as i64, seq_len as i64],
            input_ids.into_boxed_slice(),
        ))
        .map_err(|e| EmbeddingError::Inference(e.to_string()))?;

        let attention_mask_tensor = Tensor::from_array((
            vec![batch_size as i64, seq_len as i64],
            attention_mask.into_boxed_slice(),
        ))
        .map_err(|e| EmbeddingError::Inference(e.to_string()))?;

        let token_type_ids_tensor = Tensor::from_array((
            vec![batch_size as i64, seq_len as i64],
            token_type_ids.into_boxed_slice(),
        ))
        .map_err(|e| EmbeddingError::Inference(e.to_string()))?;

        let inputs = ort::inputs! {
            "input_ids" => input_ids_tensor,
            "attention_mask" => attention_mask_tensor,
            "token_type_ids" => token_type_ids_tensor,
        };

        // ONNX 推理
        let mut session = self.session.lock().unwrap();
        let outputs = session
            .run(inputs)
            .map_err(|e| EmbeddingError::Inference(e.to_string()))?;

        // 提取输出（BGE-M3 输出 last_hidden_state: [batch, seq_len, hidden_dim]）
        let output_ref = outputs[0]
            .view()
            .downcast::<DynTensorValueType>()
            .map_err(|e| EmbeddingError::Output(e.to_string()))?;

        let (shape, data) = output_ref
            .try_extract_tensor::<f32>()
            .map_err(|e| EmbeddingError::Output(e.to_string()))?;

        let hidden_dim = shape.last().copied().unwrap_or(EMBEDDING_DIM as i64) as usize;

        // CLS pooling：取 [CLS] token（位置 0）的输出作为句子嵌入
        let mut result = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            let offset = i * seq_len * hidden_dim;
            let mut embedding = data[offset..offset + EMBEDDING_DIM.min(hidden_dim)].to_vec();
            embedding.resize(EMBEDDING_DIM, 0.0);

            // L2 归一化
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                for v in &mut embedding {
                    *v /= norm;
                }
            }
            result.push(embedding);
        }

        Ok(result)
    }
}

/// 嵌入服务错误类型
#[derive(Debug, thiserror::Error)]
pub enum EmbeddingError {
    #[error("semantic embedding disabled: {0}")]
    Disabled(String),
    #[error("HTTP embedding error: {0}")]
    Http(String),
    #[error("model not found: {0}")]
    ModelNotFound(PathBuf),
    #[error("tokenizer not found: {0}")]
    TokenizerNotFound(PathBuf),
    #[error("ONNX Runtime load error: {0}")]
    OrtLoad(String),
    #[error("tokenizer load error: {0}")]
    TokenizerLoad(String),
    #[error("tokenization error: {0}")]
    Tokenize(String),
    #[error("tensor error: {0}")]
    Tensor(String),
    #[error("inference error: {0}")]
    Inference(String),
    #[error("output error: {0}")]
    Output(String),
    #[error("vector store error: {0}")]
    Store(#[from] VectorStoreError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let sim = EmbeddingService::cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = EmbeddingService::cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        let sim = EmbeddingService::cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_similarity_level() {
        assert_eq!(
            SimilarityLevel::from_score(0.97),
            SimilarityLevel::Identical
        );
        assert_eq!(
            SimilarityLevel::from_score(0.85),
            SimilarityLevel::HighlySimilar
        );
        assert_eq!(
            SimilarityLevel::from_score(0.65),
            SimilarityLevel::ModeratelySimilar
        );
        assert_eq!(
            SimilarityLevel::from_score(0.45),
            SimilarityLevel::SlightlySimilar
        );
        assert_eq!(
            SimilarityLevel::from_score(0.20),
            SimilarityLevel::Unrelated
        );
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = OrderedLruCache::new(3);
        cache.put("a".into(), vec![1.0]);
        cache.put("b".into(), vec![2.0]);
        cache.put("c".into(), vec![3.0]);
        assert_eq!(cache.len(), 3);

        // 访问 a，使其成为最近使用
        assert_eq!(cache.get("a"), Some(&vec![1.0]));

        // 插入 d，淘汰最久未使用的 b
        cache.put("d".into(), vec![4.0]);
        assert!(cache.get("b").is_none());
        assert_eq!(cache.get("a"), Some(&vec![1.0]));
        assert_eq!(cache.get("d"), Some(&vec![4.0]));
        assert_eq!(cache.len(), 3);
    }
}
