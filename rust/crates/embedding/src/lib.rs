//! 向量嵌入与语义搜索引擎
//!
//! 基于 BGE-M3 ONNX 模型的文本嵌入服务，支持：
//! - 1024 维向量编码（单文本/批量）
//! - SQLite 持久化向量存储
//! - 余弦相似度计算
//! - LRU 编码缓存

pub mod config;
pub mod global;
pub mod http_backend;
pub mod service;
pub mod vector_store;
pub mod vision;

pub use config::{
    default_vision_model, load_semantic_config, load_vision_config, semantic_enabled,
    KnowledgeSearchMode, ResolvedVisionHttp, SemanticConfig, VisionConfig,
};
pub use global::{
    reload as reload_global_embedding, shared_optional, shared_required, status_json,
};
pub use service::{EmbeddingService, SimilarityLevel};
pub use vector_store::{VectorSearchResult, VectorStore};
pub use vision::{ocr_image_from_path, vision_ocr_configured, VisionOcrResult};
