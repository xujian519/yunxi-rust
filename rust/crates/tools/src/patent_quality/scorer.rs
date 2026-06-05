use crate::patent_quality::dimensions::{
    scorer_assess_claims_quality, scorer_assess_language_quality, scorer_assess_legal_quality,
    scorer_assess_specification_quality, scorer_check_completeness,
};
use crate::patent_quality::recommendations::scorer_generate_recommendations;
use crate::patent_quality::rules::scorer_apply_rules;
use crate::patent_quality::types::{scorer_generate_comparison, scorer_get_quality_level};
use crate::patent_quality::types::{QualityScorerInput, ScorerOutput, ScorerQualityScores};

pub(crate) fn scorer_score(input: &QualityScorerInput) -> ScorerOutput {
    let completeness = scorer_check_completeness(input);

    let claims_scores = scorer_assess_claims_quality(input);
    let spec_scores = scorer_assess_specification_quality(input);
    let lang_scores = scorer_assess_language_quality(input);
    let legal_scores = scorer_assess_legal_quality(input);

    let quality_scores = ScorerQualityScores {
        claims: claims_scores,
        specification: spec_scores,
        language: lang_scores,
        legal: legal_scores,
    };

    let overall = completeness * 0.25
        + quality_scores.claims.overall * 0.3
        + quality_scores.specification.overall * 0.25
        + quality_scores.language.overall * 0.1
        + quality_scores.legal.overall * 0.1;

    let issues = scorer_apply_rules(input, input.check_level);
    let recommendations = scorer_generate_recommendations(input, &issues, &quality_scores);
    let comparison = scorer_generate_comparison(overall, &input.patent_type);
    let quality_level = scorer_get_quality_level(overall).to_string();

    ScorerOutput {
        completeness_score: completeness,
        quality_scores,
        overall_quality: overall,
        quality_level,
        issues,
        recommendations,
        comparison,
    }
}

/// 质量评分工具。
///
/// 对权利要求书和说明书进行四维评分（权利要求/说明书/语言/法律），
/// 应用 12 条质量规则，生成改进建议和百分位排名。纯规则型，无需 LLM。
pub fn execute_quality_scorer(input: &QualityScorerInput) -> Result<serde_json::Value, String> {
    let output = scorer_score(input);
    serde_json::to_value(output).map_err(|e| format!("序列化失败: {e}"))
}
