use std::collections::HashSet;

use crate::patent_quality::types::{
    calculate_keyword_overlap, extract_keywords, QualityScorerInput, ScorerDimensionScores,
    ScorerLangScores, ScorerLegalScores, ScorerSpecScores,
};

/// 完整性评分（满分 100）。
/// 权重设计依据：独立权利要求（20）> 说明书四要素（各 12）> 从属权利要求（10）>
/// 多权利要求（5）= 附图（5）= 标题（5）。反映专利法 A26.2-A26.4 的核心要求。
pub(crate) fn scorer_check_completeness(input: &QualityScorerInput) -> f64 {
    let mut score: f64 = 0.0;
    if !input.invention_title.is_empty() {
        score += 5.0; // 标题存在
    }
    if !input.claims.is_empty() {
        let has_independent = input.claims.iter().any(|c| c.r#type == "independent");
        if has_independent {
            score += 20.0; // 独立权利要求是专利保护的核心
        }
        if input.claims.len() >= 2 {
            score += 10.0; // 至少 1 项从属权利要求，细化保护范围
        }
        if input.claims.len() >= 5 {
            score += 5.0; // 多层次权利要求布局
        }
    }
    let spec = &input.specification;
    // 说明书四要素各 12 分，合计 48 分；对应专利法第 26 条第 3 款
    if let Some(f) = &spec.technical_field {
        if f.len() > 10 {
            score += 12.0;
        }
    }
    if let Some(b) = &spec.background_art {
        if b.len() > 20 {
            score += 12.0;
        }
    }
    if let Some(ic) = &spec.invention_content {
        if ic.len() > 50 {
            score += 12.0;
        }
    }
    if let Some(e) = &spec.embodiment {
        if e.len() > 100 {
            score += 12.0;
        }
    }
    if !input.drawings.is_empty() {
        score += 12.0; // 附图对理解发明至关重要
    }
    score.min(100.0)
}

pub(crate) fn scorer_assess_claims_quality(input: &QualityScorerInput) -> ScorerDimensionScores {
    if input.claims.is_empty() {
        return ScorerDimensionScores {
            clarity: 0.0,
            support: 0.0,
            breadth: 0.0,
            protection_scope: 0.0,
            overall: 0.0,
        };
    }

    let mut clarity = 100.0;
    let mut support = 100.0;
    let mut breadth = 100.0;
    let mut protection_scope = 100.0;

    let avg_len: f64 = input
        .claims
        .iter()
        .map(|c| c.content.len() as f64)
        .sum::<f64>()
        / input.claims.len() as f64;
    if avg_len > 200.0 {
        clarity -= 15.0;
    }
    if avg_len > 300.0 {
        clarity -= 15.0;
    }
    if avg_len > 400.0 {
        clarity -= 20.0;
    }

    let complex = input
        .claims
        .iter()
        .filter(|c| c.content.matches('，').count() > 5)
        .count() as f64;
    clarity -= (complex / input.claims.len() as f64) * 20.0;

    let dep_count = input
        .claims
        .iter()
        .filter(|c| c.r#type == "dependent")
        .count() as f64;
    let ind_count = input
        .claims
        .iter()
        .filter(|c| c.r#type == "independent")
        .count() as f64;
    if ind_count > 0.0 {
        let ratio = dep_count / ind_count;
        if ratio < 1.0 {
            support -= 30.0;
        }
        if ratio < 2.0 {
            support -= 15.0;
        }
        if ratio >= 3.0 {
            support += 10.0;
        }
    }

    let claims_len = input.claims.len() as f64;
    if claims_len < 2.0 {
        breadth -= 30.0;
    }
    if claims_len < 3.0 {
        breadth -= 15.0;
    }
    if claims_len >= 5.0 {
        breadth += 10.0;
    }
    if claims_len >= 10.0 {
        breadth += 10.0;
    }

    #[allow(clippy::float_cmp)]
    if ind_count == 1.0 {
        protection_scope = 100.0;
    } else if ind_count > 3.0 {
        protection_scope -= 20.0;
    }

    let overall = (clarity + support + breadth + protection_scope) / 4.0;
    ScorerDimensionScores {
        clarity,
        support,
        breadth,
        protection_scope,
        overall,
    }
}

pub(crate) fn scorer_assess_specification_quality(input: &QualityScorerInput) -> ScorerSpecScores {
    let spec = &input.specification;
    let mut clarity = 100.0;
    let mut sufficiency = 100.0;
    let mut consistency = 100.0;
    let mut supportiveness = 100.0;

    let field_len = spec.technical_field.as_ref().map_or(0, |s| s.trim().len());
    let ic_len = spec
        .invention_content
        .as_ref()
        .map_or(0, |s| s.trim().len());
    let emb_len = spec.embodiment.as_ref().map_or(0, |s| s.trim().len());
    let bg_len = spec.background_art.as_ref().map_or(0, |s| s.trim().len());

    if field_len < 20 {
        clarity -= 20.0;
    }
    if ic_len < 50 {
        clarity -= 20.0;
    }
    if emb_len < 100 {
        clarity -= 20.0;
    }
    if emb_len < 200 {
        clarity -= 20.0;
    }

    if field_len == 0 {
        sufficiency -= 25.0;
    }
    if bg_len == 0 {
        sufficiency -= 25.0;
    }
    if ic_len == 0 {
        sufficiency -= 25.0;
    }
    if emb_len == 0 {
        sufficiency -= 25.0;
    }

    if field_len > 0 && ic_len > 0 {
        // SAFETY: field_len > 0 and ic_len > 0 guarantee Some.
        let field_kw = extract_keywords(
            spec.technical_field
                .as_ref()
                .expect("field_len > 0 guarantees Some"),
        );
        let content_kw = extract_keywords(
            spec.invention_content
                .as_ref()
                .expect("ic_len > 0 guarantees Some"),
        );
        let overlap = calculate_keyword_overlap(&field_kw, &content_kw);
        if overlap < 0.2 {
            consistency -= 30.0;
        }
    } else {
        consistency -= 30.0;
    }

    if emb_len > 0 && !input.claims.is_empty() {
        let claim_terms: HashSet<std::string::String> = input
            .claims
            .iter()
            .flat_map(|c| extract_keywords(&c.content))
            .collect();
        // SAFETY: emb_len > 0 guarantees Some.
        let emb_text = spec
            .embodiment
            .as_ref()
            .expect("emb_len > 0 guarantees Some");
        let supported = claim_terms
            .iter()
            .filter(|t| emb_text.contains(t.as_str()))
            .count();
        if !claim_terms.is_empty() {
            supportiveness = (supported as f64 / claim_terms.len() as f64) * 100.0;
        }
    } else if !input.claims.is_empty() {
        supportiveness = 0.0;
    }

    let overall = (clarity + sufficiency + consistency + supportiveness) / 4.0;
    ScorerSpecScores {
        clarity,
        sufficiency,
        consistency,
        supportiveness,
        overall,
    }
}

pub(crate) fn scorer_assess_language_quality(input: &QualityScorerInput) -> ScorerLangScores {
    let mut grammar = 100.0;
    let mut terminology = 100.0;
    let mut accuracy = 100.0;

    for claim in &input.claims {
        if claim.content.contains("。。") || claim.content.contains("，，") {
            grammar -= 10.0;
        }
        if claim.content.contains("。") || claim.content.contains("、，") {
            grammar -= 5.0;
        }
        if !claim.content.ends_with('。') {
            grammar -= 5.0;
        }
    }

    let tech_terms = ["装置", "方法", "系统", "设备", "模块", "单元", "组件"];
    let has_terms = input
        .claims
        .iter()
        .any(|c| tech_terms.iter().any(|t| c.content.contains(t)));
    if !has_terms {
        terminology -= 40.0;
    }

    let vague = ["大约", "左右", "可能", "也许", "大概"];
    for claim in &input.claims {
        for term in &vague {
            if claim.content.contains(term) {
                accuracy -= 10.0;
            }
        }
    }

    let overall = (grammar + terminology + accuracy) / 3.0;
    ScorerLangScores {
        grammar,
        terminology,
        accuracy,
        overall,
    }
}

pub(crate) fn scorer_assess_legal_quality(input: &QualityScorerInput) -> ScorerLegalScores {
    let mut formality = 100.0;
    let mut patentability = 80.0;
    let mut risk_level = "low".to_string();

    if input.invention_title.is_empty() {
        formality -= 20.0;
    }
    if input.claims.is_empty() {
        formality -= 50.0;
    } else if input.claims[0].r#type != "independent" {
        formality -= 30.0;
    }

    let spec = &input.specification;
    if spec.technical_field.is_none()
        || spec.background_art.is_none()
        || spec.invention_content.is_none()
    {
        patentability -= 30.0;
    }
    if spec.embodiment.as_ref().map_or(0, String::len) < 100 {
        patentability -= 20.0;
    }

    if formality < 70.0 || patentability < 60.0 {
        risk_level = "high".to_string();
    } else if formality < 85.0 || patentability < 75.0 {
        risk_level = "medium".to_string();
    }

    #[allow(clippy::manual_midpoint)]
    let overall = (formality + patentability) / 2.0;
    ScorerLegalScores {
        formality,
        patentability,
        risk_level,
        overall,
    }
}
