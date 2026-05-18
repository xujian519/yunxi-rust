use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{strategy_name, ResponseStrategy};

/// 论点生成器驳回类型。
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentRejectionType {
    Novelty,
    Inventiveness,
    Utility,
    Support,
    Clarity,
    Scope,
    AmendmentScope,
    Unity,
    Formality,
    Other,
}

impl ArgumentRejectionType {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Novelty => "novelty",
            Self::Inventiveness => "inventiveness",
            Self::Utility => "utility",
            Self::Support => "support",
            Self::Clarity => "clarity",
            Self::Scope => "scope",
            Self::AmendmentScope => "amendment_scope",
            Self::Unity => "unity",
            Self::Formality => "formality",
            Self::Other => "other",
        }
    }
}

/// 论点生成器驳回理由输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArgumentRejectionReason {
    #[serde(rename = "type")]
    pub(crate) rejection_type: ArgumentRejectionType,
    #[serde(default)]
    pub(crate) severity: String,
    #[serde(default)]
    pub(crate) affected_claims: Vec<u32>,
    #[serde(default)]
    pub(crate) related_references: Vec<String>,
}

/// 论点生成器引用文献输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArgumentCitedReference {
    #[serde(default)]
    pub(crate) publication_number: String,
    #[serde(default)]
    pub(crate) title: String,
}

/// 论点生成器 OA 解析结果输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArgumentOAParseResult {
    #[serde(default)]
    pub(crate) rejection_reasons: Vec<ArgumentRejectionReason>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) rejection_types: Vec<ArgumentRejectionType>,
    #[serde(default)]
    pub(crate) affected_claims: Vec<u32>,
    #[serde(default)]
    pub(crate) cited_references: Vec<ArgumentCitedReference>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) patent_title: String, // 保留原因: 预留给未来在答复中显示或引用专利标题
}

/// 评分项（用于论点生成器）。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreItem {
    pub strategy: ResponseStrategy,
    pub score: f64,
    #[serde(default)]
    pub details: ScoreItemDetails,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScoreItemDetails {
    #[serde(default)]
    pub rejection_match: f64,
    #[serde(default)]
    pub historical_success: f64,
    #[serde(default)]
    pub risk_adjustment: f64,
}

/// 策略论点生成器输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyArgumentInput {
    pub parse_result: ArgumentOAParseResult,
    pub strategy: ResponseStrategy,
    #[serde(default)]
    pub scores: Vec<ScoreItem>,
    #[serde(default)]
    pub case_count: usize,
}

/// 论点输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ResponseArgumentOutput {
    pub(crate) category: String,
    pub(crate) argument: String,
    pub(crate) evidence: Vec<String>,
    pub(crate) target_rejection: String,
    pub(crate) strength: u32,
    pub(crate) precedents: Vec<String>,
}

/// 修改建议输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AmendmentSuggestionOutput {
    pub(crate) claim_number: u32,
    pub(crate) current_text: String,
    pub(crate) proposed_text: String,
    pub(crate) reason: String,
    pub(crate) amendment_type: String,
    pub(crate) expected_effect: String,
    pub(crate) adds_new_matter: bool,
}

/// 替代策略输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AlternativeStrategyOutput {
    pub(crate) strategy: ResponseStrategy,
    pub(crate) probability: f64,
    pub(crate) rationale: String,
}

/// 论点生成器整体输出。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ArgumentGeneratorOutput {
    arguments: Vec<ResponseArgumentOutput>,
    amendment_suggestions: Vec<AmendmentSuggestionOutput>,
    risks: Vec<String>,
    additional_evidence: Vec<String>,
    alternatives: Vec<AlternativeStrategyOutput>,
    rationale: String,
}

/// 论点模板。
struct ArgumentTemplate {
    category: &'static str,
    template: &'static str,
    strength: u32,
}

fn get_argument_templates(rejection_type: &ArgumentRejectionType) -> Vec<ArgumentTemplate> {
    match rejection_type {
        ArgumentRejectionType::Novelty => vec![
            ArgumentTemplate {
                category: "区别技术特征",
                template: "本申请权利要求{claims}与{reference}相比，至少存在以下区别技术特征：{features}。这些区别技术特征在{reference}中并未公开，也不属于本领域技术人员的公知常识。",
                strength: 4,
            },
            ArgumentTemplate {
                category: "技术效果",
                template: "上述区别技术特征带来了{effect}的技术效果，这是{reference}所未曾预期和实现的。",
                strength: 4,
            },
        ],
        ArgumentRejectionType::Inventiveness => vec![
            ArgumentTemplate {
                category: "非显而易见性",
                template: "{reference}虽然公开了相似的技术方案，但并未给出将{features}应用于本申请技术领域以解决{problem}的技术启示。",
                strength: 4,
            },
            ArgumentTemplate {
                category: "预料不到的技术效果",
                template: "本申请通过{features}的设置，实现了{effect}的技术效果，这对于本领域技术人员来说是预料不到的。",
                strength: 5,
            },
            ArgumentTemplate {
                category: "技术障碍",
                template: "本领域技术人员在面对{problem}时，存在{obstacle}的技术障碍，而本申请通过{features}成功克服了该障碍。",
                strength: 4,
            },
        ],
        ArgumentRejectionType::Support => vec![ArgumentTemplate {
            category: "充分公开",
            template: "说明书在{section}部分对{features}进行了详细描述，本领域技术人员根据说明书公开的内容能够实现该技术方案。",
            strength: 3,
        }],
        ArgumentRejectionType::Clarity => vec![ArgumentTemplate {
            category: "保护范围明确",
            template: "权利要求{claims}中{features}的表述是清晰的，其保护范围是明确的，本领域技术人员能够理解其含义。",
            strength: 3,
        }],
        ArgumentRejectionType::Scope => vec![ArgumentTemplate {
            category: "必要技术特征",
            template: "权利要求{claims}包含了实现{function}所需的全部必要技术特征，保护范围适当。",
            strength: 3,
        }],
        ArgumentRejectionType::Unity => vec![ArgumentTemplate {
            category: "单一发明构思",
            template: "各项权利要求属于同一个总的发明构思，因为它们都基于{concept}这一技术特征，解决了{problem}这一技术问题。",
            strength: 3,
        }],
        ArgumentRejectionType::Formality => vec![ArgumentTemplate {
            category: "形式问题修正",
            template: "已对权利要求{claims}中的形式问题进行修正，修正后的表述符合专利法要求。",
            strength: 2,
        }],
        ArgumentRejectionType::Utility => vec![ArgumentTemplate {
            category: "实用性",
            template: "本申请的技术方案能够制造和使用，并产生了积极效果，具备实用性。",
            strength: 3,
        }],
        ArgumentRejectionType::AmendmentScope => vec![ArgumentTemplate {
            category: "修改依据",
            template: "修改内容来源于说明书{section}的记载，未超出原说明书和权利要求书记载的范围。",
            strength: 3,
        }],
        ArgumentRejectionType::Other => vec![ArgumentTemplate {
            category: "一般性答辩",
            template: "针对审查员指出的问题，申请人认为...",
            strength: 2,
        }],
    }
}

/// 生成关键论点。
pub(crate) fn generate_key_arguments(
    parse_result: &ArgumentOAParseResult,
    _strategy: ResponseStrategy,
) -> Vec<ResponseArgumentOutput> {
    let mut result = Vec::new();
    for rejection in &parse_result.rejection_reasons {
        let templates = get_argument_templates(&rejection.rejection_type);
        for tmpl in templates {
            let argument = customize_template(tmpl.template, rejection, parse_result);
            let evidence = generate_evidence(rejection, parse_result);
            result.push(ResponseArgumentOutput {
                category: tmpl.category.to_string(),
                argument,
                evidence,
                target_rejection: rejection.rejection_type.as_str().to_string(),
                strength: tmpl.strength,
                precedents: vec![],
            });
        }
    }
    result
}

/// 自定义模板占位符。
pub(crate) fn customize_template(
    template: &str,
    rejection: &ArgumentRejectionReason,
    parse_result: &ArgumentOAParseResult,
) -> String {
    let mut result = template.to_string();

    let claims_str = if rejection.affected_claims.is_empty() {
        "相关".to_string()
    } else {
        rejection
            .affected_claims
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    };
    result = result.replace("{claims}", &claims_str);

    let reference = if !rejection.related_references.is_empty() {
        rejection.related_references[0].clone()
    } else if !parse_result.cited_references.is_empty() {
        parse_result.cited_references[0].publication_number.clone()
    } else {
        "对比文件".to_string()
    };
    result = result.replace("{reference}", &reference);

    // Derive fallback values for additional placeholders from available data
    let features = if rejection.affected_claims.is_empty() {
        "区别技术特征".to_string()
    } else {
        format!("权利要求{claims_str}记载的技术特征")
    };
    result = result.replace("{features}", &features);

    let effect = "显著".to_string();
    result = result.replace("{effect}", &effect);

    let problem = "现有技术中存在的技术问题".to_string();
    result = result.replace("{problem}", &problem);

    let obstacle = "技术实现".to_string();
    result = result.replace("{obstacle}", &obstacle);

    let section = "具体实施方式".to_string();
    result = result.replace("{section}", &section);

    let function = "发明目的".to_string();
    result = result.replace("{function}", &function);

    let concept = "发明构思".to_string();
    result = result.replace("{concept}", &concept);

    result
}

/// 生成证据。
fn generate_evidence(
    rejection: &ArgumentRejectionReason,
    parse_result: &ArgumentOAParseResult,
) -> Vec<String> {
    let mut evidence = Vec::new();
    if !rejection.related_references.is_empty() {
        for ref_num in &rejection.related_references {
            if let Some(ref_data) = parse_result
                .cited_references
                .iter()
                .find(|r| r.publication_number == *ref_num)
            {
                evidence.push(format!("对比文件{}: {}", ref_num, ref_data.title));
            }
        }
    }
    evidence
}

/// 生成修改建议。
pub(crate) fn generate_amendment_suggestions(
    parse_result: &ArgumentOAParseResult,
    strategy: ResponseStrategy,
) -> Vec<AmendmentSuggestionOutput> {
    if strategy == ResponseStrategy::Argue {
        return vec![];
    }

    let mut suggestions = Vec::new();
    for rejection in &parse_result.rejection_reasons {
        if let Some(s) = generate_amendment_for_rejection(rejection) {
            suggestions.push(s);
        }
    }
    suggestions
}

/// 为特定驳回理由生成修改建议。
fn generate_amendment_for_rejection(
    rejection: &ArgumentRejectionReason,
) -> Option<AmendmentSuggestionOutput> {
    match rejection.rejection_type {
        ArgumentRejectionType::Novelty | ArgumentRejectionType::Inventiveness => {
            if rejection.affected_claims.is_empty() {
                None
            } else {
                Some(AmendmentSuggestionOutput {
                    claim_number: rejection.affected_claims[0],
                    current_text: "（原文）".to_string(),
                    proposed_text: "（添加区别技术特征）".to_string(),
                    reason: "通过添加区别技术特征来克服新颖性/创造性问题".to_string(),
                    amendment_type: "modify".to_string(),
                    expected_effect: "使权利要求与对比文件明确区分".to_string(),
                    adds_new_matter: false,
                })
            }
        }
        ArgumentRejectionType::Clarity | ArgumentRejectionType::Scope => {
            if rejection.affected_claims.is_empty() {
                None
            } else {
                Some(AmendmentSuggestionOutput {
                    claim_number: rejection.affected_claims[0],
                    current_text: "（原文）".to_string(),
                    proposed_text: "（进一步限定技术特征）".to_string(),
                    reason: "通过进一步限定来明确保护范围".to_string(),
                    amendment_type: "modify".to_string(),
                    expected_effect: "使权利要求的保护范围更加清晰明确".to_string(),
                    adds_new_matter: false,
                })
            }
        }
        ArgumentRejectionType::Formality => Some(AmendmentSuggestionOutput {
            claim_number: 1,
            current_text: "（原文）".to_string(),
            proposed_text: "（修正后的表述）".to_string(),
            reason: "修正形式缺陷".to_string(),
            amendment_type: "modify".to_string(),
            expected_effect: "符合专利法形式要求".to_string(),
            adds_new_matter: false,
        }),
        _ => None,
    }
}

/// 识别风险。
pub(crate) fn identify_risks(
    parse_result: &ArgumentOAParseResult,
    strategy: ResponseStrategy,
) -> Vec<String> {
    let mut risks = Vec::new();

    let high_severity = parse_result
        .rejection_reasons
        .iter()
        .filter(|r| r.severity == "high")
        .count();
    if high_severity > 0 {
        risks.push(format!(
            "存在{high_severity}项高严重程度驳回理由，可能较难克服"
        ));
    }

    match strategy {
        ResponseStrategy::Argue => {
            let has_formality = parse_result
                .rejection_reasons
                .iter()
                .any(|r| r.rejection_type == ArgumentRejectionType::Formality);
            if has_formality {
                risks.push("存在形式缺陷，建议一并修改".to_string());
            }
        }
        ResponseStrategy::Amend => {
            risks.push("修改可能导致保护范围缩小".to_string());
            risks.push("需要注意避免引入新事项".to_string());
        }
        _ => {}
    }

    if parse_result.affected_claims.len() > 5 {
        risks.push("涉及权利要求数量较多，答复难度较大".to_string());
    }

    if parse_result.cited_references.len() > 3 {
        risks.push("引用对比文件较多，需要逐一针对性答辩".to_string());
    }

    risks
}

/// 建议补充证据。
pub(crate) fn suggest_additional_evidence(parse_result: &ArgumentOAParseResult) -> Vec<String> {
    let mut evidence = Vec::new();
    for rejection in &parse_result.rejection_reasons {
        match rejection.rejection_type {
            ArgumentRejectionType::Inventiveness => {
                evidence.push("补充实验数据证明技术效果".to_string());
                evidence.push("提供技术对比表格".to_string());
            }
            ArgumentRejectionType::Novelty => {
                evidence.push("准备区别特征对比表".to_string());
            }
            ArgumentRejectionType::Utility => {
                evidence.push("提供样品或试用报告".to_string());
                evidence.push("附上产业化证明材料".to_string());
            }
            _ => {}
        }
    }
    evidence.sort();
    evidence.dedup();
    evidence
}

/// 生成替代策略。
pub(crate) fn generate_alternatives(
    scores: &[ScoreItem],
    selected_strategy: ResponseStrategy,
) -> Vec<AlternativeStrategyOutput> {
    scores
        .iter()
        .filter(|s| s.strategy != selected_strategy && s.strategy != ResponseStrategy::Abandon)
        .take(2)
        .map(|s| {
            let parts: Vec<&str> = {
                let mut p = Vec::new();
                if s.details.rejection_match > 70.0 {
                    p.push("与驳回理由匹配度较高");
                }
                if s.details.historical_success > 70.0 {
                    p.push("历史成功率较高");
                }
                if s.details.risk_adjustment > 70.0 {
                    p.push("风险较低");
                }
                p
            };
            let rationale = if parts.is_empty() {
                "可作为备选方案".to_string()
            } else {
                parts.join("；")
            };
            AlternativeStrategyOutput {
                strategy: s.strategy,
                probability: s.score,
                rationale,
            }
        })
        .collect()
}

/// 生成推荐理由。
pub(crate) fn generate_rationale(strategy_score: &ScoreItem, case_count: usize) -> String {
    let strategy_name_str = strategy_name(strategy_score.strategy);
    let mut parts = vec![format!("推荐采用{strategy_name_str}")];

    if strategy_score.details.rejection_match > 75.0 {
        parts.push("，该策略与审查意见指出的驳回理由高度匹配".to_string());
    }

    if strategy_score.details.historical_success > 70.0 && case_count > 0 {
        parts.push(format!(
            "，基于{case_count}个相似案例的分析，该策略具有较高的成功率"
        ));
    }

    if strategy_score.details.risk_adjustment > 70.0 {
        parts.push("，且风险可控".to_string());
    }

    parts.join("") + "。"
}

/// 执行策略论点生成。
pub fn execute_strategy_arguments(input: &StrategyArgumentInput) -> Result<Value, String> {
    let arguments = generate_key_arguments(&input.parse_result, input.strategy);
    let amendment_suggestions = generate_amendment_suggestions(&input.parse_result, input.strategy);
    let risks = identify_risks(&input.parse_result, input.strategy);
    let additional_evidence = suggest_additional_evidence(&input.parse_result);
    let alternatives = generate_alternatives(&input.scores, input.strategy);

    let rationale = if let Some(best) = input.scores.first() {
        generate_rationale(best, input.case_count)
    } else {
        format!("推荐采用{}", strategy_name(input.strategy))
    };

    let output = ArgumentGeneratorOutput {
        arguments,
        amendment_suggestions,
        risks,
        additional_evidence,
        alternatives,
        rationale,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}
