//! 意图分类器
//!
//! 基于 Athena `core/intent/intent_recognition_adapter.py` 重写。
//! 使用关键词匹配 + 可选的嵌入相似度进行意图分类。

use crate::intent_types::IntentType;
use embedding::config::semantic_enabled;
use embedding::global::shared_optional;

/// 意图分类结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct IntentResult {
    pub intent: IntentType,
    pub confidence: f64,
    pub method: String,
    pub alternatives: Vec<(IntentType, f64)>,
}

/// 意图分类器
pub struct IntentClassifier {
    /// 是否使用嵌入增强（需要 BGE-M3 模型）
    use_embedding: bool,
}

impl IntentClassifier {
    /// `use_embedding`: 为 true 时在关键词置信度不足时尝试嵌入增强（仍需 `semantic.enabled`）
    pub fn new(use_embedding: bool) -> Self {
        Self { use_embedding }
    }

    /// 按用户配置：仅在 semantic.enabled 时启用嵌入
    pub fn from_user_settings() -> Self {
        Self {
            use_embedding: semantic_enabled(),
        }
    }

    /// 仅使用关键词匹配
    pub fn keyword_only() -> Self {
        Self {
            use_embedding: false,
        }
    }

    /// 分类用户输入的意图
    pub fn classify(&self, text: &str) -> IntentResult {
        let keyword_result = self.classify_by_keywords(text);

        // 如果关键词匹配置信度已经很高，直接返回
        if keyword_result.confidence >= 0.9 {
            return keyword_result;
        }

        // 如果启用了嵌入且模型可用，使用嵌入增强
        if self.use_embedding && semantic_enabled() {
            if let Some(embedding_result) = self.classify_by_embedding(text) {
                // 如果嵌入分类置信度更高，优先使用
                if embedding_result.confidence > keyword_result.confidence {
                    return embedding_result;
                }
            }
        }

        keyword_result
    }

    /// 关键词匹配分类
    fn classify_by_keywords(&self, text: &str) -> IntentResult {
        let all_intents = self.active_intents();
        let mut scored: Vec<(IntentType, f64)> = Vec::new();

        for intent in &all_intents {
            let keywords = intent.keywords();
            if keywords.is_empty() {
                continue;
            }

            let mut match_count = 0usize;
            let mut best_keyword_len = 0;
            let mut total_keyword_weight = 0.0;
            for kw in keywords {
                if text.contains(kw) {
                    match_count += 1;
                    best_keyword_len = best_keyword_len.max(kw.len());
                    // 更长的关键词权重更高
                    total_keyword_weight += kw.len() as f64 / 2.0;
                }
            }

            if match_count > 0 {
                let coverage = match_count as f64 / keywords.len() as f64;
                let length_bonus = best_keyword_len as f64 / 10.0;
                let score = (coverage * 0.4
                    + length_bonus.min(0.3) * 0.3
                    + (total_keyword_weight / 20.0).min(0.3))
                .min(1.0);
                scored.push((*intent, score));
            }
        }

        if scored.is_empty() {
            return IntentResult {
                intent: IntentType::Unknown,
                confidence: 0.0,
                method: "keyword".into(),
                alternatives: vec![],
            };
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let best = scored.first().unwrap();
        let best_intent = best.0;
        let best_score = best.1;

        let alternatives: Vec<(IntentType, f64)> = scored.into_iter().skip(1).take(3).collect();

        IntentResult {
            intent: best_intent,
            confidence: best_score,
            method: "keyword".into(),
            alternatives,
        }
    }

    /// 嵌入相似度分类（需要嵌入服务可用）
    fn classify_by_embedding(&self, text: &str) -> Option<IntentResult> {
        let svc = shared_optional()?;
        let text_vec = svc.encode(text).ok()?;

        // 为每个有描述性标签的意图计算嵌入相似度
        let intent_labels = [
            (IntentType::PatentDrafting, "撰写专利申请文件"),
            (IntentType::PatentSearch, "检索相关专利技术文献"),
            (IntentType::NoveltyApplication, "分析专利的新颖性"),
            (IntentType::CreativityApplication, "判断专利的创造性"),
            (IntentType::OpinionResponse, "答复审查意见通知书"),
            (IntentType::LiteralInfringement, "分析专利侵权风险"),
            (IntentType::InvalidationGrounds, "提出专利无效宣告请求"),
            (IntentType::LegalQuery, "查询知识产权法律法规"),
            (IntentType::GuidelineQuery, "查询专利审查指南"),
            (IntentType::ClaimDraftingStrategy, "制定权利要求撰写策略"),
            (IntentType::JudgmentPrediction, "预测案件判决结果"),
        ];

        let mut scored = Vec::new();
        let labels: Vec<&str> = intent_labels.iter().map(|(_, l)| *l).collect();
        let Ok(embeddings) = svc.encode_batch(&labels) else {
            return None;
        };

        for ((intent, _), intent_vec) in intent_labels.iter().zip(embeddings) {
            let sim = f64::from(embedding::service::EmbeddingService::cosine_similarity(
                &text_vec,
                &intent_vec,
            ));
            scored.push((*intent, sim));
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if scored.is_empty() {
            return None;
        }

        let best = scored.first().unwrap();
        let best_intent = best.0;
        let best_score = best.1;

        let alternatives: Vec<(IntentType, f64)> = scored.into_iter().skip(1).take(3).collect();

        Some(IntentResult {
            intent: best_intent,
            confidence: best_score,
            method: "embedding".into(),
            alternatives,
        })
    }

    /// 活跃意图列表（仅包含有对应关键词的意图）
    fn active_intents(&self) -> Vec<IntentType> {
        use IntentType::*;
        vec![
            PatentDrafting,
            ClaimDraftingStrategy,
            PatentSearch,
            NoveltyApplication,
            NoveltyRejection,
            CreativityApplication,
            CreativityRejection,
            OpinionResponse,
            LiteralInfringement,
            InvalidationGrounds,
            LegalQuery,
            GuidelineQuery,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classifier() -> IntentClassifier {
        IntentClassifier::keyword_only()
    }

    #[test]
    fn test_patent_drafting() {
        let result = classifier().classify("帮我撰写一份关于图像识别的专利申请");
        assert_eq!(result.intent, IntentType::PatentDrafting);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_patent_search() {
        let result = classifier().classify("检索深度学习相关专利");
        assert_eq!(result.intent, IntentType::PatentSearch);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_novelty() {
        let result = classifier().classify("分析这个专利的新颖性");
        assert_eq!(result.intent, IntentType::NoveltyApplication);
    }

    #[test]
    fn test_creativity() {
        let result = classifier().classify("判断创造性");
        assert_eq!(result.intent, IntentType::CreativityApplication);
    }

    #[test]
    fn test_oa_response() {
        let result = classifier().classify("审查意见答复怎么做");
        assert_eq!(result.intent, IntentType::OpinionResponse);
    }

    #[test]
    fn test_infringement() {
        let result = classifier().classify("分析专利侵权风险");
        assert_eq!(result.intent, IntentType::LiteralInfringement);
    }

    #[test]
    fn test_unknown() {
        let result = classifier().classify("今天天气怎么样");
        assert_eq!(result.intent, IntentType::Unknown);
    }

    #[test]
    fn test_legal_query() {
        let result = classifier().classify("查询知识产权法律规定");
        assert_eq!(result.intent, IntentType::LegalQuery);
    }
}
