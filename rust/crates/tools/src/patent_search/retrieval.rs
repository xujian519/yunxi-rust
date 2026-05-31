//! 混合检索工具包装层

use serde::Deserialize;
use serde_json::Value;

use patent_domain::retrieval::{HybridSearchConfig, HybridSearcher, RetrievedSnippet};

/// 混合检索输入
#[derive(Debug, Deserialize)]
pub struct HybridRetrievalInput {
    /// 检索查询
    pub query: String,
    /// 向量检索结果
    #[serde(default)]
    pub vector_results: Vec<RetrievedSnippet>,
    /// 图谱检索结果
    #[serde(default)]
    pub graph_results: Vec<RetrievedSnippet>,
    /// 法律库检索结果
    #[serde(default)]
    pub legal_results: Vec<RetrievedSnippet>,
    /// 向量权重（0-1）
    #[serde(default)]
    pub vector_weight: Option<f64>,
    /// 图谱权重（0-1）
    #[serde(default)]
    pub graph_weight: Option<f64>,
    /// 法律库权重（0-1）
    #[serde(default)]
    pub legal_weight: Option<f64>,
    /// 返回结果数量
    #[serde(default)]
    pub top_k: Option<usize>,
}

/// 执行混合检索
pub fn hybrid_retrieval(input: HybridRetrievalInput) -> Result<Value, String> {
    let config = HybridSearchConfig {
        vector_weight: input.vector_weight.unwrap_or(0.4),
        graph_weight: input.graph_weight.unwrap_or(0.3),
        legal_weight: input.legal_weight.unwrap_or(0.3),
        top_k: input.top_k.unwrap_or(10),
    };

    let searcher = HybridSearcher::new(config);
    let result = searcher.search(
        &input.query,
        input.vector_results,
        input.graph_results,
        input.legal_results,
    );

    serde_json::to_value(result).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_retrieval_basic() {
        let input = HybridRetrievalInput {
            query: "神经网络 图像识别".into(),
            vector_results: vec![RetrievedSnippet {
                source: "vector".into(),
                title: "向量结果1".into(),
                text: "基于神经网络的方法".into(),
                score: 0.9,
            }],
            graph_results: vec![RetrievedSnippet {
                source: "graph".into(),
                title: "图谱结果1".into(),
                text: "深度学习网络".into(),
                score: 0.8,
            }],
            legal_results: vec![],
            vector_weight: Some(0.5),
            graph_weight: Some(0.3),
            legal_weight: Some(0.2),
            top_k: Some(5),
        };

        let result = hybrid_retrieval(input).unwrap();
        assert!(result["snippets"].is_array());
        assert_eq!(result["graph_expanded"], true);
        assert_eq!(result["total_sources"], 2);
    }

    #[test]
    fn test_hybrid_retrieval_empty() {
        let input = HybridRetrievalInput {
            query: "测试查询".into(),
            vector_results: vec![],
            graph_results: vec![],
            legal_results: vec![],
            vector_weight: None,
            graph_weight: None,
            legal_weight: None,
            top_k: None,
        };

        let result = hybrid_retrieval(input).unwrap();
        assert_eq!(result["total_sources"], 0);
        assert_eq!(result["graph_expanded"], false);
    }
}
