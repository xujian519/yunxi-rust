//! 统一搜索接口
//!
//! 跨知识图谱、法律法规、知识卡片的统一搜索引擎。
//! 支持 FTS5 文本搜索和向量语义搜索两种模式。

use crate::knowledge_cards::CardIndex;
use crate::law_db::LawDatabase;
use crate::semantic_index::PrebuiltSemanticIndex;
use crate::types::{SearchResult, SearchSource};

/// 索引统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct IndexStats {
    pub collection: String,
    pub total_nodes: usize,
    pub indexed: usize,
    pub failed: usize,
}
use embedding::service::EmbeddingService;
use embedding::vector_store::VectorStore;
use patent_domain::sqlite_graph::SqliteKnowledgeGraph;

/// 统一搜索引擎
pub struct UnifiedSearch {
    kg: Option<SqliteKnowledgeGraph>,
    law_db: Option<LawDatabase>,
    card_index: Option<CardIndex>,
    embedding_svc: Option<EmbeddingService>,
    vector_store: Option<VectorStore>,
}

/// 搜索模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// FTS5 关键词搜索（默认）
    Text,
    /// BGE-M3 向量语义搜索
    Semantic,
    /// 混合搜索：文本 + 语义
    Hybrid,
}

impl Default for SearchMode {
    fn default() -> Self {
        Self::Text
    }
}

/// 搜索配置
pub struct SearchConfig {
    pub query: String,
    pub limit: usize,
    pub search_kg: bool,
    pub search_law: bool,
    pub search_cards: bool,
    pub min_card_quality: f64,
    pub mode: SearchMode,
    /// 语义搜索权重（混合模式下：文本分 × text_weight + 向量分 × (1 - text_weight)）
    pub text_weight: f64,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 20,
            search_kg: true,
            search_law: true,
            search_cards: true,
            min_card_quality: 0.5,
            mode: SearchMode::Text,
            text_weight: 0.4,
        }
    }
}

impl UnifiedSearch {
    /// 创建搜索引擎，指定各数据源路径（路径不存在则跳过该数据源）
    pub fn new(
        kg_path: Option<&str>,
        law_db_path: Option<&str>,
        card_index_path: Option<&str>,
    ) -> Self {
        let kg = kg_path.and_then(|p| {
            if std::path::Path::new(p).exists() {
                SqliteKnowledgeGraph::open_readonly(p).ok()
            } else {
                None
            }
        });

        let law_db = law_db_path.and_then(|p| {
            if std::path::Path::new(p).exists() {
                LawDatabase::open(p).ok()
            } else {
                None
            }
        });

        let card_index = card_index_path.and_then(|p| {
            if std::path::Path::new(p).exists() {
                CardIndex::load(p).ok()
            } else {
                None
            }
        });

        // 仅在用户开启 semantic.enabled 时加载嵌入（HTTP 8766 或本地 ONNX）
        let embedding_svc = embedding::global::shared_optional();
        let vector_store = if embedding_svc.is_some() {
            VectorStore::open_default(1024).ok()
        } else {
            None
        };

        Self {
            kg,
            law_db,
            card_index,
            embedding_svc,
            vector_store,
        }
    }

    /// 使用显式的嵌入服务创建
    pub fn with_embedding(
        kg_path: Option<&str>,
        law_db_path: Option<&str>,
        card_index_path: Option<&str>,
        embedding_svc: EmbeddingService,
        vector_store: VectorStore,
    ) -> Self {
        let mut search = Self::new(kg_path, law_db_path, card_index_path);
        search.embedding_svc = Some(embedding_svc);
        search.vector_store = Some(vector_store);
        search
    }

    /// 执行统一搜索
    pub fn search(&self, config: &SearchConfig) -> Vec<SearchResult> {
        match config.mode {
            SearchMode::Text => self.search_text(config),
            SearchMode::Semantic => self.search_semantic(config),
            SearchMode::Hybrid => self.search_hybrid(config),
        }
    }

    /// 纯文本搜索（原有逻辑）
    fn search_text(&self, config: &SearchConfig) -> Vec<SearchResult> {
        let mut results = Vec::new();

        if config.search_kg {
            let remaining = config.limit.saturating_sub(results.len());
            if remaining > 0 {
                if let Some(ref kg) = self.kg {
                    if let Ok(nodes) = kg.search_nodes(&config.query, None, remaining) {
                        for node in nodes {
                            results.push(SearchResult {
                                source: SearchSource::KnowledgeGraph,
                                title: if node.name.is_empty() {
                                    node.title.clone()
                                } else {
                                    node.name.clone()
                                },
                                content: node.content.clone().unwrap_or_default(),
                                score: 0.8,
                                id: Some(node.id.clone()),
                                item_type: Some(node.node_type.clone()),
                            });
                        }
                    }
                }
            }
        }

        if config.search_law {
            let remaining = config.limit.saturating_sub(results.len());
            if remaining > 0 {
                if let Some(ref db) = self.law_db {
                    if let Ok(laws) = db.search_by_content(&config.query, remaining) {
                        for law in laws {
                            results.push(SearchResult {
                                source: SearchSource::LawDatabase,
                                title: law.name.clone(),
                                content: law.content.clone().unwrap_or_default(),
                                score: 0.7,
                                id: Some(law.id.clone()),
                                item_type: Some(law.level.clone()),
                            });
                        }
                    }
                }
            }
        }

        if config.search_cards {
            let remaining = config.limit.saturating_sub(results.len());
            if remaining > 0 {
                if let Some(ref index) = self.card_index {
                    let cards = index.search_by_keyword(&config.query, remaining);
                    for mut card in cards {
                        let _ = index.load_content(&mut card);
                        results.push(SearchResult {
                            source: SearchSource::KnowledgeCard,
                            title: card.title.clone(),
                            content: card.content.clone(),
                            score: card.quality,
                            id: Some(card.id.clone()),
                            item_type: Some(card.concept.clone()),
                        });
                    }
                }
            }
        }

        results
    }

    /// 纯语义搜索（优先预构建 `.yunpat-semantic-index.sqlite`）
    fn search_semantic(&self, config: &SearchConfig) -> Vec<SearchResult> {
        let Some(ref svc) = self.embedding_svc else {
            return self.search_text(config);
        };

        let Ok(query_vec) = svc.encode(&config.query) else {
            return self.search_text(config);
        };

        let mut results =
            self.search_prebuilt_semantic(&query_vec, &config.query, config.limit, 1.0);

        // 补充 ~/.yunxi/vectors 中的 KG/法规向量（若已索引）
        if let Some(ref store) = self.vector_store {
            let collections = ["knowledge_graph", "laws", "knowledge_cards"];
            for collection in &collections {
                if let Ok(vec_results) = store.search(collection, &query_vec, config.limit) {
                    for vr in vec_results {
                        results.push(SearchResult {
                            source: match *collection {
                                "knowledge_graph" => SearchSource::KnowledgeGraph,
                                "laws" => SearchSource::LawDatabase,
                                _ => SearchSource::KnowledgeCard,
                            },
                            title: vr.id.clone(),
                            content: String::new(),
                            score: f64::from(vr.score),
                            id: Some(vr.id.clone()),
                            item_type: vr
                                .metadata
                                .get("type")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        });
                    }
                }
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(config.limit);
        results
    }

    /// 从预构建 BGE 语义库检索（Obsidian/知识库 Markdown chunks）
    fn search_prebuilt_semantic(
        &self,
        query_vec: &[f32],
        query: &str,
        limit: usize,
        alpha: f32,
    ) -> Vec<SearchResult> {
        let Some(index) = PrebuiltSemanticIndex::open_default() else {
            return vec![];
        };
        let hits = if alpha >= 0.99 {
            index.search_vector(query_vec, limit)
        } else {
            index.hybrid_search(query_vec, query, limit, alpha)
        };
        hits.into_iter()
            .map(|h| SearchResult {
                source: SearchSource::SemanticChunk,
                title: h.title,
                content: h.content,
                score: f64::from(h.score),
                id: Some(h.chunk_id),
                item_type: Some(h.file_path),
            })
            .collect()
    }

    /// 混合搜索：FTS + 预构建语义库（+ 可选辅助向量集合）
    fn search_hybrid(&self, config: &SearchConfig) -> Vec<SearchResult> {
        let mut combined = self.search_text(config);

        let Some(ref svc) = self.embedding_svc else {
            return combined;
        };

        let Ok(query_vec) = svc.encode(&config.query) else {
            return combined;
        };

        let semantic_weight = 1.0 - config.text_weight;
        let alpha = 0.7_f32;

        let mut seen_ids: std::collections::HashSet<String> =
            combined.iter().filter_map(|r| r.id.clone()).collect();

        for mut hit in self.search_prebuilt_semantic(&query_vec, &config.query, config.limit, alpha)
        {
            let dedupe_key = hit
                .item_type
                .clone()
                .or_else(|| hit.id.clone())
                .unwrap_or_else(|| hit.title.clone());
            if seen_ids.contains(&dedupe_key) {
                continue;
            }
            seen_ids.insert(dedupe_key);
            hit.score *= semantic_weight;
            combined.push(hit);
        }

        if let Some(ref store) = self.vector_store {
            let collections = ["knowledge_graph", "laws"];
            for collection in &collections {
                if let Ok(vec_results) = store.search(collection, &query_vec, config.limit) {
                    for vr in vec_results {
                        if seen_ids.contains(&vr.id) {
                            continue;
                        }
                        seen_ids.insert(vr.id.clone());
                        combined.push(SearchResult {
                            source: match *collection {
                                "knowledge_graph" => SearchSource::KnowledgeGraph,
                                _ => SearchSource::LawDatabase,
                            },
                            title: vr.id.clone(),
                            content: String::new(),
                            score: f64::from(vr.score) * semantic_weight,
                            id: Some(vr.id.clone()),
                            item_type: vr
                                .metadata
                                .get("type")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        });
                    }
                }
            }
        }

        for result in &mut combined {
            if result.source != SearchSource::SemanticChunk
                && result.score > 0.0
                && result.score <= 1.0
            {
                result.score *= config.text_weight;
            }
        }

        combined.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        combined.truncate(config.limit);
        combined
    }

    /// 将文档索引到向量存储（用于语义搜索的前提条件）
    pub fn index_document(
        &self,
        collection: &str,
        id: &str,
        text: &str,
        doc_type: &str,
    ) -> Result<(), String> {
        let svc = self
            .embedding_svc
            .as_ref()
            .ok_or("embedding service not available")?;
        let store = self
            .vector_store
            .as_ref()
            .ok_or("vector store not available")?;

        let vector = svc.encode(text).map_err(|e| e.to_string())?;
        store
            .upsert(
                collection,
                id,
                &vector,
                serde_json::json!({"type": doc_type}),
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// 批量索引知识图谱节点到向量存储（冷启动）
    ///
    /// 将 KG 中所有节点编码为向量并存入向量存储，使语义搜索可用。
    /// 首次使用语义搜索前必须调用此方法。
    pub fn index_knowledge_graph(&self, batch_size: usize) -> Result<IndexStats, String> {
        let svc = self
            .embedding_svc
            .as_ref()
            .ok_or("embedding service not available")?;
        let store = self
            .vector_store
            .as_ref()
            .ok_or("vector store not available")?;
        let kg = self.kg.as_ref().ok_or("knowledge graph not available")?;

        let stats = kg.stats().map_err(|e| e.to_string())?;
        let types = kg.node_type_distribution().map_err(|e| e.to_string())?;

        let mut indexed = 0usize;
        let mut failed = 0usize;

        for type_entry in &types {
            let nodes = kg
                .get_nodes_by_type(&type_entry.node_type, stats.node_count)
                .map_err(|e| e.to_string())?;

            for chunk in nodes.chunks(batch_size) {
                let texts: Vec<String> = chunk
                    .iter()
                    .map(|n| format!("{} {}", n.name, n.content.as_deref().unwrap_or("")))
                    .collect();

                let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
                match svc.encode_batch(&refs) {
                    Ok(vectors) => {
                        let items: Vec<_> = chunk
                            .iter()
                            .zip(vectors.iter())
                            .map(|(node, vec)| {
                                (
                                    node.id.clone(),
                                    vec.clone(),
                                    serde_json::json!({
                                        "type": node.node_type,
                                        "name": node.name,
                                    }),
                                )
                            })
                            .collect();
                        match store.upsert_batch("knowledge_graph", &items) {
                            Ok((ins, skip)) => {
                                indexed += ins;
                                failed += skip;
                            }
                            Err(e) => {
                                failed += items.len();
                                eprintln!("batch upsert failed: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        failed += chunk.len();
                        eprintln!("batch encode failed: {e}");
                    }
                }
            }
        }

        Ok(IndexStats {
            collection: "knowledge_graph".into(),
            total_nodes: stats.node_count,
            indexed,
            failed,
        })
    }

    /// 批量索引法律法规到向量存储
    pub fn index_law_database(&self, batch_size: usize) -> Result<IndexStats, String> {
        let svc = self
            .embedding_svc
            .as_ref()
            .ok_or("embedding service not available")?;
        let store = self
            .vector_store
            .as_ref()
            .ok_or("vector store not available")?;
        let db = self.law_db.as_ref().ok_or("law database not available")?;

        let total = db.count().map_err(|e| e.to_string())?;
        let mut indexed = 0usize;
        let mut failed = 0usize;

        // 分批加载和索引
        let mut offset = 0;
        loop {
            let laws = db.list_all(batch_size, offset).map_err(|e| e.to_string())?;
            if laws.is_empty() {
                break;
            }

            for chunk in laws.chunks(batch_size) {
                let texts: Vec<String> = chunk
                    .iter()
                    .map(|law| format!("{} {}", law.name, law.content.as_deref().unwrap_or("")))
                    .collect();

                let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
                match svc.encode_batch(&refs) {
                    Ok(vectors) => {
                        let items: Vec<_> = chunk
                            .iter()
                            .zip(vectors.iter())
                            .map(|(law, vec)| {
                                (
                                    law.id.clone(),
                                    vec.clone(),
                                    serde_json::json!({
                                        "type": law.level,
                                        "name": law.name,
                                    }),
                                )
                            })
                            .collect();
                        match store.upsert_batch("laws", &items) {
                            Ok((ins, skip)) => {
                                indexed += ins;
                                failed += skip;
                            }
                            Err(e) => {
                                failed += items.len();
                                eprintln!("batch upsert failed: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        failed += chunk.len();
                        eprintln!("batch encode failed: {e}");
                    }
                }
            }
            offset += batch_size;
        }

        Ok(IndexStats {
            collection: "laws".into(),
            total_nodes: total,
            indexed,
            failed,
        })
    }

    /// 检查各数据源是否可用
    pub fn status(&self) -> serde_json::Value {
        let prebuilt = PrebuiltSemanticIndex::open_default().map(|idx| {
            serde_json::json!({
                "path": idx.db_path().display().to_string(),
                "chunks": idx.chunk_count(),
                "embedding_model": idx.embedding_model(),
            })
        });
        serde_json::json!({
            "knowledge_graph": self.kg.is_some(),
            "law_database": self.law_db.is_some(),
            "card_index": self.card_index.as_ref().map_or(0, |c| c.len()),
            "prebuilt_semantic_index": prebuilt,
            "query_embedding": self.embedding_svc.is_some(),
            "aux_vector_store": self.vector_store.is_some(),
            "semantic_search": self.embedding_svc.is_some()
                && (PrebuiltSemanticIndex::open_default().is_some() || self.vector_store.is_some()),
        })
    }

    /// 嵌入服务是否可用（用于查询编码）
    pub fn has_embedding(&self) -> bool {
        self.embedding_svc.is_some()
    }

    /// 预构建 BGE 语义库是否可用
    pub fn has_prebuilt_semantic_index(&self) -> bool {
        PrebuiltSemanticIndex::open_default().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_search_engine() -> Option<UnifiedSearch> {
        let paths = crate::KnowledgePaths::discover();
        if paths.patent_kg_db.is_none() && paths.laws_db.is_none() && paths.card_index.is_none() {
            return None;
        }
        Some(UnifiedSearch::new(
            paths.patent_kg_db.as_deref(),
            paths.laws_db.as_deref(),
            paths.card_index.as_deref(),
        ))
    }

    #[test]
    fn test_unified_search() {
        let Some(engine) = make_search_engine() else {
            eprintln!("skipped: no local knowledge data");
            return;
        };
        let config = SearchConfig {
            query: "创造性".into(),
            limit: 5,
            ..Default::default()
        };
        let results = engine.search(&config);
        assert!(!results.is_empty(), "should find results for '创造性'");
    }

    #[test]
    fn test_search_status() {
        let Some(engine) = make_search_engine() else {
            eprintln!("skipped: no local knowledge data");
            return;
        };
        let status = engine.status();
        assert!(status["knowledge_graph"].as_bool().unwrap_or(false));
        assert!(status["law_database"].as_bool().unwrap_or(false));
    }

    #[test]
    fn test_semantic_search_fallback() {
        let Some(engine) = make_search_engine() else {
            eprintln!("skipped: no local knowledge data");
            return;
        };
        let config = SearchConfig {
            query: "新颖性".into(),
            limit: 5,
            mode: SearchMode::Semantic,
            ..Default::default()
        };
        let results = engine.search(&config);
        assert!(!results.is_empty(), "should fallback to text search");
    }

    #[test]
    fn test_hybrid_search_fallback() {
        let Some(engine) = make_search_engine() else {
            eprintln!("skipped: no local knowledge data");
            return;
        };
        let config = SearchConfig {
            query: "专利".into(),
            limit: 5,
            mode: SearchMode::Hybrid,
            ..Default::default()
        };
        let results = engine.search(&config);
        assert!(!results.is_empty(), "hybrid should work without embedding");
    }
}
