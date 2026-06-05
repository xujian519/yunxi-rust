//! 语义检索增强
//!
//! 当 embedding feature 启用时，提供基于向量的语义检索能力。
//! 写入时自动编码并存储向量，查询时使用余弦相似度搜索。
//! 未启用时，所有操作静默降级（返回空结果）。

/// 语义搜索结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SemanticSearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub metadata: serde_json::Value,
}

/// 语义检索引擎
///
/// 通过 `disabled()` 创建不启用语义的空实例（向后兼容），
/// 或通过 `enabled(service, store)` 创建完整语义搜索能力。
pub struct SemanticSearch {
    #[cfg(feature = "semantic")]
    service: Option<std::sync::Arc<embedding::EmbeddingService>>,
    #[cfg(feature = "semantic")]
    store: Option<embedding::VectorStore>,
}

impl SemanticSearch {
    /// 创建不启用语义的空实例。
    ///
    /// 所有 `index` / `search` / `remove` 调用将静默降级，
    /// 不产生任何副作用。
    pub fn disabled() -> Self {
        Self {
            #[cfg(feature = "semantic")]
            service: None,
            #[cfg(feature = "semantic")]
            store: None,
        }
    }

    /// 启用语义搜索（需要 embedding feature）。
    ///
    /// # Arguments
    /// * `service` - 嵌入服务，用于将文本编码为向量
    /// * `store` - 向量存储，用于持久化和检索向量
    #[cfg(feature = "semantic")]
    pub fn enabled(
        service: std::sync::Arc<embedding::EmbeddingService>,
        store: embedding::VectorStore,
    ) -> Self {
        Self {
            service: Some(service),
            store: Some(store),
        }
    }

    /// 将文本编码为向量并存储到向量库。
    ///
    /// `collection` 固定为 `"memory"`。
    /// 如果语义未启用或服务不可用，静默返回 `Ok(())`。
    pub fn index(
        &self,
        id: &str,
        content: &str,
        metadata: serde_json::Value,
    ) -> Result<(), String> {
        #[cfg(feature = "semantic")]
        {
            let service = match self.service.as_ref() {
                Some(s) => s,
                None => return Ok(()),
            };
            let store = match self.store.as_ref() {
                Some(s) => s,
                None => return Ok(()),
            };

            match service.encode(content) {
                Ok(vector) => {
                    if let Err(e) = store.upsert("memory", id, &vector, metadata) {
                        eprintln!("[memory/semantic] 向量写入失败: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("[memory/semantic] 编码失败（降级跳过）: {e}");
                }
            }
        }
        let _ = (id, content, metadata);
        Ok(())
    }

    /// 语义搜索。
    ///
    /// 返回按相似度降序排列的结果。
    /// 如果语义未启用或服务不可用，返回空 `Vec`。
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SemanticSearchResult>, String> {
        #[cfg(feature = "semantic")]
        {
            let service = match self.service.as_ref() {
                Some(s) => s,
                None => return Ok(vec![]),
            };
            let store = match self.store.as_ref() {
                Some(s) => s,
                None => return Ok(vec![]),
            };

            let query_vec = match service.encode(query) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("[memory/semantic] 查询编码失败: {e}");
                    return Ok(vec![]);
                }
            };

            match store.search("memory", &query_vec, limit) {
                Ok(results) => Ok(results
                    .into_iter()
                    .map(|r| SemanticSearchResult {
                        id: r.id,
                        content: String::new(), // content 从 tier_store 获取
                        score: r.score,
                        metadata: r.metadata,
                    })
                    .collect()),
                Err(e) => {
                    eprintln!("[memory/semantic] 向量搜索失败: {e}");
                    Ok(vec![])
                }
            }
        }
        #[cfg(not(feature = "semantic"))]
        {
            let _ = (query, limit);
            Ok(vec![])
        }
    }

    /// 从向量库删除指定条目。
    ///
    /// 如果语义未启用或删除失败，静默返回 `Ok(())`。
    pub fn remove(&self, id: &str) -> Result<(), String> {
        #[cfg(feature = "semantic")]
        {
            if let Some(ref store) = self.store {
                if let Err(e) = store.delete("memory", id) {
                    eprintln!("[memory/semantic] 向量删除失败: {e}");
                }
            }
        }
        let _ = id;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_disabled() {
        let ss = SemanticSearch::disabled();
        // index 应该静默成功
        assert!(ss
            .index(
                "test-id",
                "test content",
                serde_json::json!({"key": "value"})
            )
            .is_ok());
        // search 应该返回空 Vec
        let results = ss.search("test query", 10).unwrap();
        assert!(results.is_empty());
        // remove 应该静默成功
        assert!(ss.remove("test-id").is_ok());
    }

    #[test]
    fn test_semantic_search_result_serde() {
        let result = SemanticSearchResult {
            id: "test-id".to_string(),
            content: "test content".to_string(),
            score: 0.95,
            metadata: serde_json::json!({"tag": "reflection"}),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SemanticSearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.score, 0.95);
        assert_eq!(deserialized.metadata["tag"], "reflection");
    }
}
