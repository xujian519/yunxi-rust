use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ResponseStrategy;

/// 用户偏好。
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserPreference {
    Aggressive,
    Moderate,
    Conservative,
}

/// 评分器驳回理由输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreRejectionReason {
    #[serde(rename = "type")]
    pub(crate) rejection_type: String,
    pub(crate) severity: String,
    #[serde(default)]
    pub(crate) suggested_response: Option<String>,
}

/// 评分器历史案例输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreHistoricalCase {
    #[allow(dead_code)]
    pub(crate) id: String, // 保留原因: 历史案例唯一标识符，预留给调试和日志使用
    pub(crate) strategy: String,
    pub(crate) outcome: String,
    #[serde(default)]
    pub(crate) rejection_reasons: Vec<ScoreRejectionReason>,
    #[serde(default)]
    pub(crate) technical_field: String,
    #[serde(default)]
    pub(crate) granted_claims: Vec<u32>,
}

/// 评分器引用文献输入。
#[derive(Debug, Deserialize)]
pub struct ScoreCitedReference {
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) publication_number: String, // 保留原因: 引用文献公开号，预留给未来详细分析使用
}

/// 策略评分器输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyScoreInput {
    pub rejection_reasons: Vec<ScoreRejectionReason>,
    #[serde(default)]
    pub rejection_types: Vec<String>,
    #[serde(default)]
    pub affected_claims: Vec<u32>,
    #[serde(default)]
    pub cited_references: Vec<ScoreCitedReference>,
    #[serde(default)]
    pub patent_title: String,
    #[serde(default)]
    pub user_preference: Option<UserPreference>,
    #[serde(default)]
    pub risk_tolerance: Option<f64>,
    #[serde(default)]
    pub historical_cases: Vec<ScoreHistoricalCase>,
}

/// 评分详情。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScoreDetails {
    pub base_score: f64,
    pub rejection_match: f64,
    pub historical_success: f64,
    pub risk_adjustment: f64,
    pub user_preference: f64,
}

/// 策略评分结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StrategyScore {
    pub strategy: String,
    pub score: f64,
    pub details: ScoreDetails,
}

/// 评分器输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StrategyOutput {
    scores: Vec<StrategyScore>,
    recommended_strategy: String,
    recommended_score: f64,
}

// 加权系数。
const W_BASE: f64 = 0.3;
const W_REJECTION: f64 = 0.25;
const W_HISTORY: f64 = 0.2;
const W_RISK: f64 = 0.15;
const W_USER: f64 = 0.1;

/// 基础偏好分数表。
#[allow(clippy::match_same_arms)]
pub(crate) fn base_preference_score(pref: UserPreference, strategy: ResponseStrategy) -> f64 {
    match (pref, strategy) {
        (UserPreference::Aggressive, ResponseStrategy::Argue) => 85.0,
        (UserPreference::Aggressive, ResponseStrategy::Amend) => 60.0,
        (UserPreference::Aggressive, ResponseStrategy::Both) => 75.0,
        (UserPreference::Aggressive, ResponseStrategy::Abandon) => 20.0,
        (UserPreference::Aggressive, ResponseStrategy::Appeal) => 70.0,
        (UserPreference::Moderate, ResponseStrategy::Argue) => 70.0,
        (UserPreference::Moderate, ResponseStrategy::Amend) => 75.0,
        (UserPreference::Moderate, ResponseStrategy::Both) => 80.0,
        (UserPreference::Moderate, ResponseStrategy::Abandon) => 30.0,
        (UserPreference::Moderate, ResponseStrategy::Appeal) => 50.0,
        (UserPreference::Conservative, ResponseStrategy::Argue) => 50.0,
        (UserPreference::Conservative, ResponseStrategy::Amend) => 85.0,
        (UserPreference::Conservative, ResponseStrategy::Both) => 70.0,
        (UserPreference::Conservative, ResponseStrategy::Abandon) => 50.0,
        (UserPreference::Conservative, ResponseStrategy::Appeal) => 40.0,
    }
}

/// 用户偏好分数。
#[allow(clippy::match_same_arms)]
pub(crate) fn user_preference_score(pref: UserPreference, strategy: ResponseStrategy) -> f64 {
    match (pref, strategy) {
        (UserPreference::Aggressive, ResponseStrategy::Argue) => 100.0,
        (UserPreference::Aggressive, ResponseStrategy::Both) => 85.0,
        (UserPreference::Aggressive, ResponseStrategy::Appeal) => 80.0,
        (UserPreference::Aggressive, ResponseStrategy::Amend) => 60.0,
        (UserPreference::Aggressive, ResponseStrategy::Abandon) => 30.0,
        (UserPreference::Moderate, ResponseStrategy::Both) => 100.0,
        (UserPreference::Moderate, ResponseStrategy::Amend) => 90.0,
        (UserPreference::Moderate, ResponseStrategy::Argue) => 80.0,
        (UserPreference::Moderate, ResponseStrategy::Appeal) => 60.0,
        (UserPreference::Moderate, ResponseStrategy::Abandon) => 40.0,
        (UserPreference::Conservative, ResponseStrategy::Amend) => 100.0,
        (UserPreference::Conservative, ResponseStrategy::Both) => 80.0,
        (UserPreference::Conservative, ResponseStrategy::Argue) => 50.0,
        (UserPreference::Conservative, ResponseStrategy::Abandon) => 60.0,
        (UserPreference::Conservative, ResponseStrategy::Appeal) => 40.0,
    }
}

/// 驳回类型与最佳策略映射。
#[allow(clippy::match_same_arms)]
pub(crate) fn best_strategies_for_rejection(rejection_type: &str) -> Vec<ResponseStrategy> {
    match rejection_type {
        "novelty" => vec![
            ResponseStrategy::Amend,
            ResponseStrategy::Argue,
            ResponseStrategy::Both,
        ],
        "inventiveness" => vec![
            ResponseStrategy::Argue,
            ResponseStrategy::Amend,
            ResponseStrategy::Both,
        ],
        "utility" => vec![ResponseStrategy::Amend, ResponseStrategy::Abandon],
        "support" => vec![ResponseStrategy::Amend, ResponseStrategy::Argue],
        "clarity" => vec![ResponseStrategy::Amend],
        "scope" => vec![ResponseStrategy::Amend, ResponseStrategy::Argue],
        "amendmentScope" | "amendment_scope" => {
            vec![ResponseStrategy::Amend, ResponseStrategy::Abandon]
        }
        "unity" => vec![ResponseStrategy::Amend, ResponseStrategy::Argue],
        "formality" => vec![ResponseStrategy::Amend],
        _ => vec![ResponseStrategy::Argue, ResponseStrategy::Amend],
    }
}

/// 计算驳回理由匹配度。
pub(crate) fn calculate_rejection_match(
    rejection_reasons: &[ScoreRejectionReason],
    strategy: ResponseStrategy,
) -> f64 {
    if rejection_reasons.is_empty() {
        return 50.0;
    }

    let mut total = 0.0;
    let mut weight_sum = 0.0;

    for reason in rejection_reasons {
        let weight = match reason.severity.as_str() {
            "high" => 1.5,
            "medium" => 1.0,
            _ => 0.5,
        };
        let suggested_match = match (&reason.suggested_response, strategy) {
            (Some(s), _) if s == "abandon" => {
                strategy == ResponseStrategy::Abandon || strategy == ResponseStrategy::Appeal
            }
            (Some(s), _) if s == "both" => {
                strategy == ResponseStrategy::Both
                    || strategy == ResponseStrategy::Argue
                    || strategy == ResponseStrategy::Amend
            }
            (Some(s), _) => s == strategy.as_str() || strategy == ResponseStrategy::Both,
            _ => false,
        };
        let suitable = best_strategies_for_rejection(&reason.rejection_type).contains(&strategy);
        let score = (if suggested_match { 80.0 } else { 40.0 }) * 0.6
            + (if suitable { 90.0 } else { 50.0 }) * 0.4;

        total += score * weight;
        weight_sum += weight;
    }

    if weight_sum > 0.0 {
        total / weight_sum
    } else {
        50.0
    }
}

/// 计算风险调整。
pub(crate) fn calculate_risk_adjustment(
    rejection_reasons: &[ScoreRejectionReason],
    affected_claims: &[u32],
    strategy: ResponseStrategy,
    risk_tolerance: f64,
) -> f64 {
    let mut risk_score = 100.0;

    let high_severity = rejection_reasons
        .iter()
        .filter(|r| r.severity == "high")
        .count();
    if high_severity > 2 {
        risk_score -= 20.0;
    } else if high_severity > 0 {
        risk_score -= 10.0;
    }

    if affected_claims.len() > 5 {
        risk_score -= 15.0;
    } else if affected_claims.len() > 3 {
        risk_score -= 8.0;
    }

    risk_score -= match strategy {
        ResponseStrategy::Abandon => 30.0,
        ResponseStrategy::Appeal => 25.0,
        ResponseStrategy::Both => 10.0,
        _ => 0.0,
    };

    risk_score += (risk_tolerance - 0.5) * 20.0;
    risk_score.clamp(20.0, 100.0)
}

/// 计算案例相似度。
#[allow(clippy::cast_precision_loss)]
pub(crate) fn calculate_case_similarity(
    rejection_types: &[String],
    affected_claims: &[u32],
    cited_count: usize,
    patent_title: &str,
    case: &ScoreHistoricalCase,
) -> f64 {
    let mut similarity = 0.0;
    let mut factors = 0.0;

    // 驳回类型交集
    let parse_types: std::collections::HashSet<&str> =
        rejection_types.iter().map(String::as_str).collect();
    let case_types: std::collections::HashSet<&str> = case
        .rejection_reasons
        .iter()
        .map(|r| r.rejection_type.as_str())
        .collect();
    let intersection = parse_types
        .iter()
        .filter(|t| case_types.contains(*t))
        .count();
    let union_size = parse_types.union(&case_types).count().max(1);
    similarity += (intersection as f64 / union_size as f64) * 0.4;
    factors += 0.4;

    // 权利要求数差异
    let claim_diff = (affected_claims.len() as f64 - case.granted_claims.len() as f64).abs();
    similarity += (1.0 - claim_diff / 10.0).max(0.0) * 0.2;
    factors += 0.2;

    // 引用数差异
    let ref_diff = (cited_count as f64 - case.rejection_reasons.len() as f64).abs();
    similarity += (1.0 - ref_diff / 5.0).max(0.0) * 0.2;
    factors += 0.2;

    // 标题匹配
    if !patent_title.is_empty() && !case.technical_field.is_empty() {
        let title_words: Vec<&str> = patent_title.split_whitespace().collect();
        let field_words: Vec<&str> = case.technical_field.split_whitespace().collect();
        let word_match = title_words
            .iter()
            .filter(|w| field_words.contains(w))
            .count();
        similarity += (word_match as f64 / 3.0).min(1.0) * 0.2;
        factors += 0.2;
    }

    if factors > 0.0 {
        similarity / factors
    } else {
        0.0
    }
}

/// 计算历史成功率。
#[allow(clippy::cast_precision_loss)]
pub(crate) fn calculate_historical_success(
    input: &StrategyScoreInput,
    strategy: ResponseStrategy,
) -> f64 {
    if input.historical_cases.is_empty() {
        return 60.0;
    }

    let risk_tolerance = input.risk_tolerance.unwrap_or(0.5);
    let relevant: Vec<&ScoreHistoricalCase> = input
        .historical_cases
        .iter()
        .filter(|c| {
            calculate_case_similarity(
                &input.rejection_types,
                &input.affected_claims,
                input.cited_references.len(),
                &input.patent_title,
                c,
            ) >= 0.6
        })
        .collect();

    if relevant.is_empty() {
        return 60.0;
    }

    let strategy_cases: Vec<&&ScoreHistoricalCase> = relevant
        .iter()
        .filter(|c| c.strategy == strategy.as_str())
        .collect();

    if strategy_cases.is_empty() {
        return 60.0;
    }

    let success_count = strategy_cases
        .iter()
        .filter(|c| c.outcome == "success" || c.outcome == "partial_success")
        .count();

    let base_rate = (success_count as f64 / strategy_cases.len() as f64) * 100.0;

    // 根据风险容忍度调整历史成功率预期
    // risk_tolerance 范围 0.0~1.0，越高越激进
    if risk_tolerance > 0.7 {
        // 激进：提高预期成功率
        (base_rate * 1.1).min(100.0)
    } else if risk_tolerance < 0.3 {
        // 保守：降低预期成功率
        base_rate * 0.9
    } else {
        base_rate
    }
}

/// 评估所有策略。
pub(crate) fn evaluate_strategies(input: &StrategyScoreInput) -> Vec<StrategyScore> {
    let pref = input.user_preference.unwrap_or(UserPreference::Moderate);
    let risk = input.risk_tolerance.unwrap_or(0.5);

    let mut scores: Vec<StrategyScore> = ResponseStrategy::all()
        .iter()
        .map(|&strategy| {
            let base = base_preference_score(pref, strategy);
            let rej_match = calculate_rejection_match(&input.rejection_reasons, strategy);
            let hist_success = calculate_historical_success(input, strategy);
            let risk_adj = calculate_risk_adjustment(
                &input.rejection_reasons,
                &input.affected_claims,
                strategy,
                risk,
            );
            let user_pref = user_preference_score(pref, strategy);

            let total = base * W_BASE
                + rej_match * W_REJECTION
                + hist_success * W_HISTORY
                + risk_adj * W_RISK
                + user_pref * W_USER;

            StrategyScore {
                strategy: strategy.as_str().to_string(),
                score: total.clamp(0.0, 100.0),
                details: ScoreDetails {
                    base_score: base,
                    rejection_match: rej_match,
                    historical_success: hist_success,
                    risk_adjustment: risk_adj,
                    user_preference: user_pref,
                },
            }
        })
        .collect();

    scores.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scores
}

/// 执行策略评分。
pub fn execute_strategy_score(input: &StrategyScoreInput) -> Result<Value, String> {
    let scores = evaluate_strategies(input);

    let recommended = scores.first();
    let output = StrategyOutput {
        recommended_strategy: recommended.map_or(String::new(), |s| s.strategy.clone()),
        recommended_score: recommended.map_or(0.0, |s| s.score),
        scores,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}
