//! 意图类型定义
//!
//! 基于 Athena 的 50 个专利法律意图类别。

use serde::{Deserialize, Serialize};

/// 意图类型（对应 Athena 的 intent_classes）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntentType {
    // 专利撰写
    PatentDrafting,
    ClaimDraftingStrategy,
    DefensiveDrafting,
    BroadScopeProtection,
    ClaimAmendment,

    // 专利检索
    PatentSearch,
    CaseSearchInvalidation,

    // 新颖性
    NoveltyApplication,
    NoveltyRejection,

    // 创造性
    CreativityApplication,
    CreativityRejection,

    // 审查意见
    OpinionResponse,
    ArgumentDrafting,

    // 侵权分析
    LiteralInfringement,
    EquivalentInfringement,
    AllElementsRule,
    DoctrineOfEquivalents,
    ProsecutionHistoryEstoppel,
    LiteralInterpretation,

    // 权利要求解释
    ScopeClaimOnly,
    ScopeWithSpecification,
    ScopeWithProsecution,
    DefinitionClarity,
    SpecificationSupport,
    SupportDisclosure,

    // 无效宣告
    InvalidationGrounds,
    InvalidationDefense,
    InvalidationDrafting,

    // 审查标准
    ExaminationStandard,
    GuidelineComparison,
    GuidelineQuery,
    RuleInterpretation,
    SectionLookup,

    // 法律分析
    LegalQuery,
    LegalResearch,
    JudgmentPrediction,
    AddedSubjectMatter,
    EvidenceCollection,
    DefenseAnalysis,

    // 通用
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

    // 未知
    Unknown,
}

impl IntentType {
    /// 获取意图的关键词列表（用于关键词匹配）
    pub fn keywords(&self) -> &[&str] {
        match self {
            // 专利撰写
            Self::PatentDrafting => &[
                "撰写",
                "写专利",
                "申请文件",
                "专利申请",
                "专利撰写",
                "撰写专利",
                "撰写申请",
                "写一份专利",
            ],
            Self::ClaimDraftingStrategy => &[
                "权利要求撰写",
                "权利要求策略",
                "独立权利要求",
                "从属权利要求",
                "权利要求布局",
            ],
            Self::DefensiveDrafting => &["防御性撰写", "保护策略", "权利要求保护", "专利保护"],
            Self::BroadScopeProtection => &["宽保护", "大范围保护", "保护范围", "权利要求范围"],
            Self::ClaimAmendment => &["修改权利要求", "权利要求修改", "主动修改", "答复修改"],
            // 专利检索
            Self::PatentSearch => &[
                "检索",
                "搜索专利",
                "查专利",
                "专利检索",
                "现有技术检索",
                "检索相关专利",
                "查找专利",
            ],
            Self::CaseSearchInvalidation => &["无效案例", "无效决定", "复审无效", "口审记录"],
            // 新颖性
            Self::NoveltyApplication => &["新颖性", "新颖性分析", "判断新颖性"],
            Self::NoveltyRejection => &["新颖性驳回", "缺乏新颖性", "没有新颖性"],
            // 创造性
            Self::CreativityApplication => &["创造性", "创造性分析", "判断创造性", "显而易见"],
            Self::CreativityRejection => &["创造性驳回", "缺乏创造性", "没有创造性", "显而易见"],
            // 审查意见
            Self::OpinionResponse => &[
                "审查意见",
                "答复审查",
                "OA答复",
                "审查意见答复",
                "OA",
                "Office Action",
            ],
            Self::ArgumentDrafting => &["答辩", "答辩意见", "陈述意见", "意见陈述书"],
            // 侵权分析
            Self::LiteralInfringement => &["侵权", "字面侵权", "侵权分析"],
            Self::EquivalentInfringement => &["等同侵权", "等同替换", "实质性相同"],
            Self::AllElementsRule => &["全面覆盖", "全部要素", "全部技术特征"],
            Self::DoctrineOfEquivalents => &["等同原则", "等同判定", "等同特征"],
            Self::ProsecutionHistoryEstoppel => &["禁止反悔", "审查历史禁止反悔", "答辩历史"],
            Self::LiteralInterpretation => &["字面解释", "权利要求解释", "权利要求字面"],
            // 权利要求解释
            Self::ScopeClaimOnly => &["权利要求", "保护范围", "权利要求限定"],
            Self::ScopeWithSpecification => &["说明书解释", "说明书范围", "实施例"],
            Self::ScopeWithProsecution => &["审查档案", "审查过程", "审查记录"],
            Self::DefinitionClarity => &["定义清晰", "术语定义", "权利要求清晰"],
            Self::SpecificationSupport => &["说明书支持", "得到说明书支持", "超范围"],
            Self::SupportDisclosure => &["公开充分", "充分公开", "能够实现"],
            // 无效宣告
            Self::InvalidationGrounds => &["无效宣告", "无效理由", "专利无效", "无效请求"],
            Self::InvalidationDefense => &["无效答辩", "无效应对", "无效防御"],
            Self::InvalidationDrafting => &["撰写无效请求", "无效宣告请求书", "无效意见"],
            // 审查标准
            Self::ExaminationStandard => &["审查标准", "审查实务", "审查操作"],
            Self::GuidelineComparison => &["审查指南对比", "指南比较", "审查指南解读"],
            Self::GuidelineQuery => &["审查指南", "审查标准", "指南"],
            Self::RuleInterpretation => &["规则解释", "条款理解", "法条解释"],
            Self::SectionLookup => &["查法条", "条款查询", "第几条"],
            // 法律分析
            Self::LegalQuery => &["法律", "法条", "法律规定", "知识产权法"],
            Self::LegalResearch => &["法律研究", "案例研究", "法学研究", "判例"],
            Self::JudgmentPrediction => &["判决预测", "案件预测", "胜诉率"],
            Self::AddedSubjectMatter => &["超范围修改", "新增内容", "修改超范围"],
            Self::EvidenceCollection => &["证据收集", "举证", "收集证据"],
            Self::DefenseAnalysis => &["抗辩", "抗辩分析", "不侵权抗辩"],
            // 通用
            Self::CodeGeneration => &["写代码", "编程", "生成代码", "code"],
            Self::CodeReview => &["代码审查", "代码review", "review代码"],
            Self::CreativeWriting => &["创意写作", "写作", "文章"],
            Self::DataAnalysis => &["数据分析", "数据统计", "数据处理"],
            Self::ProblemSolving => &["解决问题", "方案", "办法"],
            Self::Emotional => &["心情", "情绪", "不开心", "开心"],
            Self::LifestyleService => &["生活", "美食", "旅游", "推荐"],
            Self::WeatherQuery => &["天气", "气温", "下雨"],
            Self::MapNavigation => &["地图", "导航", "路线"],
            Self::TrafficQuery => &["交通", "路况", "堵车"],
            Self::SystemControl => &["系统", "设置", "配置"],
            Self::CrimeAnalysis => &["犯罪", "刑事", "量刑"],
            Self::Unknown => &[],
        }
    }

    /// 从字符串名解析（兼容 Athena 的命名）
    pub fn from_athena_name(name: &str) -> Self {
        match name {
            "PATENT_DRAFTING" => Self::PatentDrafting,
            "CLAIM_DRAFTING_STRATEGY" => Self::ClaimDraftingStrategy,
            "PATENT_SEARCH" => Self::PatentSearch,
            "NOVELTY_APPLICATION" => Self::NoveltyApplication,
            "NOVELTY_REJECTION" => Self::NoveltyRejection,
            "CREATIVITY_APPLICATION" => Self::CreativityApplication,
            "CREATIVITY_REJECTION" => Self::CreativityRejection,
            "OPINION_RESPONSE" => Self::OpinionResponse,
            "ARGUMENT_DRAFTING" => Self::ArgumentDrafting,
            "LITERAL_INFRINGEMENT" => Self::LiteralInfringement,
            "EQUIVALENT_INFRINGEMENT" => Self::EquivalentInfringement,
            "ALL_ELEMENTS_RULE" => Self::AllElementsRule,
            "DOCTRINE_OF_EQUIVALENTS" => Self::DoctrineOfEquivalents,
            "INVALIDATION_GROUNDS" => Self::InvalidationGrounds,
            "INVALIDATION_DEFENSE" => Self::InvalidationDefense,
            "LEGAL_QUERY" => Self::LegalQuery,
            "LEGAL_RESEARCH" => Self::LegalResearch,
            "JUDGMENT_PREDICTION" => Self::JudgmentPrediction,
            "GUIDELINE_QUERY" => Self::GuidelineQuery,
            "GUIDELINE_COMPARISON" => Self::GuidelineComparison,
            "EXAMINATION_STANDARD" => Self::ExaminationStandard,
            "CODE_GENERATION" => Self::CodeGeneration,
            "DEFENSE_ANALYSIS" => Self::DefenseAnalysis,
            "EVIDENCE_COLLECTION" => Self::EvidenceCollection,
            "DEFENSIVE_DRAFTING" => Self::DefensiveDrafting,
            "BROAD_SCOPE_PROTECTION" => Self::BroadScopeProtection,
            "CLAIM_AMENDMENT" => Self::ClaimAmendment,
            "PROSECUTION_HISTORY_ESTOPPEL" => Self::ProsecutionHistoryEstoppel,
            "LITERAL_INTERPRETATION" => Self::LiteralInterpretation,
            "SCOPE_CLAIM_ONLY" => Self::ScopeClaimOnly,
            "SCOPE_WITH_SPECIFICATION" => Self::ScopeWithSpecification,
            "SCOPE_WITH_PROSECUTION" => Self::ScopeWithProsecution,
            "DEFINITION_CLARITY" => Self::DefinitionClarity,
            "SPECIFICATION_SUPPORT" => Self::SpecificationSupport,
            "SUPPORT_DISCLOSURE" => Self::SupportDisclosure,
            "CASE_SEARCH_INVALIDATION" => Self::CaseSearchInvalidation,
            "INVALIDATION_DRAFTING" => Self::InvalidationDrafting,
            "RULE_INTERPRETATION" => Self::RuleInterpretation,
            "SECTION_LOOKUP" => Self::SectionLookup,
            "ADDED_SUBJECT_MATTER" => Self::AddedSubjectMatter,
            "CODE_REVIEW" => Self::CodeReview,
            "CREATIVE_WRITING" => Self::CreativeWriting,
            "DATA_ANALYSIS" => Self::DataAnalysis,
            "PROBLEM_SOLVING" => Self::ProblemSolving,
            "EMOTIONAL" => Self::Emotional,
            "LIFESTYLE_SERVICE" => Self::LifestyleService,
            "WEATHER_QUERY" => Self::WeatherQuery,
            "MAP_NAVIGATION" => Self::MapNavigation,
            "TRAFFIC_QUERY" => Self::TrafficQuery,
            "SYSTEM_CONTROL" => Self::SystemControl,
            "CRIME_ANALYSIS" => Self::CrimeAnalysis,
            _ => Self::Unknown,
        }
    }

    /// 获取 Athena 兼容的名称
    pub fn to_athena_name(&self) -> &'static str {
        match self {
            Self::PatentDrafting => "PATENT_DRAFTING",
            Self::ClaimDraftingStrategy => "CLAIM_DRAFTING_STRATEGY",
            Self::DefensiveDrafting => "DEFENSIVE_DRAFTING",
            Self::BroadScopeProtection => "BROAD_SCOPE_PROTECTION",
            Self::ClaimAmendment => "CLAIM_AMENDMENT",
            Self::PatentSearch => "PATENT_SEARCH",
            Self::CaseSearchInvalidation => "CASE_SEARCH_INVALIDATION",
            Self::NoveltyApplication => "NOVELTY_APPLICATION",
            Self::NoveltyRejection => "NOVELTY_REJECTION",
            Self::CreativityApplication => "CREATIVITY_APPLICATION",
            Self::CreativityRejection => "CREATIVITY_REJECTION",
            Self::OpinionResponse => "OPINION_RESPONSE",
            Self::ArgumentDrafting => "ARGUMENT_DRAFTING",
            Self::LiteralInfringement => "LITERAL_INFRINGEMENT",
            Self::EquivalentInfringement => "EQUIVALENT_INFRINGEMENT",
            Self::AllElementsRule => "ALL_ELEMENTS_RULE",
            Self::DoctrineOfEquivalents => "DOCTRINE_OF_EQUIVALENTS",
            Self::ProsecutionHistoryEstoppel => "PROSECUTION_HISTORY_ESTOPPEL",
            Self::LiteralInterpretation => "LITERAL_INTERPRETATION",
            Self::ScopeClaimOnly => "SCOPE_CLAIM_ONLY",
            Self::ScopeWithSpecification => "SCOPE_WITH_SPECIFICATION",
            Self::ScopeWithProsecution => "SCOPE_WITH_PROSECUTION",
            Self::DefinitionClarity => "DEFINITION_CLARITY",
            Self::SpecificationSupport => "SPECIFICATION_SUPPORT",
            Self::SupportDisclosure => "SUPPORT_DISCLOSURE",
            Self::InvalidationGrounds => "INVALIDATION_GROUNDS",
            Self::InvalidationDefense => "INVALIDATION_DEFENSE",
            Self::InvalidationDrafting => "INVALIDATION_DRAFTING",
            Self::ExaminationStandard => "EXAMINATION_STANDARD",
            Self::GuidelineComparison => "GUIDELINE_COMPARISON",
            Self::GuidelineQuery => "GUIDELINE_QUERY",
            Self::RuleInterpretation => "RULE_INTERPRETATION",
            Self::SectionLookup => "SECTION_LOOKUP",
            Self::LegalQuery => "LEGAL_QUERY",
            Self::LegalResearch => "LEGAL_RESEARCH",
            Self::JudgmentPrediction => "JUDGMENT_PREDICTION",
            Self::AddedSubjectMatter => "ADDED_SUBJECT_MATTER",
            Self::EvidenceCollection => "EVIDENCE_COLLECTION",
            Self::DefenseAnalysis => "DEFENSE_ANALYSIS",
            Self::CodeGeneration => "CODE_GENERATION",
            Self::CodeReview => "CODE_REVIEW",
            Self::CreativeWriting => "CREATIVE_WRITING",
            Self::DataAnalysis => "DATA_ANALYSIS",
            Self::ProblemSolving => "PROBLEM_SOLVING",
            Self::Emotional => "EMOTIONAL",
            Self::LifestyleService => "LIFESTYLE_SERVICE",
            Self::WeatherQuery => "WEATHER_QUERY",
            Self::MapNavigation => "MAP_NAVIGATION",
            Self::TrafficQuery => "TRAFFIC_QUERY",
            Self::SystemControl => "SYSTEM_CONTROL",
            Self::CrimeAnalysis => "CRIME_ANALYSIS",
            Self::Unknown => "UNKNOWN",
        }
    }
}
