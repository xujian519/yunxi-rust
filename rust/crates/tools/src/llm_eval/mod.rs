use serde::Serialize;
use serde_json::Value;

mod faithfulness;
mod g_eval;
mod self_consistency;

/// LLM-as-Judge 评估引擎。
///
/// 参照 DeepEval 的 G-Eval 模式，使用 LLM 对输入进行结构化评估。
pub struct LLMJudgeEngine {
    model: String,
    max_tokens: u32,
}

impl LLMJudgeEngine {
    pub fn new(model: String, max_tokens: u32) -> Self {
        Self { model, max_tokens }
    }

    pub fn with_default() -> Self {
        Self {
            model: "deepseek-v4-pro-4-6".to_string(),
            max_tokens: 4000,
        }
    }
}

/// G-Eval 评估结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GEvalResult {
    pub score: f64,
    pub reasoning: String,
    pub confidence: f64,
    pub criteria_met: Vec<String>,
    pub criteria_missing: Vec<String>,
}

/// Faithfulness 评估结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaithfulnessResult {
    pub faithfulness_score: f64,
    pub hallucinations: Vec<Hallucination>,
    pub supported_claims: Vec<String>,
    pub source_alignment: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hallucination {
    pub claim: String,
    pub severity: String,
    pub explanation: String,
}

/// Self-Consistency 评估结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfConsistencyResult {
    pub consistency_score: f64,
    pub majority_answer: String,
    pub answer_distribution: Vec<AnswerVariant>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerVariant {
    pub answer: String,
    pub count: usize,
    pub percentage: f64,
}

/// 统一评估输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LLMEvalOutput {
    pub evaluation_type: String,
    pub overall_score: f64,
    pub quality_level: String,
    pub details: Value,
    pub timestamp: String,
}

pub fn quality_level_from_score(score: f64) -> &'static str {
    if score >= 90.0 {
        "excellent"
    } else if score >= 75.0 {
        "good"
    } else if score >= 60.0 {
        "fair"
    } else {
        "poor"
    }
}
