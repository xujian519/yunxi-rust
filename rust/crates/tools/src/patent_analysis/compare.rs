// compare.rs - 语义对比工具 (SemanticCompare) 及共享文本处理辅助函数

use serde::Deserialize;
use serde_json::{json, Value};

pub(crate) const STOPWORDS: &[&str] = &[
    "的", "了", "在", "是", "和", "与", "或", "中", "对", "为", "以", "及", "等", "被", "将", "从",
    "到", "由", "其", "该", "所述", "可", "能", "所", "有", "一种", "包括", "包含", "具有", "通过",
    "采用", "利用", "基于", "根据",
];

pub(crate) const HIGH_SIMILARITY_THRESHOLD: f64 = 0.7;
pub(crate) const MEDIUM_SIMILARITY_THRESHOLD: f64 = 0.4;
pub(crate) const EQUIVALENT_THRESHOLD: f64 = 0.6;
pub(crate) const MIN_OVERLAP_RATIO: f64 = 0.3;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareDoc {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub abstract_text: Option<String>,
    #[serde(default)]
    pub claims: Option<Vec<String>>,
    #[serde(default)]
    pub features: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticCompareInput {
    pub target: CompareDoc,
    pub prior_art: CompareDoc,
    #[serde(default)]
    pub compare_mode: Option<String>,
    #[serde(default)]
    pub weights: Option<Value>,
}

pub fn semantic_compare(input: SemanticCompareInput) -> Result<Value, String> {
    let mode = input.compare_mode.unwrap_or_else(|| "lexical".to_string());

    match mode.as_str() {
        "lexical" | "auto" => {
            compare_lexical(&input.target, &input.prior_art, input.weights.as_ref())
        }
        "embedding" => compare_embedding_stub(&input.target, &input.prior_art),
        "marg_lite" => compare_marg_stub(&input.target, &input.prior_art),
        _ => Err(format!("unknown compare_mode: {mode}")),
    }
}

pub(crate) fn compute_jaccard(text1: &str, text2: &str) -> f64 {
    if text1.is_empty() || text2.is_empty() {
        return 0.0;
    }

    let words1 = extract_keywords(text1);
    let words2 = extract_keywords(text2);

    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }

    let set1: std::collections::HashSet<_> = words1.iter().collect();
    let set2: std::collections::HashSet<_> = words2.iter().collect();

    let intersection: std::collections::HashSet<_> = set1.intersection(&set2).copied().collect();
    let union: std::collections::HashSet<_> = set1.union(&set2).copied().collect();

    if union.is_empty() {
        0.0
    } else {
        #[allow(clippy::cast_precision_loss)]
        {
            intersection.len() as f64 / union.len() as f64
        }
    }
}

pub(crate) fn extract_keywords(text: &str) -> Vec<String> {
    let separators = [",", "、", "。", ";", " ", "\t", "\n", "，", "；", "\r"];
    let mut result = String::from(text);
    for sep in &separators {
        result = result.replace(sep, " ");
    }

    result
        .split_whitespace()
        .filter(|s| s.len() >= 2 && !STOPWORDS.contains(s))
        .map(std::string::ToString::to_string)
        .collect()
}

#[allow(clippy::too_many_lines, clippy::unnecessary_wraps)]
fn compare_lexical(
    target: &CompareDoc,
    prior_art: &CompareDoc,
    custom_weights: Option<&Value>,
) -> Result<Value, String> {
    // 默认权重: title 0.2, abstract 0.3, claims 0.3, features 0.2
    let mut weights = [0.2, 0.3, 0.3, 0.2];

    if let Some(custom) = custom_weights {
        if let Some(w) = custom.get("title") {
            weights[0] = w.as_f64().unwrap_or(0.2);
        }
        if let Some(w) = custom.get("abstract") {
            weights[1] = w.as_f64().unwrap_or(0.3);
        }
        if let Some(w) = custom.get("claims") {
            weights[2] = w.as_f64().unwrap_or(0.3);
        }
        if let Some(w) = custom.get("features") {
            weights[3] = w.as_f64().unwrap_or(0.2);
        }
    }

    let mut total_score = 0.0;
    let mut dimensions = json!({});

    // Title比较
    if let (Some(t1), Some(t2)) = (&target.title, &prior_art.title) {
        let jaccard = compute_jaccard(t1, t2);
        let score = jaccard * weights[0];
        total_score += score;
        dimensions["title"] = json!({
            "score": score,
            "jaccard": jaccard,
            "details": format!("标题相似度: {:.2}", jaccard)
        });
    }

    // Abstract比较
    if let (Some(a1), Some(a2)) = (&target.abstract_text, &prior_art.abstract_text) {
        let jaccard = compute_jaccard(a1, a2);
        let score = jaccard * weights[1];
        total_score += score;
        dimensions["abstract"] = json!({
            "score": score,
            "jaccard": jaccard
        });
    }

    // Claims比较
    if let (Some(c1), Some(c2)) = (&target.claims, &prior_art.claims) {
        let claim_text1 = c1.join(" ");
        let claim_text2 = c2.join(" ");
        let jaccard = compute_jaccard(&claim_text1, &claim_text2);
        let score = jaccard * weights[2];
        total_score += score;

        // 权利要求逐项对比
        let mut element_matches = Vec::new();
        for (i, claim1) in c1.iter().enumerate() {
            for claim2 in c2 {
                let elem_jaccard = compute_jaccard(claim1, claim2);
                if elem_jaccard > 0.3 {
                    element_matches.push(json!({
                        "target_claim": i + 1,
                        "similarity": elem_jaccard
                    }));
                }
            }
        }

        dimensions["claims"] = json!({
            "score": score,
            "jaccard": jaccard,
            "element_matches": element_matches
        });
    }

    // Features比较
    if let (Some(f1), Some(f2)) = (&target.features, &prior_art.features) {
        let feat_text1 = f1.join(" ");
        let feat_text2 = f2.join(" ");
        let jaccard = compute_jaccard(&feat_text1, &feat_text2);
        let score = jaccard * weights[3];
        total_score += score;

        // 找出重叠特征
        let words1 = extract_keywords(&feat_text1);
        let words2 = extract_keywords(&feat_text2);
        let set1: std::collections::HashSet<_> = words1.iter().collect();
        let set2: std::collections::HashSet<_> = words2.iter().collect();
        let overlap: Vec<_> = set1.intersection(&set2).copied().collect();

        dimensions["features"] = json!({
            "score": score,
            "jaccard": jaccard,
            "overlap": overlap
        });
    }

    // 提取关键词
    let target_text = format!(
        "{} {} {}",
        target.title.as_deref().unwrap_or(""),
        target.abstract_text.as_deref().unwrap_or(""),
        target
            .features
            .as_ref()
            .map(|v| v.join(" "))
            .unwrap_or_default()
    );
    let prior_text = format!(
        "{} {} {}",
        prior_art.title.as_deref().unwrap_or(""),
        prior_art.abstract_text.as_deref().unwrap_or(""),
        prior_art
            .features
            .as_ref()
            .map(|v| v.join(" "))
            .unwrap_or_default()
    );

    let keywords_target = extract_keywords(&target_text);
    let keywords_prior = extract_keywords(&prior_text);

    // 结论
    let conclusion = if total_score >= HIGH_SIMILARITY_THRESHOLD {
        "高相似度 - 可能存在冲突"
    } else if total_score >= MEDIUM_SIMILARITY_THRESHOLD {
        "中等相似度 - 需要进一步分析"
    } else {
        "低相似度 - 冲突可能性较低"
    };

    Ok(json!({
        "overall_similarity": total_score,
        "mode": "lexical",
        "dimensions": dimensions,
        "keywords_target": keywords_target,
        "keywords_prior_art": keywords_prior,
        "conclusion": conclusion
    }))
}

#[allow(clippy::unnecessary_wraps)]
fn compare_embedding_stub(_target: &CompareDoc, _prior_art: &CompareDoc) -> Result<Value, String> {
    Ok(json!({
        "status": "stub",
        "message": "embedding模式需要配置向量模型",
        "note": "建议使用sentence-transformers或OpenAI embeddings"
    }))
}

#[allow(clippy::unnecessary_wraps)]
fn compare_marg_stub(_target: &CompareDoc, _prior_art: &CompareDoc) -> Result<Value, String> {
    Ok(json!({
        "status": "stub",
        "message": "MARG模式需要配置多角度推理生成器",
        "note": "MARG = Multi-Aspect Reasoning Generator"
    }))
}
