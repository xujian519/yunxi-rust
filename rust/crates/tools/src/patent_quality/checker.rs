use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::scorer::{default_check_level, QualityClaim, QualitySpec};

/// 质量检查输入（对齐 TS `QualityCheckInput`）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityCheckerInput {
    pub claims: Vec<QualityClaim>,
    pub specification: QualitySpec,
    pub patent_type: String,
    pub invention_title: String,
    #[serde(default = "default_check_level")]
    pub check_level: u8,
}

/// 质量检查输出
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerOutput {
    overall_quality: f64,
    quality_level: &'static str,
    completeness_score: f64,
    issues: Vec<CheckerIssue>,
    rules_applied: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerIssue {
    rule_id: &'static str,
    category: &'static str,
    severity: &'static str,
    description: String,
    location: String,
    suggestion: String,
}

// --- 规则定义 ---

type CheckerRuleFn = fn(&QualityCheckerInput) -> Option<CheckerIssue>;

struct CheckerRule {
    id: &'static str,
    min_level: u8, // 1/2/3，低于此级别不执行
    check: CheckerRuleFn,
}

// --- 规则实现（移植自 TS QualityRules.ts） ---

fn checker_claim_001_first_independent(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    let first = input.claims.first()?;
    if first.r#type == "independent" {
        None
    } else {
        Some(CheckerIssue {
            rule_id: "CLAIM_001",
            category: "权利要求",
            severity: "critical",
            description: "第一项权利要求必须是独立权利要求".into(),
            location: format!("权利要求{}", first.number),
            suggestion: "将第一项权利要求改为独立权利要求，或调整权利要求顺序".into(),
        })
    }
}

fn checker_claim_002_dependent_ref(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    for claim in &input.claims {
        if claim.r#type == "dependent" {
            let dep = claim.depends_on.unwrap_or(0);
            if dep == 0 || dep >= claim.number {
                return Some(CheckerIssue {
                    rule_id: "CLAIM_002",
                    category: "权利要求",
                    severity: "high",
                    description: format!("权利要求{}的引用关系无效", claim.number),
                    location: format!("权利要求{}", claim.number),
                    suggestion: format!(
                        "检查权利要求{}的引用关系，确保引用在先的权利要求",
                        claim.number
                    ),
                });
            }
        }
    }
    None
}

fn checker_claim_003_length(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    for claim in &input.claims {
        if claim.content.len() > 500 {
            return Some(CheckerIssue {
                rule_id: "CLAIM_003",
                category: "权利要求",
                severity: "medium",
                description: format!("权利要求{}过长（{}字）", claim.number, claim.content.len()),
                location: format!("权利要求{}", claim.number),
                suggestion: "建议将部分技术特征拆分到从属权利要求中".into(),
            });
        }
    }
    None
}

fn checker_spec_001_tech_field(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    let field = input.specification.technical_field.as_deref().unwrap_or("");
    if field.len() < 20 {
        Some(CheckerIssue {
            rule_id: "SPEC_001",
            category: "说明书",
            severity: "high",
            description: "技术领域描述不充分".into(),
            location: "技术领域".into(),
            suggestion: "技术领域应明确说明发明所属或直接应用的具体技术领域".into(),
        })
    } else {
        None
    }
}

fn checker_spec_002_background(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    let bg = input.specification.background_art.as_deref().unwrap_or("");
    if bg.len() < 50 {
        Some(CheckerIssue {
            rule_id: "SPEC_002",
            category: "说明书",
            severity: "high",
            description: "背景技术描述不充分".into(),
            location: "背景技术".into(),
            suggestion: "背景技术应介绍现有技术及其存在的问题".into(),
        })
    } else {
        None
    }
}

fn checker_spec_003_invention(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    let inv = input
        .specification
        .invention_content
        .as_deref()
        .unwrap_or("");
    if inv.len() < 100 {
        Some(CheckerIssue {
            rule_id: "SPEC_003",
            category: "说明书",
            severity: "high",
            description: "发明内容描述不充分".into(),
            location: "发明内容".into(),
            suggestion: "发明内容应清楚、完整地描述要解决的技术问题、技术方案和有益效果".into(),
        })
    } else {
        None
    }
}

fn checker_lang_001_incomplete_end(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    for claim in &input.claims {
        if claim.content.ends_with('，') || claim.content.ends_with('、') {
            return Some(CheckerIssue {
                rule_id: "LANG_002",
                category: "语言表达",
                severity: "medium",
                description: format!("权利要求{}结尾不完整", claim.number),
                location: format!("权利要求{}", claim.number),
                suggestion: "确保权利要求以句号结尾，表达完整".into(),
            });
        }
    }
    None
}

fn checker_lang_002_vague(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    let vague = ["大约", "左右", "可能", "也许", "大概", "约"];
    for claim in &input.claims {
        for term in vague {
            if claim.content.contains(term) {
                return Some(CheckerIssue {
                    rule_id: "LANG_003",
                    category: "语言表达",
                    severity: "medium",
                    description: format!("存在模糊表达\"{term}\""),
                    location: format!("权利要求{}", claim.number),
                    suggestion: "使用精确的技术表述，避免模糊词汇".into(),
                });
            }
        }
    }
    None
}

fn checker_legal_001_single_claim(input: &QualityCheckerInput) -> Option<CheckerIssue> {
    if input.claims.is_empty() {
        return Some(CheckerIssue {
            rule_id: "LEGAL_001",
            category: "法律要求",
            severity: "critical",
            description: "权利要求不能为空".into(),
            location: "权利要求书".into(),
            suggestion: "至少需要一个独立权利要求".into(),
        });
    }
    None
}

/// 完整性评分（简化版，对齐 TS checkCompleteness）
fn checker_completeness_score(input: &QualityCheckerInput) -> f64 {
    let mut score: f64 = 0.0;

    if !input.invention_title.is_empty() {
        score += 5.0;
    }
    if !input.claims.is_empty() {
        let has_independent = input.claims.iter().any(|c| c.r#type == "independent");
        if has_independent {
            score += 20.0;
        }
        if input.claims.len() >= 2 {
            score += 10.0;
        }
        if input.claims.len() >= 5 {
            score += 5.0;
        }
    }
    if let Some(field) = &input.specification.technical_field {
        if field.len() > 10 {
            score += 12.0;
        }
    }
    if let Some(bg) = &input.specification.background_art {
        if bg.len() > 20 {
            score += 12.0;
        }
    }
    if let Some(inv) = &input.specification.invention_content {
        if inv.len() > 50 {
            score += 12.0;
        }
    }
    if let Some(emb) = &input.specification.embodiment {
        if emb.len() > 100 {
            score += 12.0;
        }
    }

    score.min(100.0)
}

fn checker_quality_level(score: f64) -> &'static str {
    if score >= 90.0 {
        "excellent"
    } else if score >= 75.0 {
        "good"
    } else if score >= 60.0 {
        "fair"
    } else {
        "poor"
    }
}

/// 所有规则列表
fn checker_all_rules() -> Vec<CheckerRule> {
    vec![
        CheckerRule {
            id: "CLAIM_001",
            min_level: 1,
            check: checker_claim_001_first_independent,
        },
        CheckerRule {
            id: "CLAIM_002",
            min_level: 1,
            check: checker_claim_002_dependent_ref,
        },
        CheckerRule {
            id: "CLAIM_003",
            min_level: 2,
            check: checker_claim_003_length,
        },
        CheckerRule {
            id: "SPEC_001",
            min_level: 1,
            check: checker_spec_001_tech_field,
        },
        CheckerRule {
            id: "SPEC_002",
            min_level: 1,
            check: checker_spec_002_background,
        },
        CheckerRule {
            id: "SPEC_003",
            min_level: 1,
            check: checker_spec_003_invention,
        },
        CheckerRule {
            id: "LANG_002",
            min_level: 2,
            check: checker_lang_001_incomplete_end,
        },
        CheckerRule {
            id: "LANG_003",
            min_level: 2,
            check: checker_lang_002_vague,
        },
        CheckerRule {
            id: "LEGAL_001",
            min_level: 1,
            check: checker_legal_001_single_claim,
        },
    ]
}

fn checker_execute(input: &QualityCheckerInput) -> CheckerOutput {
    let level = input.check_level;
    let rules = checker_all_rules();

    let mut issues = Vec::new();
    let mut rules_applied = Vec::new();

    for rule in &rules {
        if level >= rule.min_level {
            rules_applied.push(rule.id);
            if let Some(issue) = (rule.check)(input) {
                issues.push(issue);
            }
        }
    }

    let comp = checker_completeness_score(input);

    // 扣分：critical -20, high -10, medium -5
    let penalty: f64 = issues
        .iter()
        .map(|i| match i.severity {
            "critical" => 20.0,
            "high" => 10.0,
            "medium" => 5.0,
            _ => 1.0,
        })
        .sum();
    let overall = (comp - penalty).max(0.0);

    CheckerOutput {
        overall_quality: (overall * 10.0).round() / 10.0,
        quality_level: checker_quality_level(overall),
        completeness_score: comp,
        issues,
        rules_applied,
    }
}

/// 基于规则的专利质量检查工具（无 LLM）。
///
/// 检查权利要求结构、说明书充分性、语言表达规范性等。
pub fn execute_quality_checker(input: &QualityCheckerInput) -> Result<Value, String> {
    let output = checker_execute(input);
    serde_json::to_value(output).map_err(|e| format!("序列化失败: {e}"))
}
