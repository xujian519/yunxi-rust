//! 意图分类器
//!
//! 基于 Athena `core/intent/intent_recognition_adapter.py` 重写。
//! 使用关键词匹配 + 可选的嵌入相似度进行意图分类。

use std::sync::Arc;

use crate::intent_types::IntentType;
use embedding::config::semantic_enabled;
use embedding::global::shared_optional;
use memory::UnifiedMemory;

/// 意图分类结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct IntentResult {
    pub intent: IntentType,
    pub confidence: f64,
    pub method: String,
    pub alternatives: Vec<(IntentType, f64)>,
}

pub struct IntentClassifier {
    /// 是否使用嵌入增强（需要 BGE-M3 模型）
    use_embedding: bool,
    /// 用户偏好记忆（可选）
    memory: Option<Arc<UnifiedMemory>>,
}

impl IntentClassifier {
    /// `use_embedding`: 为 true 时在关键词置信度不足时尝试嵌入增强（仍需 `semantic.enabled`）
    pub fn new(use_embedding: bool) -> Self {
        Self {
            use_embedding,
            memory: None,
        }
    }

    /// 按用户配置：仅在 semantic.enabled 时启用嵌入
    pub fn from_user_settings() -> Self {
        Self {
            use_embedding: semantic_enabled(),
            memory: None,
        }
    }

    /// 注入记忆系统（用于读取/记录用户偏好）。
    pub fn with_memory(mut self, memory: Arc<UnifiedMemory>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// 仅使用关键词匹配
    pub fn keyword_only() -> Self {
        Self {
            use_embedding: false,
            memory: None,
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
                    let mut result = embedding_result;
                    self.apply_preference_boost(&mut result);
                    return result;
                }
            }
        }

        let mut result = keyword_result;
        self.apply_preference_boost(&mut result);
        result
    }

    /// 从记忆系统读取用户意图偏好，调整分类结果。
    fn apply_preference_boost(&self, result: &mut IntentResult) {
        let Some(ref memory) = self.memory else {
            return;
        };

        // 搜索所有意图偏好记录
        let prefs = memory.search("intent:preference", 50);
        if prefs.is_empty() {
            return;
        }

        // 解析偏好：content 格式为 "intent:preference:{IntentTypeName}:{count}"
        let mut boosts: Vec<(String, f64)> = Vec::new();
        for entry in &prefs {
            let parts: Vec<&str> = entry.content.split(':').collect();
            if parts.len() >= 4 && parts[0] == "intent" && parts[1] == "preference" {
                if let Ok(count) = parts[3].parse::<f64>() {
                    // 每次使用提升 ~10%，最多累计 +50%
                    let boost = (count * 0.10).min(0.50);
                    boosts.push((parts[2].to_string(), boost));
                }
            }
        }

        if boosts.is_empty() {
            return;
        }

        // 对主意图和备选意图应用偏好提升
        for (intent_name, boost) in &boosts {
            let current_name = format!("{:?}", result.intent);
            if current_name == *intent_name {
                result.confidence = (result.confidence * (1.0 + boost)).min(1.0);
            }
            for alt in &mut result.alternatives {
                let alt_name = format!("{:?}", alt.0);
                if alt_name == *intent_name {
                    let boosted = (alt.1 * (1.0 + boost)).min(1.0);
                    // 如果备选超过主意图，向上冒泡
                    if boosted > result.confidence {
                        result.confidence = boosted;
                        result.intent = alt.0;
                        result.method.push_str("+preference");
                    }
                    alt.1 = boosted;
                }
            }
        }
    }

    /// 记录用户意图偏好（当用户修正了自动分类结果时调用）。
    ///
    /// 每次调用增加该意图的偏好计数，使后续分类更倾向于该意图。
    pub fn record_user_preference(memory: &UnifiedMemory, intent: IntentType) {
        let name = format!("{intent:?}");
        let id = format!("intent:preference:{name}");

        // 读取现有计数
        let count = match memory.retrieve(&id) {
            Ok(Some(content)) => {
                content
                    .split(':')
                    .next_back()
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0)
                    + 1
            }
            _ => 1,
        };

        let importance = (0.6 + count as f64 * 0.05).min(0.9);
        let content = format!("intent:preference:{name}:{count}");

        let _ = memory.remember_rich(
            &id,
            "intent",
            &content,
            None,
            vec!["intent".into(), "preference".into()],
            importance,
            "intent_preference",
        );
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

        // 为每个有描述性标签的意图计算嵌入相似度（覆盖全部 50 种意图）
        let intent_labels = [
            // 专利撰写
            (IntentType::PatentDrafting, "撰写专利申请文件"),
            (IntentType::ClaimDraftingStrategy, "制定权利要求撰写策略"),
            (IntentType::DefensiveDrafting, "防御性专利撰写保护策略"),
            (IntentType::BroadScopeProtection, "宽范围权利要求保护"),
            (IntentType::ClaimAmendment, "修改权利要求"),
            // 专利检索
            (IntentType::PatentSearch, "检索相关专利技术文献"),
            (IntentType::CaseSearchInvalidation, "搜索无效宣告案例"),
            // 新颖性
            (IntentType::NoveltyApplication, "分析专利的新颖性"),
            (IntentType::NoveltyRejection, "新颖性驳回分析"),
            // 创造性
            (IntentType::CreativityApplication, "判断专利的创造性"),
            (IntentType::CreativityRejection, "创造性驳回分析"),
            // 审查意见
            (IntentType::OpinionResponse, "答复审查意见通知书"),
            (IntentType::ArgumentDrafting, "撰写答辩意见"),
            // 侵权分析
            (IntentType::LiteralInfringement, "分析专利侵权风险"),
            (IntentType::EquivalentInfringement, "等同侵权分析"),
            (IntentType::AllElementsRule, "全面覆盖原则分析"),
            (IntentType::DoctrineOfEquivalents, "等同原则判定"),
            (IntentType::ProsecutionHistoryEstoppel, "禁止反悔原则"),
            (IntentType::LiteralInterpretation, "权利要求字面解释"),
            // 权利要求解释
            (IntentType::ScopeClaimOnly, "权利要求保护范围限定"),
            (IntentType::ScopeWithSpecification, "结合说明书解释权利要求"),
            (IntentType::ScopeWithProsecution, "结合审查档案解释权利要求"),
            (IntentType::DefinitionClarity, "权利要求术语定义清晰度"),
            (IntentType::SpecificationSupport, "说明书对权利要求的支持"),
            (IntentType::SupportDisclosure, "充分公开要求分析"),
            // 无效宣告
            (IntentType::InvalidationGrounds, "提出专利无效宣告请求"),
            (IntentType::InvalidationDefense, "无效宣告答辩"),
            (IntentType::InvalidationDrafting, "撰写无效宣告请求书"),
            // 审查标准
            (IntentType::ExaminationStandard, "专利审查标准查询"),
            (IntentType::GuidelineComparison, "审查指南对比分析"),
            (IntentType::GuidelineQuery, "查询专利审查指南"),
            (IntentType::RuleInterpretation, "法条规则解释"),
            (IntentType::SectionLookup, "查询具体法条条款"),
            // 法律分析
            (IntentType::LegalQuery, "查询知识产权法律法规"),
            (IntentType::LegalResearch, "法律研究与案例检索"),
            (IntentType::JudgmentPrediction, "预测案件判决结果"),
            (IntentType::AddedSubjectMatter, "超范围修改分析"),
            (IntentType::EvidenceCollection, "证据收集与举证"),
            (IntentType::DefenseAnalysis, "抗辩分析"),
            // 通用
            (IntentType::CodeGeneration, "生成编程代码"),
            (IntentType::CodeReview, "代码审查"),
            (IntentType::CreativeWriting, "创意写作"),
            (IntentType::DataAnalysis, "数据分析统计"),
            (IntentType::ProblemSolving, "解决问题方案"),
            (IntentType::Emotional, "情感情绪交流"),
            (IntentType::LifestyleService, "生活服务推荐"),
            (IntentType::WeatherQuery, "天气查询"),
            (IntentType::MapNavigation, "地图导航路线"),
            (IntentType::TrafficQuery, "交通路况查询"),
            (IntentType::SystemControl, "系统设置控制"),
            (IntentType::CrimeAnalysis, "犯罪量刑分析"),
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

    /// 活跃意图列表（包含所有 50 种有对应关键词的意图，排除 Unknown）
    fn active_intents(&self) -> Vec<IntentType> {
        use IntentType::*;
        vec![
            // 专利撰写 (5)
            PatentDrafting,
            ClaimDraftingStrategy,
            DefensiveDrafting,
            BroadScopeProtection,
            ClaimAmendment,
            // 专利检索 (2)
            PatentSearch,
            CaseSearchInvalidation,
            // 新颖性 (2)
            NoveltyApplication,
            NoveltyRejection,
            // 创造性 (2)
            CreativityApplication,
            CreativityRejection,
            // 审查意见 (2)
            OpinionResponse,
            ArgumentDrafting,
            // 侵权分析 (6)
            LiteralInfringement,
            EquivalentInfringement,
            AllElementsRule,
            DoctrineOfEquivalents,
            ProsecutionHistoryEstoppel,
            LiteralInterpretation,
            // 权利要求解释 (6)
            ScopeClaimOnly,
            ScopeWithSpecification,
            ScopeWithProsecution,
            DefinitionClarity,
            SpecificationSupport,
            SupportDisclosure,
            // 无效宣告 (3)
            InvalidationGrounds,
            InvalidationDefense,
            InvalidationDrafting,
            // 审查标准 (5)
            ExaminationStandard,
            GuidelineComparison,
            GuidelineQuery,
            RuleInterpretation,
            SectionLookup,
            // 法律分析 (6)
            LegalQuery,
            LegalResearch,
            JudgmentPrediction,
            AddedSubjectMatter,
            EvidenceCollection,
            DefenseAnalysis,
            // 通用 (11)
            CodeGeneration,
            CodeReview,
            CreativeWriting,
            DataAnalysis,
            ProblemSolving,
            Emotional,
            LifestyleService,
            WeatherQuery,
            MapNavigation,
            TrafficQuery,
            SystemControl,
            CrimeAnalysis,
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
        let result = classifier().classify("随便说点什么吧");
        assert_eq!(result.intent, IntentType::Unknown);
    }

    #[test]
    fn test_legal_query() {
        let result = classifier().classify("查询知识产权法律规定");
        assert_eq!(result.intent, IntentType::LegalQuery);
    }
}
