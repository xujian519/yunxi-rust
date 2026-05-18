// 工具 1: OaParse - OA 文档解析器

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OA 解析工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct OaParseInput {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub patent_title: Option<String>, // 保留原因: 预留给 OA 答复时显示专利标题
    #[serde(default)]
    pub document_type: String, // cn/pct/us/ep
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub examiner: Option<String>, // 保留原因: 预留给 OA 答复时显示审查员信息
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub notification_date: Option<String>, // 保留原因: 预留给 OA 答复时计算截止日期
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(dead_code)]
    pub deadline: Option<String>, // 保留原因: 预留给 OA 答复时提醒用户截止日期
}

/// OA 解析工具输出
#[derive(Debug, Clone, Serialize)]
pub struct OaParseOutput {
    pub application_number: String,
    pub document_type: String,
    pub rejection_reasons: Vec<RejectionReason>,
    pub summary: String,
    pub overall_severity: String,
    pub total_affected_claims: Vec<u32>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RejectionReason {
    #[serde(rename = "type")]
    pub rejection_type: String,
    pub severity: String,
    pub description: String,
    pub affected_claims: Vec<u32>,
    pub cited_references: Vec<String>,
    pub overcome_probability: f64,
    pub suggested_response: String,
}

/// 驳回类型枚举
#[derive(Debug, Clone, PartialEq)]
pub(super) enum RejectionType {
    Novelty,
    Inventiveness,
    Utility,
    Support,
    Clarity,
    Scope,
    AmendmentScope,
    Unity,
    Formality,
    #[allow(dead_code)]
    Other,
}

#[allow(dead_code)]
impl RejectionType {
    pub(super) fn from_str(s: &str) -> Self {
        match s {
            "Novelty" => RejectionType::Novelty,
            "Inventiveness" => RejectionType::Inventiveness,
            "Utility" => RejectionType::Utility,
            "Support" => RejectionType::Support,
            "Clarity" => RejectionType::Clarity,
            "Scope" => RejectionType::Scope,
            "AmendmentScope" => RejectionType::AmendmentScope,
            "Unity" => RejectionType::Unity,
            "Formality" => RejectionType::Formality,
            _ => RejectionType::Other,
        }
    }

    pub(super) fn keywords(&self) -> Vec<&'static str> {
        match self {
            RejectionType::Novelty => vec![
                "不具备新颖性",
                "已被公开",
                "已被披露",
                "现有技术",
                "对比文件",
                "公开了",
                "没有新颖性",
                "属于现有技术",
                "完全相同",
                "不具备区别技术特征",
            ],
            RejectionType::Inventiveness => vec![
                "不具备创造性",
                "显而易见",
                "容易想到",
                "常规技术手段",
                "无需创造性劳动",
                "不具备突出的实质性特点",
                "不具备显著进步",
                "结合启示",
                "常规选择",
            ],
            RejectionType::Utility => vec![
                "不具备实用性",
                "无法制造",
                "无法使用",
                "无法产生积极效果",
                "缺乏实用性",
                "不能实施",
            ],
            RejectionType::Support => vec![
                "未充分公开",
                "说明书不支持",
                "超范围",
                "无法从说明书得到",
                "记载不充分",
                "技术方案不清楚",
            ],
            RejectionType::Clarity => vec![
                "不清楚",
                "不明确",
                "模糊",
                "限定不确切",
                "权利要求不清楚",
                "表述不清",
                "含义不明",
            ],
            RejectionType::Scope => {
                vec!["保护范围不明确", "范围过宽", "范围过窄", "保护范围不清楚"]
            }
            RejectionType::AmendmentScope => vec![
                "修改超范围",
                "超出原说明书和权利要求书记载的范围",
                "新增技术特征",
                "修改不符合规定",
            ],
            RejectionType::Unity => vec![
                "不具备单一性",
                "缺少单一性",
                "没有特定技术特征",
                "不属于一个总的发明构思",
                "单一性缺陷",
            ],
            RejectionType::Formality => vec![
                "形式缺陷",
                "格式不符合",
                "缺少必要文件",
                "术语不规范",
                "撰写不规范",
                "不符合规定格式",
            ],
            RejectionType::Other => vec![],
        }
    }

    pub(super) fn base_overcome_probability(&self) -> f64 {
        match self {
            RejectionType::Novelty | RejectionType::Unity => 0.55,
            RejectionType::Inventiveness | RejectionType::Other => 0.50,
            RejectionType::Utility | RejectionType::Scope => 0.60,
            RejectionType::Support => 0.65,
            RejectionType::Clarity => 0.70,
            RejectionType::AmendmentScope => 0.45,
            RejectionType::Formality => 0.75,
        }
    }
}

pub fn execute_oa_parse(input: &OaParseInput) -> Result<Value, String> {
    let content = &input.content;
    let doc_type = input.document_type.clone();

    // 提取申请号
    let app_number = if let Some(num) = &input.application_number {
        num.clone()
    } else {
        extract_application_number(content, &doc_type).unwrap_or_default()
    };

    // 识别驳回类型
    let rejection_reasons = classify_rejections(content, &doc_type);

    // 计算总体严重程度
    let overall_severity = calculate_overall_severity(&rejection_reasons);

    // 汇总受影响的权利要求
    let mut all_claims = rejection_reasons
        .iter()
        .flat_map(|r| r.affected_claims.clone())
        .collect::<Vec<_>>();
    all_claims.sort_unstable();
    all_claims.dedup();

    // 生成摘要
    let summary = generate_summary(&rejection_reasons, &all_claims);

    // 计算置信度
    let confidence = calculate_confidence(&rejection_reasons, content);

    let output = OaParseOutput {
        application_number: app_number,
        document_type: doc_type,
        rejection_reasons,
        summary,
        overall_severity,
        total_affected_claims: all_claims,
        confidence,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

/// 提取申请号
fn extract_application_number(content: &str, doc_type: &str) -> Option<String> {
    let patterns = match doc_type {
        "cn" => vec![r"CN\d{12,13}\.\d|ZL\d{12}\.\d", r"\d{12,13}\.\d"],
        "pct" => vec![r"PCT/CN\d{4}/\d+"],
        "us" => vec![r"US\d{2}/\d{4,8}"],
        "ep" => vec![r"EP\d{8}"],
        _ => return None,
    };

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.find(content) {
                return Some(caps.as_str().to_string());
            }
        }
    }
    None
}

/// 分类驳回理由
fn classify_rejections(content: &str, _doc_type: &str) -> Vec<RejectionReason> {
    let mut reasons = Vec::new();
    let content_lower = content.to_lowercase();

    for rejection_type in &[
        RejectionType::Novelty,
        RejectionType::Inventiveness,
        RejectionType::Utility,
        RejectionType::Support,
        RejectionType::Clarity,
        RejectionType::Scope,
        RejectionType::AmendmentScope,
        RejectionType::Unity,
        RejectionType::Formality,
    ] {
        let keywords = rejection_type.keywords();
        let matched_keywords: Vec<_> = keywords
            .iter()
            .filter(|kw| content_lower.contains(&kw.to_lowercase()))
            .collect();

        if !matched_keywords.is_empty() {
            // 提取受影响的权利要求
            let affected_claims = extract_affected_claims(content);

            // 提取引用的现有技术
            let cited_references = extract_cited_references(content);

            // 计算严重程度
            let severity = calculate_severity(&matched_keywords, rejection_type);

            // 计算克服概率
            let mut overcome_prob = rejection_type.base_overcome_probability();
            overcome_prob += match severity.as_str() {
                "high" => -0.15,
                "low" => 0.10,
                _ => 0.0,
            };
            overcome_prob = overcome_prob.clamp(0.0, 1.0);

            // 建议答复策略
            let suggested_response = suggest_response_strategy(rejection_type, &severity);

            let description = format!(
                "{rejection_type:?} - 匹配关键词: {}",
                matched_keywords
                    .iter()
                    .map(|s| **s)
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            reasons.push(RejectionReason {
                rejection_type: format!("{rejection_type:?}"),
                severity,
                description,
                affected_claims,
                cited_references,
                overcome_probability: overcome_prob,
                suggested_response,
            });
        }
    }

    reasons
}

/// 提取受影响的权利要求
fn extract_affected_claims(content: &str) -> Vec<u32> {
    let mut claims = Vec::new();

    // 匹配 "权利要求 1" 或 "权利要求1-3" 等格式
    let patterns = vec![r"权利要求\s*(\d+)", r"claim\s*(\d+)", r"权\s*(\d+)"];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(content) {
                if let Some(num) = caps.get(1) {
                    if let Ok(n) = num.as_str().parse::<u32>() {
                        if !claims.contains(&n) {
                            claims.push(n);
                        }
                    }
                }
            }
        }
    }

    claims.sort_unstable();
    claims
}

/// 提取引用的现有技术
fn extract_cited_references(content: &str) -> Vec<String> {
    let mut refs = Vec::new();

    // 匹配对比文件
    let patterns = vec![
        r"(对比文件\d+[:：]\s*[A-Z]{2}\d+)",
        r"(对比文件\d+[:：]\s*CN\d+)",
        r"([A-Z]{2}\d+[A-Z]?)", // US1234567, EP1234567 等
        r"(CN\d{12,13})",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(content) {
                if let Some(reference) = caps.get(1) {
                    let ref_str = reference.as_str().trim().to_string();
                    if !refs.contains(&ref_str) {
                        refs.push(ref_str);
                    }
                }
            }
        }
    }

    refs
}

/// 计算严重程度
fn calculate_severity(matched_keywords: &[&&str], rejection_type: &RejectionType) -> String {
    let keyword_count = matched_keywords.len();

    match rejection_type {
        RejectionType::Novelty | RejectionType::Inventiveness => {
            if keyword_count >= 3 {
                "high".to_string()
            } else if keyword_count >= 2 {
                "medium".to_string()
            } else {
                "low".to_string()
            }
        }
        RejectionType::Formality => "low".to_string(),
        RejectionType::Clarity | RejectionType::Support => {
            if keyword_count >= 2 {
                "medium".to_string()
            } else {
                "low".to_string()
            }
        }
        _ => {
            if keyword_count >= 3 {
                "medium".to_string()
            } else {
                "low".to_string()
            }
        }
    }
}

/// 建议答复策略
fn suggest_response_strategy(rejection_type: &RejectionType, severity: &str) -> String {
    match rejection_type {
        RejectionType::Novelty | RejectionType::Inventiveness => {
            if severity == "high" {
                "amend".to_string() // 高严重程度建议修改
            } else {
                "argue".to_string() // 低严重程度建议争辩
            }
        }
        RejectionType::Support | RejectionType::Clarity => "amend".to_string(),
        RejectionType::Formality => "both".to_string(),
        _ => "argue".to_string(),
    }
}

/// 计算总体严重程度
pub(super) fn calculate_overall_severity(reasons: &[RejectionReason]) -> String {
    let high_count = reasons.iter().filter(|r| r.severity == "high").count();
    let medium_count = reasons.iter().filter(|r| r.severity == "medium").count();

    if high_count >= 2 || (high_count >= 1 && medium_count >= 2) {
        "high".to_string()
    } else if high_count >= 1 || medium_count >= 2 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

/// 生成摘要
fn generate_summary(reasons: &[RejectionReason], all_claims: &[u32]) -> String {
    if reasons.is_empty() {
        return "未检测到驳回理由".to_string();
    }

    let types: Vec<_> = reasons.iter().map(|r| r.rejection_type.clone()).collect();
    let claims_str = if all_claims.len() <= 3 {
        format!("{all_claims:?}")
    } else {
        format!("{:?}... (共{}项)", &all_claims[..3], all_claims.len())
    };

    format!(
        "检测到 {} 类驳回: {:?}，影响权利要求 {}",
        reasons.len(),
        types,
        claims_str
    )
}

/// 计算置信度
#[allow(clippy::cast_precision_loss)]
fn calculate_confidence(reasons: &[RejectionReason], content: &str) -> f64 {
    if reasons.is_empty() {
        return 0.3;
    }

    let base_confidence = 0.7;
    let keyword_bonus = reasons
        .iter()
        .map(|r| r.description.len() as f64 * 0.001)
        .sum::<f64>();
    let content_length_bonus = if content.len() > 500 { 0.1 } else { 0.0 };

    (base_confidence + keyword_bonus + content_length_bonus).min(0.95)
}
