//! 知识库通用类型定义

use serde::{Deserialize, Serialize};

/// 法律法规文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawDocument {
    pub id: String,
    pub level: String,
    pub name: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub publish: Option<String>,
    pub expired: bool,
    pub category_id: i64,
    #[serde(default)]
    pub content: Option<String>,
}

/// 法律法规类别
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LawCategory {
    pub id: i64,
    pub name: String,
    pub folder: String,
    pub is_sub_folder: bool,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub order: Option<i64>,
}

/// 知识卡片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCard {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub concept: String,
    #[serde(default)]
    pub quality: f64,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub file_path: String,
    #[serde(default)]
    pub related_concepts: Vec<String>,
    #[serde(default)]
    pub generated_at: String,
    #[serde(default)]
    pub version: u32,
    /// 卡片正文内容（加载后填充）
    #[serde(default)]
    pub content: String,
}

/// 搜索来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SearchSource {
    KnowledgeGraph,
    LawDatabase,
    KnowledgeCard,
    /// 预构建语义 chunk（`.yunpat-semantic-index.sqlite`）
    SemanticChunk,
}

/// 搜索结果条目
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub source: SearchSource,
    pub title: String,
    pub content: String,
    pub score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_type: Option<String>,
}

/// 知识来源类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum KnowledgeCategory {
    PatentLaw,
    TrademarkLaw,
    CopyrightLaw,
    GuidelineRule,
    SupremeCourtCase,
    IpcSection,
    Other,
}
