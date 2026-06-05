use crate::patent_quality::types::{
    QualityScorerInput, ScorerIssue, ScorerQualityScores, ScorerRecommendation,
};

pub(crate) fn scorer_generate_recommendations(
    input: &QualityScorerInput,
    issues: &[ScorerIssue],
    scores: &ScorerQualityScores,
) -> Vec<ScorerRecommendation> {
    let mut recs = Vec::new();

    for issue in issues {
        recs.push(ScorerRecommendation {
            area: issue.category.clone(),
            priority: if issue.severity == "critical" || issue.severity == "high" {
                "high"
            } else {
                "medium"
            }
            .to_string(),
            current: issue.description.clone(),
            suggested: issue.suggestion.clone(),
            rationale: format!(
                "根据规则{}，{}",
                issue.rule_reference.as_deref().unwrap_or(""),
                issue.description
            ),
            expected_impact: None,
        });
    }

    if scores.claims.breadth < 80.0 {
        recs.push(ScorerRecommendation {
            area: "权利要求".to_string(),
            priority: "medium".to_string(),
            current: format!("权利要求数量为{}项", input.claims.len()),
            suggested: "建议增加从属权利要求，形成多层次保护".to_string(),
            rationale: "多层次保护可以提高专利的稳定性和抗无效能力".to_string(),
            expected_impact: Some("预计提高保护范围得分10-20分".to_string()),
        });
    }

    if scores.specification.supportiveness < 80.0 {
        recs.push(ScorerRecommendation {
            area: "说明书".to_string(),
            priority: "high".to_string(),
            current: "说明书对权利要求的支持不足".to_string(),
            suggested: "在具体实施方式中详细描述权利要求中的技术特征".to_string(),
            rationale: "A26.4要求权利要求应当得到说明书的支持".to_string(),
            expected_impact: Some("预计提高支持性得分20-30分".to_string()),
        });
    }

    if scores.language.accuracy < 85.0 {
        recs.push(ScorerRecommendation {
            area: "语言表达".to_string(),
            priority: "medium".to_string(),
            current: "存在模糊或不精确的表达".to_string(),
            suggested: "使用精确的技术术语，避免模糊词汇".to_string(),
            rationale: "精确的表达有助于明确保护范围".to_string(),
            expected_impact: Some("预计提高表达准确性得分10-15分".to_string()),
        });
    }

    recs
}
