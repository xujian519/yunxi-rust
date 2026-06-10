// 单一性检查

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

/// 单一性检查输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnityCheckInput {
    pub(super) claims: Vec<UcClaimInput>,
    #[allow(dead_code)]
    pub(super) patent_type: String, // 保留原因: 预留给不同专利类型的差异化单一性标准
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) invention_title: Option<String>, // 保留原因: 预留给标题与权利要求的关联分析
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcClaimInput {
    #[serde(rename = "type")]
    pub(super) claim_type: String,
    pub(super) number: u32,
    pub(super) content: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) depends_on: Option<u32>, // 保留原因: 预留给权利要求依赖关系的单一性分析
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) category: Option<String>, // 保留原因: 预留给按产品/方法分类的单一性分析
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcOutput {
    pub(super) rule43_unity: UcRule43Check,
    pub(super) rule44_general_concept: UcRule44Check,
    pub(super) feature_analysis: UcFeatureAnalysis,
    pub(super) unity_analysis: UcUnityAnalysis,
    pub(super) overall_report: UcOverallReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcRule43Check {
    pub(super) passed: bool,
    pub(super) issues: Vec<UcRuleIssue>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcRuleIssue {
    pub(super) claim_number: u32,
    pub(super) issue: String,
    pub(super) suggestion: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcRule44Check {
    pub(super) passed: bool,
    pub(super) has_general_concept: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) general_concept: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcFeatureAnalysis {
    pub(super) independent_claims_analysis: Vec<UcIndependentClaimAnalysis>,
    pub(super) common_features: Vec<String>,
    pub(super) corresponding_features: Vec<UcCorrespondingFeature>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcIndependentClaimAnalysis {
    pub(super) claim_number: u32,
    pub(super) content: String,
    pub(super) technical_features: Vec<UcTechFeature>,
    pub(super) primary_features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) technical_field: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcTechFeature {
    pub(super) content: String,
    pub(super) feature_type: String,
    pub(super) weight: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcCorrespondingFeature {
    pub(super) claim1: u32,
    pub(super) claim2: u32,
    pub(super) feature: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcUnityAnalysis {
    pub(super) has_unity: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) general_inventive_concept: Option<String>,
    pub(super) common_features: Vec<String>,
    pub(super) technical_correlation_score: f64,
    pub(super) details: UcUnityDetails,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcUnityDetails {
    pub(super) independent_claims_count: usize,
    pub(super) unified_groups: Vec<Vec<u32>>,
    pub(super) non_unified_claims: Vec<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UcOverallReport {
    pub(super) passed: bool,
    pub(super) total_issues: usize,
    pub(super) recommendations: Vec<String>,
    pub(super) unity_score: u32,
}

pub(super) fn analyze_independent_claims(input: &UnityCheckInput, output: &mut UcOutput) {
    let independent: Vec<&UcClaimInput> = input
        .claims
        .iter()
        .filter(|c| c.claim_type == "independent")
        .collect();

    for claim in &independent {
        let features = uc_extract_technical_features(&claim.content);
        let primary = identify_primary_features(&features);
        let field = uc_infer_technical_field(&claim.content);

        output
            .feature_analysis
            .independent_claims_analysis
            .push(UcIndependentClaimAnalysis {
                claim_number: claim.number,
                content: claim.content.clone(),
                technical_features: features,
                primary_features: primary,
                technical_field: field,
            });
    }
}

pub(super) fn identify_common_features(output: &mut UcOutput) {
    let analyses = &output.feature_analysis.independent_claims_analysis;
    if analyses.len() < 2 {
        return;
    }

    let all_features: Vec<Vec<String>> = analyses
        .iter()
        .map(|a| {
            a.technical_features
                .iter()
                .map(|f| f.content.clone())
                .collect()
        })
        .collect();

    let first = &all_features[0];
    let mut common = Vec::new();

    for feature in first {
        let in_all = all_features[1..].iter().all(|fs| fs.contains(feature));
        if in_all && !common.contains(feature) {
            common.push(feature.clone());
        }
    }

    output.feature_analysis.common_features = common;
    identify_corresponding_features(output);
}

fn identify_corresponding_features(output: &mut UcOutput) {
    let analyses = &output.feature_analysis.independent_claims_analysis;

    for i in 0..analyses.len() {
        for j in (i + 1)..analyses.len() {
            let features1: Vec<&str> = analyses[i]
                .technical_features
                .iter()
                .map(|f| f.content.as_str())
                .collect();
            let features2: Vec<&str> = analyses[j]
                .technical_features
                .iter()
                .map(|f| f.content.as_str())
                .collect();

            for f1 in &features1 {
                for f2 in &features2 {
                    if are_features_corresponding(f1, f2) {
                        output.feature_analysis.corresponding_features.push(
                            UcCorrespondingFeature {
                                claim1: analyses[i].claim_number,
                                claim2: analyses[j].claim_number,
                                feature: f1.to_string(),
                            },
                        );
                    }
                }
            }
        }
    }
}

fn are_features_corresponding(f1: &str, f2: &str) -> bool {
    if f1 == f2 {
        return false; // 已在 commonFeatures 中处理
    }
    if f1.contains(f2) || f2.contains(f1) {
        return true;
    }
    calculate_similarity(f1, f2) > 0.6
}

/// 加权相似度：字符重叠 0.2 + Jaccard 0.5 + Bigram 余弦 0.3。
pub(super) fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    let max_chars = s1.chars().count().max(s2.chars().count()) as f64;
    if max_chars == 0.0 {
        return 0.0;
    }

    // 字符重叠（基于字符数，非字节数）
    let chars2: HashSet<char> = s2.chars().collect();
    let common_chars = s1.chars().filter(|c| chars2.contains(c)).count() as f64;
    let char_overlap = common_chars / max_chars;

    // Jaccard（按词）
    let words1: HashSet<&str> = s1.split_whitespace().filter(|w| !w.is_empty()).collect();
    let words2: HashSet<&str> = s2.split_whitespace().filter(|w| !w.is_empty()).collect();
    let jaccard = if words1.is_empty() && words2.is_empty() {
        char_overlap
    } else {
        let intersection = words1.intersection(&words2).count() as f64;
        let union = words1.union(&words2).count() as f64;
        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    };

    // Bigram 余弦（基于字符级 bigram）
    let bigrams1 = get_bigrams(s1);
    let bigrams2 = get_bigrams(s2);
    let bigram2_set: HashSet<&String> = bigrams2.iter().collect();
    let common_bigrams = bigrams1.iter().filter(|b| bigram2_set.contains(*b)).count() as f64;
    let cosine = if !bigrams1.is_empty() && !bigrams2.is_empty() {
        (2.0 * common_bigrams) / (bigrams1.len() + bigrams2.len()) as f64
    } else {
        0.0
    };

    // 加权组合: 字符重叠 0.2 + Jaccard 0.5 + cosine 0.3
    // 权重设计: Jaccard 对中文词汇匹配最稳定，字符重叠补充未分词短语，cosine 捕捉语义分布
    char_overlap * 0.2 + jaccard * 0.5 + cosine * 0.3
}

/// 字符级 bigram 提取。对中文按字符窗口计算，对英文按字节窗口计算。
fn get_bigrams(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return Vec::new();
    }
    chars
        .windows(2)
        .map(|w| format!("{}{}", w[0], w[1]))
        .collect()
}

fn check_unity_requirement(independent_count: usize, output: &mut UcOutput) {
    if independent_count <= 1 {
        output.rule43_unity.passed = true;
        output.unity_analysis.has_unity = true;
        let nums: Vec<u32> = output
            .feature_analysis
            .independent_claims_analysis
            .iter()
            .map(|a| a.claim_number)
            .collect();
        output.unity_analysis.details.unified_groups = vec![nums];
        return;
    }

    let has_common = !output.feature_analysis.common_features.is_empty();
    let has_corresponding = !output.feature_analysis.corresponding_features.is_empty();

    if !has_common && !has_corresponding {
        let claim_nums: Vec<u32> = output
            .feature_analysis
            .independent_claims_analysis
            .iter()
            .map(|a| a.claim_number)
            .collect();

        for num in &claim_nums {
            output.rule43_unity.issues.push(UcRuleIssue {
                claim_number: *num,
                issue: "与其他独立权利要求之间缺乏相同或相应的特定技术特征".into(),
                suggestion: "建议删除或分案申请".into(),
            });
        }

        output.rule43_unity.passed = false;
        output.unity_analysis.has_unity = false;
        output.unity_analysis.details.non_unified_claims = claim_nums;
    } else {
        output.rule43_unity.passed = true;
        output.unity_analysis.has_unity = true;
        let nums: Vec<u32> = output
            .feature_analysis
            .independent_claims_analysis
            .iter()
            .map(|a| a.claim_number)
            .collect();
        output.unity_analysis.details.unified_groups = vec![nums];
    }
}

fn evaluate_general_incept(output: &mut UcOutput) {
    let analyses = &output.feature_analysis.independent_claims_analysis;

    if analyses.len() <= 1 {
        output.rule44_general_concept.passed = true;
        output.rule44_general_concept.has_general_concept = true;
        output.rule44_general_concept.general_concept = analyses
            .first()
            .map(|a| a.content.clone())
            .or(Some("单一发明".into()));
        return;
    }

    let common = &output.feature_analysis.common_features;
    if common.is_empty() {
        output.rule44_general_concept.has_general_concept = false;
        output.rule44_general_concept.passed = false;
        let corr_count = output.feature_analysis.corresponding_features.len();
        output.unity_analysis.technical_correlation_score = (corr_count as f64 * 0.15).min(1.0);
    } else {
        output.rule44_general_concept.has_general_concept = true;
        output.rule44_general_concept.passed = true;
        let concept = format!("基于{}的{}项相关发明", common.join("、"), analyses.len());
        output.rule44_general_concept.general_concept = Some(concept.clone());
        output.unity_analysis.general_inventive_concept = Some(concept);
        output.unity_analysis.technical_correlation_score =
            (0.5 + common.len() as f64 * 0.1).min(1.0);
    }
}

fn generate_uc_overall_report(output: &mut UcOutput) {
    let total = output.rule43_unity.issues.len();
    output.overall_report.total_issues = total;

    let base = if output.unity_analysis.has_unity {
        60.0
    } else {
        0.0
    };
    let bonus = output.unity_analysis.technical_correlation_score * 40.0;
    output.overall_report.unity_score = (base + bonus).round() as u32;

    output.overall_report.passed =
        output.rule43_unity.passed && output.rule44_general_concept.passed;

    let mut recs = Vec::new();
    if !output.unity_analysis.has_unity {
        recs.push("建议将不具备单一性的权利要求分案申请".into());
    }
    if !output.rule43_unity.issues.is_empty() {
        recs.push("建议删除或修改不具备单一性的权利要求".into());
    }
    if !output.rule44_general_concept.has_general_concept
        && output.feature_analysis.independent_claims_analysis.len() > 1
    {
        recs.push("建议补充总的发明构思说明，或调整权利要求使其属于同一发明构思".into());
    }
    if output.overall_report.unity_score >= 80 {
        recs.push("单一性良好，可以继续申请".into());
    } else if output.overall_report.unity_score >= 60 {
        recs.push("单一性一般，建议进一步优化权利要求的关联性".into());
    }
    output.overall_report.recommendations = recs;
}

static RE_UC_QUOTE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r#"["'「」『』]([^"'「」『』]{2,})["'「」『』]"#).unwrap()
});

static RE_UC_COMP: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(
        r"[一-龥a-zA-Z0-9]{2,6}(?:芯片|电路|传感器|控制器|处理器|执行器|驱动器|接收器|发射器|显示器|存储器|计数器|定时器|调节器|变换器|转换器|适配器|连接器)",
    ).unwrap()
});

fn uc_extract_technical_features(content: &str) -> Vec<UcTechFeature> {
    let mut features = Vec::new();

    // 引号中的特征
    for cap in RE_UC_QUOTE.captures_iter(content) {
        let f = cap[1].to_string();
        features.push(UcTechFeature {
            content: f.clone(),
            feature_type: infer_feature_type(&f),
            weight: 1.0,
        });
    }

    // 组件特征
    for cap in RE_UC_COMP.captures_iter(content) {
        let f = cap[0].to_string();
        if !features.iter().any(|fe| fe.content == f) {
            features.push(UcTechFeature {
                content: f,
                feature_type: "structural".into(),
                weight: 0.8,
            });
        }
    }

    features
}

fn infer_feature_type(content: &str) -> String {
    static RE_METHOD: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static RE_STRUCT: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    static RE_COMP: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

    let re_method =
        RE_METHOD.get_or_init(|| Regex::new(r"方法|工艺|步骤|流程|过程").expect("valid regex"));
    let re_struct =
        RE_STRUCT.get_or_init(|| Regex::new(r"包括|包含|设有|设置|配置").expect("valid regex"));
    let re_comp =
        RE_COMP.get_or_init(|| Regex::new(r"由.*组成|成分|材料|组合物").expect("valid regex"));

    if re_method.is_match(content) {
        return "method".into();
    }
    if re_struct.is_match(content) {
        return "structural".into();
    }
    if re_comp.is_match(content) {
        return "compositional".into();
    }
    "functional".into()
}

fn identify_primary_features(features: &[UcTechFeature]) -> Vec<String> {
    let mut sorted = features.to_vec();
    sorted.sort_by(|a, b| {
        b.weight
            .partial_cmp(&a.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    sorted.iter().take(3).map(|f| f.content.clone()).collect()
}

fn uc_infer_technical_field(content: &str) -> Option<String> {
    static FIELDS: std::sync::OnceLock<Vec<(regex::Regex, &'static str)>> =
        std::sync::OnceLock::new();
    let fields = FIELDS.get_or_init(|| {
        vec![
            (
                Regex::new(r"电子|电路|芯片|处理器|控制器|传感器").expect("valid regex"),
                "电子技术",
            ),
            (
                Regex::new(r"机械|装置|设备|机构|结构").expect("valid regex"),
                "机械工程",
            ),
            (
                Regex::new(r"化学|材料|组合物|成分|合成").expect("valid regex"),
                "化学工程",
            ),
            (
                Regex::new(r"软件|算法|数据处理|计算|程序").expect("valid regex"),
                "计算机软件",
            ),
            (
                Regex::new(r"通信|网络|传输|信号").expect("valid regex"),
                "通信技术",
            ),
        ]
    });

    for (re, field) in fields {
        if re.is_match(content) {
            return Some(field.to_string());
        }
    }

    None
}

/// 执行单一性检查。
pub fn execute_unity_check(input: &UnityCheckInput) -> Result<Value, String> {
    let independent_count = input
        .claims
        .iter()
        .filter(|c| c.claim_type == "independent")
        .count();

    let mut output = UcOutput {
        rule43_unity: UcRule43Check {
            passed: true,
            issues: Vec::new(),
        },
        rule44_general_concept: UcRule44Check {
            passed: false,
            has_general_concept: false,
            general_concept: None,
        },
        feature_analysis: UcFeatureAnalysis {
            independent_claims_analysis: Vec::new(),
            common_features: Vec::new(),
            corresponding_features: Vec::new(),
        },
        unity_analysis: UcUnityAnalysis {
            has_unity: false,
            general_inventive_concept: None,
            common_features: Vec::new(),
            technical_correlation_score: 0.0,
            details: UcUnityDetails {
                independent_claims_count: independent_count,
                unified_groups: Vec::new(),
                non_unified_claims: Vec::new(),
            },
        },
        overall_report: UcOverallReport {
            passed: false,
            total_issues: 0,
            recommendations: Vec::new(),
            unity_score: 0,
        },
    };

    // 1. 分析独立权利要求
    analyze_independent_claims(input, &mut output);

    // 2. 识别共同特征
    identify_common_features(&mut output);

    // 3. 检查单一性
    check_unity_requirement(independent_count, &mut output);

    // 4. 评估总的发明构思
    evaluate_general_incept(&mut output);

    // 5. 生成报告
    generate_uc_overall_report(&mut output);

    serde_json::to_value(output).map_err(|e| e.to_string())
}
