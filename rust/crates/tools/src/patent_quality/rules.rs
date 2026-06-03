use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

use crate::patent_quality::types::{extract_keywords, QualityScorerInput, ScorerIssue};

pub(crate) fn scorer_rule_claim_001(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let mut issues = Vec::new();
    if let Some(first) = input.claims.first() {
        if first.r#type != "independent" {
            issues.push(ScorerIssue {
                category: "权利要求".into(),
                sub_category: Some("结构".into()),
                severity: "critical".into(),
                description: "第一项权利要求必须是独立权利要求".into(),
                location: Some(format!("权利要求{}", first.number)),
                rule_reference: Some("A26.4".into()),
                suggestion: "将第一项权利要求改为独立权利要求，或调整权利要求顺序".into(),
            });
        }
    }
    issues
}

pub(crate) fn scorer_rule_claim_002(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let mut issues = Vec::new();
    for claim in &input.claims {
        if claim.r#type == "dependent" {
            let invalid = match claim.depends_on {
                Some(d) => d >= claim.number,
                None => true,
            };
            if invalid {
                issues.push(ScorerIssue {
                    category: "权利要求".into(),
                    sub_category: Some("引用".into()),
                    severity: "high".into(),
                    description: format!("权利要求{}的引用关系无效", claim.number),
                    location: Some(format!("权利要求{}", claim.number)),
                    rule_reference: Some("A26.4".into()),
                    suggestion: format!(
                        "检查权利要求{}的引用关系，确保引用在先的权利要求",
                        claim.number
                    ),
                });
            }
        }
    }
    issues
}

/// CLAIM_003: 权利要求长度检查。
/// 阈值 500 字符：超过此长度的权利要求通常包含过多技术特征，
/// 影响清晰性和可执行性。建议拆分为从属权利要求。
pub(crate) fn scorer_rule_claim_003(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let mut issues = Vec::new();
    for claim in &input.claims {
        if claim.content.len() > 500 {
            issues.push(ScorerIssue {
                category: "权利要求".into(),
                sub_category: Some("表达".into()),
                severity: "medium".into(),
                description: format!("权利要求{}过长（{}字）", claim.number, claim.content.len()),
                location: Some(format!("权利要求{}", claim.number)),
                rule_reference: None,
                suggestion: "建议将部分技术特征拆分到从属权利要求中".into(),
            });
        }
    }
    issues
}

/// CLAIM_004: 技术术语一致性检查。
/// 阈值 3 种不同表述：同一专利中若出现 4 种以上对同一技术主题的不同称呼
///（如"装置/方法/系统/设备"混用），可能导致保护范围模糊。
pub(crate) fn scorer_rule_claim_004(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?:其特征在于|其中)[^，。]{0,50}?(?:装置|方法|系统|设备)").unwrap()
    });
    let terms: HashSet<String> = input
        .claims
        .iter()
        .flat_map(|c| RE.find_iter(&c.content).map(|m| m.as_str().to_string()))
        .collect();
    if terms.len() > 3 {
        vec![ScorerIssue {
            category: "权利要求".into(),
            sub_category: Some("术语".into()),
            severity: "medium".into(),
            description: "技术术语使用不一致".into(),
            location: None,
            rule_reference: None,
            suggestion: "统一技术术语的使用，确保同一概念使用相同表述".into(),
        }]
    } else {
        Vec::new()
    }
}

/// SPEC_001: 技术领域完整性。
/// 阈值 20 字符：技术领域应至少说明一级技术领域（如"数据处理"约 4 字）
/// 和二级应用领域（如"图像识别"约 4 字），加上连接词后约 20 字为底线。
pub(crate) fn scorer_rule_spec_001(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let field = input.specification.technical_field.as_deref().unwrap_or("");
    if field.len() < 20 {
        vec![ScorerIssue {
            category: "说明书".into(),
            sub_category: Some("技术领域".into()),
            severity: "high".into(),
            description: "技术领域描述不充分".into(),
            location: Some("技术领域".into()),
            rule_reference: Some("A26.3".into()),
            suggestion: "技术领域应明确说明发明所属或直接应用的具体技术领域".into(),
        }]
    } else {
        Vec::new()
    }
}

/// SPEC_002: 背景技术完整性。
/// 阈值 50 字符：需简要介绍现有技术 + 指出其不足，过少则无法建立对比基础。
pub(crate) fn scorer_rule_spec_002(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let bg = input.specification.background_art.as_deref().unwrap_or("");
    if bg.len() < 50 {
        vec![ScorerIssue {
            category: "说明书".into(),
            sub_category: Some("背景技术".into()),
            severity: "high".into(),
            description: "背景技术描述不充分".into(),
            location: Some("背景技术".into()),
            rule_reference: Some("A26.3".into()),
            suggestion: "背景技术应介绍现有技术及其存在的问题".into(),
        }]
    } else {
        Vec::new()
    }
}

/// SPEC_003: 发明内容完整性。
/// 阈值 100 字符：需概括技术问题、方案、效果三要素，少于一句话长度即不充分。
pub(crate) fn scorer_rule_spec_003(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let ic = input
        .specification
        .invention_content
        .as_deref()
        .unwrap_or("");
    if ic.len() < 100 {
        vec![ScorerIssue {
            category: "说明书".into(),
            sub_category: Some("发明内容".into()),
            severity: "high".into(),
            description: "发明内容描述不充分".into(),
            location: Some("发明内容".into()),
            rule_reference: Some("A26.3".into()),
            suggestion: "发明内容应清楚、完整地描述要解决的技术问题、技术方案和有益效果".into(),
        }]
    } else {
        Vec::new()
    }
}

/// SPEC_004: 具体实施方式充分性。
/// 阈值 200 字符：至少需描述一个完整实施例的核心步骤，少于一段即视为不充分。
pub(crate) fn scorer_rule_spec_004(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let emb = input.specification.embodiment.as_deref().unwrap_or("");
    if emb.len() < 200 {
        vec![ScorerIssue {
            category: "说明书".into(),
            sub_category: Some("具体实施方式".into()),
            severity: "high".into(),
            description: "具体实施方式不充分".into(),
            location: Some("具体实施方式".into()),
            rule_reference: Some("A26.3".into()),
            suggestion: "具体实施方式应详细描述至少一个实施例，使所属领域技术人员能够实现".into(),
        }]
    } else {
        Vec::new()
    }
}

/// SPEC_005: 权利要求支持性检查。
/// 阈值 0.8（80%）：权利要求中至少 80% 的技术特征应在实施方式中有对应描述，
/// 否则可能违反专利法第 26 条第 4 款（权利要求应以说明书为依据）。
pub(crate) fn scorer_rule_spec_005(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?:包括|包含|设有|配置)[^，。]{1,30}?").unwrap());
    let Some(emb) = &input.specification.embodiment else {
        return Vec::new();
    };
    let features: HashSet<String> = input
        .claims
        .iter()
        .flat_map(|c| RE.find_iter(&c.content).map(|m| m.as_str().to_string()))
        .collect();
    if features.is_empty() {
        return Vec::new();
    }
    let supported = features
        .iter()
        .filter(|f| {
            let prefix: String = f.chars().take(10).collect();
            emb.contains(&prefix)
        })
        .count();
    if (supported as f64 / features.len() as f64) < 0.8 {
        vec![ScorerIssue {
            category: "说明书".into(),
            sub_category: Some("支持性".into()),
            severity: "high".into(),
            description: "说明书对权利要求的支持不足".into(),
            location: Some("具体实施方式".into()),
            rule_reference: Some("A26.4".into()),
            suggestion: "在具体实施方式中补充描述权利要求中的技术特征".into(),
        }]
    } else {
        Vec::new()
    }
}

pub(crate) fn scorer_rule_lang_001(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let mut issues = Vec::new();
    for claim in &input.claims {
        if claim.content.contains("。。")
            || claim.content.contains("，，")
            || claim.content.contains("、，")
            || claim.content.contains(",,")
            || claim.content.contains("..")
        {
            issues.push(ScorerIssue {
                category: "语言表达".into(),
                sub_category: Some("标点符号".into()),
                severity: "low".into(),
                description: "存在标点符号使用错误".into(),
                location: Some(format!("权利要求{}", claim.number)),
                rule_reference: None,
                suggestion: "检查并修正标点符号的使用".into(),
            });
        }
    }
    issues
}

pub(crate) fn scorer_rule_lang_002(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let mut issues = Vec::new();
    for claim in &input.claims {
        if claim.content.ends_with('，') || claim.content.ends_with('、') {
            issues.push(ScorerIssue {
                category: "语言表达".into(),
                sub_category: Some("表达".into()),
                severity: "medium".into(),
                description: format!("权利要求{}结尾不完整", claim.number),
                location: Some(format!("权利要求{}", claim.number)),
                rule_reference: None,
                suggestion: "确保权利要求以句号结尾，表达完整".into(),
            });
        }
    }
    issues
}

pub(crate) fn scorer_rule_lang_003(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let vague = ["大约", "左右", "可能", "也许", "大概", "约"];
    let mut issues = Vec::new();
    for claim in &input.claims {
        for term in &vague {
            if claim.content.contains(term) {
                issues.push(ScorerIssue {
                    category: "语言表达".into(),
                    sub_category: Some("精确性".into()),
                    severity: "medium".into(),
                    description: format!("存在模糊表达\"{term}\""),
                    location: Some(format!("权利要求{}", claim.number)),
                    rule_reference: None,
                    suggestion: "使用精确的技术表述，避免模糊词汇".into(),
                });
                break;
            }
        }
    }
    issues
}

/// LEGAL_001: 单一性检查。
/// 阈值 0.3（30% 关键词重叠）：基于 Jaccard-like 系数，低于此值认为独立权利要求
/// 之间缺乏共同技术特征，可能违反专利法第 31 条第 1 款（单一性要求）。
pub(crate) fn scorer_rule_legal_001(input: &QualityScorerInput) -> Vec<ScorerIssue> {
    let independent: Vec<&crate::patent_quality::types::QualityClaim> = input
        .claims
        .iter()
        .filter(|c| c.r#type == "independent")
        .collect();
    if independent.len() <= 1 {
        return Vec::new();
    }
    let first_kw = extract_keywords(&independent[0].content);
    let has_unity = independent[1..]
        .iter()
        .all(|c| {
            crate::patent_quality::types::calculate_keyword_overlap(
                &first_kw,
                &extract_keywords(&c.content),
            ) >= 0.3
        });
    if has_unity {
        return Vec::new();
    }
    vec![ScorerIssue {
        category: "法律要求".into(),
        sub_category: Some("单一性".into()),
        severity: "high".into(),
        description: "可能存在单一性问题".into(),
        location: None,
        rule_reference: Some("A31.1".into()),
        suggestion: "检查各独立权利要求是否属于一个总的发明构思".into(),
    }]
}

type ScorerRuleFn = fn(&QualityScorerInput) -> Vec<ScorerIssue>;

pub(crate) fn scorer_apply_rules(input: &QualityScorerInput, check_level: u8) -> Vec<ScorerIssue> {
    static RULES: &[(u8, ScorerRuleFn)] = &[
        (1, scorer_rule_claim_001), // critical
        (1, scorer_rule_claim_002), // high
        (2, scorer_rule_claim_003), // medium
        (2, scorer_rule_claim_004), // medium
        (1, scorer_rule_spec_001),  // high
        (1, scorer_rule_spec_002),  // high
        (1, scorer_rule_spec_003),  // high
        (1, scorer_rule_spec_004),  // high
        (1, scorer_rule_spec_005),  // high
        (3, scorer_rule_lang_001),  // low
        (2, scorer_rule_lang_002),  // medium
        (2, scorer_rule_lang_003),  // medium
        (1, scorer_rule_legal_001), // high
    ];
    RULES
        .iter()
        .filter(|(min_lvl, _)| check_level >= *min_lvl || check_level == 0)
        .flat_map(|(_, rule)| rule(input))
        .collect()
}