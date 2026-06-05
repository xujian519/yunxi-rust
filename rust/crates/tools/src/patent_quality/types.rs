use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::LazyLock;

/// 质量评分输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityScorerInput {
    pub claims: Vec<QualityClaim>,
    #[serde(default)]
    pub specification: QualitySpec,
    #[serde(default = "default_patent_type")]
    pub patent_type: String,
    #[serde(default)]
    pub invention_title: String,
    #[serde(default)]
    pub drawings: Vec<ScorerDrawing>,
    #[serde(default = "default_check_level")]
    pub check_level: u8,
}

pub(crate) fn default_patent_type() -> String {
    "invention".to_string()
}

pub(crate) fn default_check_level() -> u8 {
    2
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityClaim {
    pub r#type: String, // "independent" | "dependent"
    pub number: u32,
    pub content: String,
    #[serde(default)]
    pub depends_on: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualitySpec {
    #[serde(default)]
    pub technical_field: Option<String>,
    #[serde(default)]
    pub background_art: Option<String>,
    #[serde(default)]
    pub invention_content: Option<String>,
    #[serde(default)]
    pub embodiment: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorerDrawing {
    pub figure_number: String,
    pub description: String,
}

// --- 输出类型 ---

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerOutput {
    pub completeness_score: f64,
    pub quality_scores: ScorerQualityScores,
    pub overall_quality: f64,
    pub quality_level: String,
    pub issues: Vec<ScorerIssue>,
    pub recommendations: Vec<ScorerRecommendation>,
    pub comparison: ScorerComparison,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerQualityScores {
    pub claims: ScorerDimensionScores,
    pub specification: ScorerSpecScores,
    pub language: ScorerLangScores,
    pub legal: ScorerLegalScores,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerDimensionScores {
    pub clarity: f64,
    pub support: f64,
    pub breadth: f64,
    pub protection_scope: f64,
    pub overall: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerSpecScores {
    pub clarity: f64,
    pub sufficiency: f64,
    pub consistency: f64,
    pub supportiveness: f64,
    pub overall: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerLangScores {
    pub grammar: f64,
    pub terminology: f64,
    pub accuracy: f64,
    pub overall: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerLegalScores {
    pub formality: f64,
    pub patentability: f64,
    pub risk_level: String,
    pub overall: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerIssue {
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_category: Option<String>,
    pub severity: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_reference: Option<String>,
    pub suggestion: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerRecommendation {
    pub area: String,
    pub priority: String,
    pub current: String,
    pub suggested: String,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_impact: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ScorerComparison {
    pub average_quality: f64,
    pub percentile: f64,
    pub ranking: String,
    pub comparison_group: String,
}

// --- 关键词提取辅助 ---

pub(crate) fn extract_keywords(text: &str) -> Vec<String> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[一-龥]{2,}").unwrap());
    let stop_words: HashSet<&str> = ["的", "是", "在", "和", "与", "或", "等", "为", "有", "中"]
        .iter()
        .copied()
        .collect();
    RE.find_iter(text)
        .map(|m| m.as_str().to_string())
        .filter(|w| !stop_words.contains(w.as_str()))
        .collect()
}

pub(crate) fn calculate_keyword_overlap(kw1: &[String], kw2: &[String]) -> f64 {
    if kw1.is_empty() || kw2.is_empty() {
        return 0.0;
    }
    let set1: HashSet<&str> = kw1.iter().map(String::as_str).collect();
    let set2: HashSet<&str> = kw2.iter().map(String::as_str).collect();
    let overlap = set1.intersection(&set2).count();
    overlap as f64 / set1.union(&set2).count().max(1) as f64
}

/// 正态分布累积分布函数（CDF）的近似计算（Abramowitz & Stegun 公式 26.2.17）
/// 最大绝对误差 < 7.5e-8
pub(crate) fn normal_cdf(z: f64) -> f64 {
    let b1 = 0.319_381_530;
    let b2 = -0.356_563_782;
    let b3 = 1.781_477_937;
    let b4 = -1.821_255_978;
    let b5 = 1.330_274_429;
    let p = 0.231_641_9;
    let c = 1.0 / (1.0 + p * z.abs());
    let t = (-z.abs() * z.abs() / 2.0).exp() * 0.398_942_280_401_433;
    let poly = ((((b5 * c + b4) * c + b3) * c + b2) * c + b1) * c;
    let cdf = if z >= 0.0 { 1.0 - t * poly } else { t * poly };
    cdf.clamp(0.0, 1.0)
}

pub(crate) fn scorer_calculate_percentile(value: f64, mean: f64, std_dev: f64) -> f64 {
    if std_dev <= 0.0 {
        return if value >= mean { 100.0 } else { 0.0 };
    }
    let z = (value - mean) / std_dev;
    normal_cdf(z) * 100.0
}

pub(crate) fn scorer_generate_comparison(overall: f64, patent_type: &str) -> ScorerComparison {
    let (avg, std_dev) = match patent_type {
        "utilityModel" => (70.0, 12.0),
        "design" => (72.0, 11.0),
        _ => (75.0, 10.0),
    };
    let percentile = scorer_calculate_percentile(overall, avg, std_dev);
    let ranking = if percentile >= 90.0 {
        "优秀"
    } else if percentile >= 75.0 {
        "良好"
    } else if percentile >= 50.0 {
        "中等"
    } else {
        "待改进"
    };
    let group = match patent_type {
        "utilityModel" => "实用新型申请",
        "design" => "外观设计申请",
        _ => "发明专利申请",
    };
    ScorerComparison {
        average_quality: avg,
        percentile,
        ranking: ranking.to_string(),
        comparison_group: group.to_string(),
    }
}

pub(crate) fn scorer_get_quality_level(overall: f64) -> &'static str {
    if overall >= 90.0 {
        "excellent"
    } else if overall >= 75.0 {
        "good"
    } else if overall >= 60.0 {
        "fair"
    } else {
        "poor"
    }
}
