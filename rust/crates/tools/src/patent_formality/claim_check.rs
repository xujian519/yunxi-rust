// 权利要求形式检查

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 清楚性问题。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClarityIssue {
    pub claim_number: u32,
    pub issue: String,
    pub suggestion: String,
}

/// 非必要技术特征。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnnecessaryFeature {
    pub claim_number: u32,
    pub feature: String,
    pub reason: String,
}

/// 输入权利要求。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CfClaimInput {
    pub claim_number: u32,
    #[serde(default)]
    pub full_text: String,
    #[serde(default)]
    pub content: String,
    #[serde(rename = "type", default)]
    pub claim_type: String,
}

/// 权利要求形式检查输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimFormalityInput {
    pub(super) claims: Vec<CfClaimInput>,
}

/// 权利要求形式检查输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CfOutput {
    pub passed: bool,
    pub clarity_issues: Vec<ClarityIssue>,
    pub unnecessary_features: Vec<UnnecessaryFeature>,
    pub recommendations: Vec<String>,
}

pub(super) const UNCLEAR_TERMS: &[&str] = &[
    "大约",
    "左右",
    "上下",
    "等等",
    "可能",
    "大概",
    "约",
    "或者其组合",
];
const DETAIL_PATTERNS: &[&str] = &["其中所述", "具体来说", "优选地", "更优选地"];
const COMMON_KNOWLEDGE: &[&str] = &["采用常规技术", "使用现有技术", "本领域技术人员熟知"];
const MAX_INDEPENDENT_CLAIM_LENGTH: usize = 300;
const MAX_DETAIL_PATTERN_COUNT: usize = 3;

pub(super) fn is_unclear(content: &str) -> bool {
    UNCLEAR_TERMS.iter().any(|p| content.contains(p))
}

pub(super) fn is_concise(content: &str) -> bool {
    if content.chars().count() > MAX_INDEPENDENT_CLAIM_LENGTH {
        return false;
    }
    let detail_count: usize = DETAIL_PATTERNS
        .iter()
        .map(|p| content.matches(p).count())
        .sum();
    detail_count < MAX_DETAIL_PATTERN_COUNT
}

fn common_knowledge_reason(keyword: &str) -> String {
    match keyword {
        "采用常规技术" => "常规技术不需要写入权利要求",
        "使用现有技术" => "现有技术不需要写入权利要求",
        "本领域技术人员熟知" => "本领域技术人员熟知的内容不需要写入权利要求",
        _ => "该内容不需要写入权利要求",
    }
    .to_string()
}

pub(super) fn identify_unnecessary_features(
    content: &str,
    claim_number: u32,
) -> Vec<UnnecessaryFeature> {
    COMMON_KNOWLEDGE
        .iter()
        .filter(|pattern| content.contains(**pattern))
        .map(|feature| UnnecessaryFeature {
            claim_number,
            feature: (*feature).into(),
            reason: common_knowledge_reason(feature),
        })
        .collect()
}

pub(super) fn perform_claim_formality_check(claims: &[CfClaimInput]) -> CfOutput {
    let mut clarity_issues = Vec::new();
    let mut unnecessary_features = Vec::new();

    for claim in claims {
        let text = if claim.claim_type == "independent" {
            &claim.full_text
        } else {
            &claim.content
        };

        if text.is_empty() {
            continue;
        }

        // 清楚性检查
        if is_unclear(text) {
            clarity_issues.push(ClarityIssue {
                claim_number: claim.claim_number,
                issue: if claim.claim_type == "independent" {
                    "权利要求包含模糊表述".to_string()
                } else {
                    "从属权利要求包含模糊表述".to_string()
                },
                suggestion: "建议使用明确的技术术语，避免\"大约\"\"左右\"\"可能\"等模糊词汇"
                    .to_string(),
            });
        }

        // 简要性检查（仅独立权利要求）
        if claim.claim_type == "independent" && !is_concise(text) {
            clarity_issues.push(ClarityIssue {
                claim_number: claim.claim_number,
                issue: "权利要求过于冗长".to_string(),
                suggestion: "建议删除非必要的技术细节，保持权利要求简洁".to_string(),
            });
        }

        // 非必要技术特征（仅独立权利要求）
        if claim.claim_type == "independent" {
            unnecessary_features.extend(identify_unnecessary_features(text, claim.claim_number));
        }
    }

    let mut recommendations = Vec::new();
    if !clarity_issues.is_empty() {
        recommendations.push("建议修改不清楚或过于冗长的权利要求".to_string());
    }
    if !unnecessary_features.is_empty() {
        recommendations.push("建议删除非必要技术特征（公知常识、常规技术等）".to_string());
    }

    let passed = clarity_issues.is_empty() && unnecessary_features.is_empty();

    CfOutput {
        passed,
        clarity_issues,
        unnecessary_features,
        recommendations,
    }
}

/// 执行权利要求形式检查。
pub fn execute_claim_formality_check(input: &ClaimFormalityInput) -> Result<Value, String> {
    let result = perform_claim_formality_check(&input.claims);
    serde_json::to_value(result).map_err(|e| e.to_string())
}
