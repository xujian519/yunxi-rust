// 说明书形式检查

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// 说明书形式检查输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecFormalityInput {
    pub(super) specification: SfSpecificationInput,
    #[serde(default)]
    pub(super) claims: Vec<SfClaimInput>,
    pub(super) patent_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfSpecificationInput {
    #[serde(default)]
    pub(super) technical_field: Option<String>,
    #[serde(default)]
    pub(super) background_art: Option<String>,
    #[serde(default)]
    pub(super) invention_content: Option<String>,
    #[serde(default)]
    pub(super) drawings_description: Option<String>,
    #[serde(default)]
    pub(super) embodiment: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfClaimInput {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub(super) claim_type: String, // 保留原因: 预留给权利要求类型（独立/从属）分析
    pub(super) number: u32,
    pub(super) content: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) depends_on: Option<u32>, // 保留原因: 预留给权利要求依赖关系检查
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfOutput {
    pub(super) article26_3_disclosure: SfArticleCheck,
    pub(super) article26_4_clarity: SfArticleCheck,
    pub(super) rule17_components: SfComponentCheck,
    pub(super) rule18_drawings: SfDrawingCheck,
    pub(super) rule19_embodiment: SfArticleCheck,
    pub(super) claims_consistency: SfClaimsConsistencyCheck,
    pub(super) overall_report: SfOverallReport,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfArticleCheck {
    pub(super) passed: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) issues: Vec<SfIssueItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfIssueItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) section: Option<String>,
    pub(super) issue: String,
    pub(super) suggestion: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfComponentCheck {
    pub(super) passed: bool,
    pub(super) missing_components: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfDrawingCheck {
    pub(super) passed: bool,
    pub(super) has_drawings: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) issues: Vec<SfIssueItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfClaimsConsistencyCheck {
    pub(super) passed: bool,
    pub(super) unsupported_claims: Vec<SfUnsupportedClaim>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfUnsupportedClaim {
    pub(super) claim_number: u32,
    pub(super) claim_content: String,
    pub(super) missing_in_spec: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SfOverallReport {
    pub(super) passed: bool,
    pub(super) total_issues: usize,
    pub(super) critical_issues: usize,
    pub(super) recommendations: Vec<String>,
}

static RE_UNCLEAR_EXPRESSIONS: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"大约|左右|上下|等等|可能|大概|约|或者其组合").unwrap()
});

static RE_BACKGROUND_DEFICIENCY: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"不足|缺点|缺陷|问题").unwrap());

static RE_TECH_SOLUTION_KEYWORDS: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"技术方案|解决|采用|包括|设置").unwrap());

static RE_UTILITY_MODEL_STRUCTURE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"形状|构造|结构|设置|配置|连接").unwrap());

static RE_FIGURE_REFERENCE: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"图\s*\d+|图\s*[一二三四五六七八九十]").unwrap());

static RE_FIG_REF_IN_EMBODIMENT: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"如图\s*\d+|如图\s*[一二三四五六七八九十]").unwrap());

static RE_QUOTE_EXTRACTION: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"["'「」『』]([^"'「」『』]{2,})["'「」『』]"#).unwrap()
});

static RE_COMPONENT_EXTRACTION: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"[一-龥a-zA-Z]{2,4}(?:器|件|装置|设备|机构|系统|单元|模块|组件|传感器|控制器|处理器|执行器|驱动器|接收器|发射器|显示器|存储器|计数器|定时器|调节器|变换器|转换器|适配器|连接器|接口|电路|网络|总线|通道|端口)").unwrap()
});

pub(super) fn check_adequate_disclosure(spec: &SfSpecificationInput, result: &mut SfOutput) {
    if let Some(field) = &spec.technical_field {
        if field.chars().count() < 20 {
            result.article26_3_disclosure.issues.push(SfIssueItem {
                section: Some("技术领域".into()),
                issue: "技术领域描述过于简单".into(),
                suggestion: "建议详细说明发明所属或直接应用的技术领域".into(),
            });
        }
    }

    if let Some(bg) = &spec.background_art {
        if !RE_BACKGROUND_DEFICIENCY.is_match(bg) {
            result.article26_3_disclosure.issues.push(SfIssueItem {
                section: Some("背景技术".into()),
                issue: "未明确指出现有技术的不足".into(),
                suggestion: "建议描述现有技术存在的问题和缺点".into(),
            });
        }
    }

    if let Some(content) = &spec.invention_content {
        if !RE_TECH_SOLUTION_KEYWORDS.is_match(content) {
            result.article26_3_disclosure.issues.push(SfIssueItem {
                section: Some("发明内容".into()),
                issue: "缺少技术方案描述".into(),
                suggestion: "建议详细描述解决技术问题所采用的技术方案".into(),
            });
        }
    }

    if let Some(emb) = &spec.embodiment {
        if emb.chars().count() < 100 {
            result.article26_3_disclosure.issues.push(SfIssueItem {
                section: Some("具体实施方式".into()),
                issue: "具体实施方式过于简单".into(),
                suggestion: "建议详细描述实现发明的优选方式，使本领域技术人员能够实现".into(),
            });
        }
    }

    result.article26_3_disclosure.passed = result.article26_3_disclosure.issues.is_empty();
}

pub(super) fn check_clarity_and_brevity(spec: &SfSpecificationInput, result: &mut SfOutput) {
    let sections: Vec<(&str, Option<&String>)> = vec![
        ("技术领域", spec.technical_field.as_ref()),
        ("背景技术", spec.background_art.as_ref()),
        ("发明内容", spec.invention_content.as_ref()),
        ("附图说明", spec.drawings_description.as_ref()),
        ("具体实施方式", spec.embodiment.as_ref()),
    ];

    let unclear_re = &*RE_UNCLEAR_EXPRESSIONS;

    for (name, content) in &sections {
        let Some(content) = content else { continue };

        if unclear_re.is_match(content) {
            result.article26_4_clarity.issues.push(SfIssueItem {
                section: Some(name.to_string()),
                issue: "包含不清楚的表述".into(),
                suggestion: "建议使用明确的技术术语，避免模糊词汇".into(),
            });
        }

        // 非具体实施方式部分检查是否简要
        if *name != "具体实施方式" && !is_spec_concise(content) {
            result.article26_4_clarity.issues.push(SfIssueItem {
                section: Some(name.to_string()),
                issue: "描述过于冗长".into(),
                suggestion: "建议删除非必要的技术细节，保持描述简洁".into(),
            });
        }
    }

    result.article26_4_clarity.passed = result.article26_4_clarity.issues.is_empty();
}

pub(super) fn check_required_components(spec: &SfSpecificationInput, result: &mut SfOutput) {
    let mut missing = Vec::new();
    if spec
        .technical_field
        .as_ref()
        .is_none_or(|s| s.trim().is_empty())
    {
        missing.push("技术领域".into());
    }
    if spec
        .background_art
        .as_ref()
        .is_none_or(|s| s.trim().is_empty())
    {
        missing.push("背景技术".into());
    }
    if spec
        .invention_content
        .as_ref()
        .is_none_or(|s| s.trim().is_empty())
    {
        missing.push("发明内容".into());
    }
    if spec.embodiment.as_ref().is_none_or(|s| s.trim().is_empty()) {
        missing.push("具体实施方式".into());
    }
    result.rule17_components.passed = missing.is_empty();
    result.rule17_components.missing_components = missing;
}

pub(super) fn check_drawings_description(spec: &SfSpecificationInput, result: &mut SfOutput) {
    let has_drawings = spec
        .drawings_description
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty());
    result.rule18_drawings.has_drawings = has_drawings;

    if !has_drawings {
        result.rule18_drawings.issues.push(SfIssueItem {
            section: None,
            issue: "缺少附图说明".into(),
            suggestion: "如果有附图，应当说明附图的图名和简要说明".into(),
        });
        result.rule18_drawings.passed = false;
        return;
    }

    // SAFETY: has_drawings early-returned if drawings_description is None.
    // SAFETY: early return above guarantees drawings_description is Some when has_drawings is true.
    let desc = spec
        .drawings_description
        .as_ref()
        .expect("has_drawings guarantees Some");
    if !RE_FIGURE_REFERENCE.is_match(desc) {
        result.rule18_drawings.issues.push(SfIssueItem {
            section: None,
            issue: "附图说明格式不规范".into(),
            suggestion: "建议使用\"图1为...\"、\"图2为...\"等格式".into(),
        });
    }

    result.rule18_drawings.passed = result.rule18_drawings.issues.is_empty();
}

pub(super) fn check_embodiment(
    spec: &SfSpecificationInput,
    patent_type: &str,
    result: &mut SfOutput,
) {
    if spec.embodiment.as_ref().is_none_or(|s| s.trim().is_empty()) {
        result.rule19_embodiment.issues.push(SfIssueItem {
            section: None,
            issue: "缺少具体实施方式".into(),
            suggestion: "应当至少提供一个实施例，详细描述实现发明的优选方式".into(),
        });
        result.rule19_embodiment.passed = false;
        return;
    }

    // SAFETY: early return above guarantees embodiment is Some.
    let emb = spec
        .embodiment
        .as_ref()
        .expect("embodiment guaranteed by early return");

    // 检查附图引用一致性
    let fig_ref_re = &*RE_FIG_REF_IN_EMBODIMENT;
    if fig_ref_re.is_match(emb)
        && spec
            .drawings_description
            .as_ref()
            .is_none_or(|s| s.trim().is_empty())
    {
        result.rule19_embodiment.issues.push(SfIssueItem {
            section: None,
            issue: "具体实施方式引用了附图但缺少附图说明".into(),
            suggestion: "建议在附图说明部分补充相应的附图说明".into(),
        });
    }

    // 实用新型必须包含形状、构造特征
    if patent_type == "utilityModel" && !RE_UTILITY_MODEL_STRUCTURE.is_match(emb) {
        result.rule19_embodiment.issues.push(SfIssueItem {
            section: None,
            issue: "实用新型缺少形状、构造特征描述".into(),
            suggestion: "实用新型必须描述产品的形状、构造或其结合".into(),
        });
    }

    result.rule19_embodiment.passed = result.rule19_embodiment.issues.is_empty();
}

pub(super) fn check_claims_consistency(
    spec: &SfSpecificationInput,
    claims: &[SfClaimInput],
    result: &mut SfOutput,
) {
    let spec_text = format!(
        "{} {} {} {} {}",
        spec.technical_field.as_deref().unwrap_or(""),
        spec.background_art.as_deref().unwrap_or(""),
        spec.invention_content.as_deref().unwrap_or(""),
        spec.drawings_description.as_deref().unwrap_or(""),
        spec.embodiment.as_deref().unwrap_or("")
    )
    .to_lowercase();

    for claim in claims {
        let features = extract_spec_key_features(&claim.content);
        for feature in features {
            if !spec_text.contains(&feature.to_lowercase()) {
                result
                    .claims_consistency
                    .unsupported_claims
                    .push(SfUnsupportedClaim {
                        claim_number: claim.number,
                        claim_content: claim.content.clone(),
                        missing_in_spec: feature,
                    });
            }
        }
    }

    result.claims_consistency.passed = result.claims_consistency.unsupported_claims.is_empty();
}

pub(super) fn is_spec_concise(content: &str) -> bool {
    if content.chars().count() > 500 {
        return false;
    }
    let sentences: Vec<&str> = content
        .split(&['。', '；', ';', '！', '?', '！', '？'][..])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if sentences.is_empty() {
        return true;
    }
    let unique: HashSet<&str> = sentences.iter().copied().collect();
    let duplication_ratio = 1.0 - (unique.len() as f64 / sentences.len() as f64);
    duplication_ratio < 0.3
}

fn extract_spec_key_features(content: &str) -> Vec<String> {
    let mut features = Vec::new();

    // 提取引号中的内容
    for cap in RE_QUOTE_EXTRACTION.captures_iter(content) {
        let f = cap[1].to_string();
        if !features.contains(&f) {
            features.push(f);
        }
    }

    // 提取组件术语
    for cap in RE_COMPONENT_EXTRACTION.captures_iter(content) {
        let f = cap[0].to_string();
        if f.len() >= 3 && f.len() <= 10 && !features.contains(&f) {
            features.push(f);
        }
    }

    features
}

pub(super) fn generate_spec_overall_report(result: &mut SfOutput) {
    let total = result.article26_3_disclosure.issues.len()
        + result.article26_4_clarity.issues.len()
        + result.rule17_components.missing_components.len()
        + result.rule18_drawings.issues.len()
        + result.rule19_embodiment.issues.len()
        + result.claims_consistency.unsupported_claims.len();

    let critical = result.article26_3_disclosure.issues.len()
        + result.rule17_components.missing_components.len()
        + result.claims_consistency.unsupported_claims.len();

    result.overall_report.total_issues = total;
    result.overall_report.critical_issues = critical;
    result.overall_report.passed = total == 0;

    let mut recs = Vec::new();
    if !result.article26_3_disclosure.issues.is_empty() {
        recs.push("建议补充技术细节，确保充分公开".into());
    }
    if !result.article26_4_clarity.issues.is_empty() {
        recs.push("建议修改不清楚或不简要的表述".into());
    }
    if !result.rule17_components.missing_components.is_empty() {
        recs.push(format!(
            "建议补充缺少的组成部分：{}",
            result.rule17_components.missing_components.join("、")
        ));
    }
    if !result.rule18_drawings.issues.is_empty() {
        recs.push("建议完善附图说明".into());
    }
    if !result.rule19_embodiment.issues.is_empty() {
        recs.push("建议补充具体实施方式".into());
    }
    if !result.claims_consistency.unsupported_claims.is_empty() {
        recs.push("建议在说明书中补充对权利要求中技术特征的描述".into());
    }
    result.overall_report.recommendations = recs;
}

/// 执行说明书形式检查。
pub fn execute_spec_formality_check(input: &SpecFormalityInput) -> Result<Value, String> {
    let mut output = SfOutput {
        article26_3_disclosure: SfArticleCheck::default(),
        article26_4_clarity: SfArticleCheck::default(),
        rule17_components: SfComponentCheck {
            passed: true,
            missing_components: Vec::new(),
        },
        rule18_drawings: SfDrawingCheck {
            passed: true,
            has_drawings: false,
            issues: Vec::new(),
        },
        rule19_embodiment: SfArticleCheck::default(),
        claims_consistency: SfClaimsConsistencyCheck {
            passed: true,
            unsupported_claims: Vec::new(),
        },
        overall_report: SfOverallReport {
            passed: true,
            total_issues: 0,
            critical_issues: 0,
            recommendations: Vec::new(),
        },
    };

    // 执行 6 项检查
    check_adequate_disclosure(&input.specification, &mut output);
    check_clarity_and_brevity(&input.specification, &mut output);
    check_required_components(&input.specification, &mut output);
    check_drawings_description(&input.specification, &mut output);
    check_embodiment(&input.specification, &input.patent_type, &mut output);

    if !input.claims.is_empty() {
        check_claims_consistency(&input.specification, &input.claims, &mut output);
    }

    generate_spec_overall_report(&mut output);

    serde_json::to_value(output).map_err(|e| e.to_string())
}
