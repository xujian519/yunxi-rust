// 保护客体检查

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 保护客体检查输入。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubjectMatterInput {
    #[allow(dead_code)]
    pub(super) invention_title: String, // 保留原因: 公共API字段，预留给显示使用
    pub(super) claims: Vec<SmClaimInput>,
    #[serde(default)]
    pub(super) specification: Option<SmSpecInput>,
    #[allow(dead_code)]
    pub(super) patent_type: String, // 保留原因: 公共API字段，预留给不同类型专利的差异化检查
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmClaimInput {
    #[serde(rename = "type")]
    pub(super) claim_type: String,
    pub(super) number: u32,
    pub(super) content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmSpecInput {
    #[serde(default)]
    #[allow(dead_code)]
    pub(super) technical_field: Option<String>, // 保留原因: 预留给技术领域匹配检查
    #[serde(default)]
    pub(super) background_art: Option<String>,
    #[serde(default)]
    pub(super) invention_content: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmOutput {
    pub(super) article2_invention_definition: SmArticle2Check,
    pub(super) article25_exclusions: SmArticle25Check,
    pub(super) technical_solution_analysis: SmTechSolutionAnalysis,
    pub(super) intellectual_activity_check: SmIntellectualActivityCheck,
    pub(super) legality_check: SmLegalityCheck,
    pub(super) overall_report: SmOverallReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmArticle2Check {
    pub(super) passed: bool,
    pub(super) is_technical_solution: bool,
    pub(super) issues: Vec<SmIssueItem>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmIssueItem {
    pub(super) claim_number: u32,
    pub(super) issue: String,
    pub(super) suggestion: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmArticle25Check {
    pub(super) passed: bool,
    pub(super) non_protectable_matters: Vec<SmNonProtectableMatter>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmNonProtectableMatter {
    pub(super) r#type: String,
    pub(super) type_name: String,
    pub(super) reason: String,
    pub(super) related_claims: Vec<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmTechSolutionAnalysis {
    pub(super) independent_claims_analysis: Vec<SmClaimAnalysis>,
    pub(super) has_technical_solution: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmClaimAnalysis {
    pub(super) is_technical_solution: bool,
    pub(super) technical_features: Vec<String>,
    pub(super) technical_problem: Option<String>,
    pub(super) technical_effect: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmIntellectualActivityCheck {
    pub(super) has_intellectual_activity_rules: bool,
    pub(super) detected_rules: Vec<SmDetectedRule>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmDetectedRule {
    pub(super) claim_number: u32,
    pub(super) rule_type: String,
    pub(super) description: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmLegalityCheck {
    pub(super) passed: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) illegal_content: Vec<SmIllegalContent>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmIllegalContent {
    pub(super) claim_number: u32,
    pub(super) content: String,
    pub(super) reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SmOverallReport {
    pub(super) passed: bool,
    pub(super) total_issues: usize,
    pub(super) critical_issues: usize,
    pub(super) is_protectable_subject_matter: bool,
    pub(super) recommendations: Vec<String>,
}

macro_rules! sm_regex {
    ($ident:ident, $pattern:expr) => {
        static $ident: std::sync::LazyLock<Regex> =
            std::sync::LazyLock::new(|| Regex::new($pattern).unwrap());
    };
}

sm_regex!(
    RE_IA_1,
    r"(?:方法|步骤).*(?:计算|运算|统计|分析|推理|判断|决策)"
);
sm_regex!(RE_IA_2, r"(?:规则|方法|模式|算法|模型).*(?:管理|控制|优化)");
sm_regex!(RE_IA_3, r"(?:游戏|竞赛|比赛).*(?:规则|方法)");
sm_regex!(RE_IA_4, r"(?:商业|经营|营销).*(?:模式|方法|策略)");
sm_regex!(RE_IA_5, r"(?:流程|程序).*(?:管理|审批|审核)");
sm_regex!(
    RE_IA_ALL,
    r"(?x)
    (?:方法|步骤).*(?:计算|运算|统计|分析|推理|判断|决策)
    | (?:规则|方法|模式|算法|模型).*(?:管理|控制|优化)
    | (?:游戏|竞赛|比赛).*(?:规则|方法)
    | (?:商业|经营|营销).*(?:模式|方法|策略)
    | (?:流程|程序).*(?:管理|审批|审核)"
);
sm_regex!(RE_MD_1, r"(?:疾病|病症|病情).*(?:诊断|检查|筛查|检测)");
sm_regex!(RE_MD_2, r"(?:手术|治疗|疗法|医治).*(?:方法|方案)");
sm_regex!(RE_MD_3, r"(?:药物|药品).*(?:配方|组合物).*(?:用于|治疗)");
sm_regex!(RE_MD_4, r"(?:针灸|按摩|理疗).*(?:方法)");
sm_regex!(RE_LEG_GAMBLING, r"(?:赌博|博彩|赌场)");
sm_regex!(RE_LEG_DRUGS, r"(?:毒品|违禁品|非法药物)");
sm_regex!(RE_LEG_FRAUD, r"(?:诈骗|欺诈|传销)");
sm_regex!(
    RE_SCI_DISC,
    r"(?:发现|找到).*(?:新|未知)|(?:规律|原理|定理).*(?:发现|揭示)"
);
sm_regex!(
    RE_ANIMAL_PLANT,
    r"(?:动物|植物).*(?:品种|变种)|(?:育种|培育).*(?:方法|技术)"
);
sm_regex!(RE_NUCLEAR, r"(?:原子核|核能).*(?:变换|裂变|聚变)");
sm_regex!(
    RE_COMPUTER,
    r"(?:仅仅|仅|单纯).*(?:计算机程序|软件|代码)|(?:存储介质|载体).*(?:计算机程序)"
);
sm_regex!(
    RE_TF_QUOTED,
    r#"["'「」『』]([^"'「」『』]{2,})["'「」『』]"#
);
sm_regex!(
    RE_TF_COMPONENT,
    r"[一-龥a-zA-Z0-9]{2,6}(?:装置|设备|机构|系统|单元|模块|组件|器件|器|电路|网络|总线|接口)"
);
sm_regex!(
    RE_MEANS_COMPONENT,
    r"装置|设备|机构|系统|单元|模块|组件|器件|电路|网络"
);
sm_regex!(RE_MEANS_ACTION, r"采用|利用|使用|设置|配置|安装|连接");
sm_regex!(RE_MEANS_CHANNEL, r"通过|经由|基于");
sm_regex!(RE_PROBLEM, r"问题|不足|缺点|缺陷|困难|需要|缺乏|缺少");
sm_regex!(
    RE_EFFECT,
    r"提高|改善|增强|优化|减少|降低|节省|效率|精度|性能|质量|稳定性"
);

fn sm_load_regex(name: &str, fallback: &str) -> Regex {
    match name {
        "ia_1" => RE_IA_1.clone(),
        "ia_2" => RE_IA_2.clone(),
        "ia_3" => RE_IA_3.clone(),
        "ia_4" => RE_IA_4.clone(),
        "ia_5" => RE_IA_5.clone(),
        "ia_all" => RE_IA_ALL.clone(),
        "md_1" => RE_MD_1.clone(),
        "md_2" => RE_MD_2.clone(),
        "md_3" => RE_MD_3.clone(),
        "md_4" => RE_MD_4.clone(),
        "leg" | "leg_gambling" => RE_LEG_GAMBLING.clone(),
        "leg_drugs" => RE_LEG_DRUGS.clone(),
        "leg_fraud" => RE_LEG_FRAUD.clone(),
        "sci_disc" => RE_SCI_DISC.clone(),
        "animal_plant" => RE_ANIMAL_PLANT.clone(),
        "nuclear" => RE_NUCLEAR.clone(),
        "computer" => RE_COMPUTER.clone(),
        "technical_feature_quoted" => RE_TF_QUOTED.clone(),
        "technical_feature_component" => RE_TF_COMPONENT.clone(),
        "technical_means_component" => RE_MEANS_COMPONENT.clone(),
        "technical_means_action" => RE_MEANS_ACTION.clone(),
        "technical_means_channel" => RE_MEANS_CHANNEL.clone(),
        "technical_problem" => RE_PROBLEM.clone(),
        "technical_effect" => RE_EFFECT.clone(),
        _ => Regex::new(fallback).unwrap(),
    }
}

fn analyze_technical_solutions(input: &SubjectMatterInput, output: &mut SmOutput) {
    let independent_claims: Vec<&SmClaimInput> = input
        .claims
        .iter()
        .filter(|c| c.claim_type == "independent")
        .collect();

    let mut has_tech = false;

    for claim in &independent_claims {
        let features = sm_extract_technical_features(&claim.content);
        let has_means = sm_has_technical_means(&claim.content);
        let problem = sm_infer_technical_problem(input);
        let effect = sm_infer_technical_effect(input);

        let is_ts = has_means && problem.is_some() && effect.is_some() && !features.is_empty();

        if is_ts {
            has_tech = true;
        } else {
            output
                .article2_invention_definition
                .issues
                .push(SmIssueItem {
                    claim_number: claim.number,
                    issue: "权利要求不构成技术方案".into(),
                    suggestion: "建议修改权利要求，使其包含技术特征、解决技术问题并产生技术效果"
                        .into(),
                });
        }

        output
            .technical_solution_analysis
            .independent_claims_analysis
            .push(SmClaimAnalysis {
                is_technical_solution: is_ts,
                technical_features: features,
                technical_problem: problem,
                technical_effect: effect,
            });
    }

    output.technical_solution_analysis.has_technical_solution = has_tech;
    output.article2_invention_definition.is_technical_solution = has_tech;
    output.article2_invention_definition.passed = has_tech;
}

struct ExclusionPattern {
    regex: Regex,
    type_value: &'static str,
    type_name: &'static str,
    reason: &'static str,
}

fn check_exclusion_patterns(
    input: &SubjectMatterInput,
    patterns: &[ExclusionPattern],
    output: &mut SmOutput,
) {
    for claim in &input.claims {
        for p in patterns {
            if p.regex.is_match(&claim.content) {
                output
                    .article25_exclusions
                    .non_protectable_matters
                    .push(SmNonProtectableMatter {
                        r#type: p.type_value.into(),
                        type_name: p.type_name.into(),
                        reason: p.reason.into(),
                        related_claims: vec![claim.number],
                    });
            }
        }
    }
}

fn check_intellectual_activity_rules(input: &SubjectMatterInput, output: &mut SmOutput) {
    let labels = [
        "智力活动方法",
        "管理规则",
        "游戏规则",
        "商业模式",
        "管理流程",
    ];
    let patterns: Vec<Regex> = [
        r"(?:方法|步骤).*(?:计算|运算|统计|分析|推理|判断|决策)",
        r"(?:规则|方法|模式|算法|模型).*(?:管理|控制|优化)",
        r"(?:游戏|竞赛|比赛).*(?:规则|方法)",
        r"(?:商业|经营|营销).*(?:模式|方法|策略)",
        r"(?:流程|程序).*(?:管理|审批|审核)",
    ]
    .iter()
    .map(|fb| sm_load_regex("ia", fb))
    .collect();

    for claim in &input.claims {
        for (i, re) in patterns.iter().enumerate() {
            if re.is_match(&claim.content) {
                output
                    .intellectual_activity_check
                    .has_intellectual_activity_rules = true;
                output
                    .intellectual_activity_check
                    .detected_rules
                    .push(SmDetectedRule {
                        claim_number: claim.number,
                        rule_type: labels[i].to_string(),
                        description: format!("检测到{}相关表述", labels[i]),
                    });
            }
        }
    }

    check_exclusion_patterns(
        input,
        &[ExclusionPattern {
            regex: sm_load_regex(
                "ia_all",
                r"(?x)
                    (?:方法|步骤).*(?:计算|运算|统计|分析|推理|判断|决策)
                    | (?:规则|方法|模式|算法|模型).*(?:管理|控制|优化)
                    | (?:游戏|竞赛|比赛).*(?:规则|方法)
                    | (?:商业|经营|营销).*(?:模式|方法|策略)
                    | (?:流程|程序).*(?:管理|审批|审核)",
            ),
            type_value: "intellectual_activity_rules",
            type_name: "智力活动的规则和方法",
            reason: "仅涉及智力活动的规则和方法，未采用技术手段",
        }],
        output,
    );
}

fn check_medical_diagnosis(input: &SubjectMatterInput, output: &mut SmOutput) {
    check_exclusion_patterns(
        input,
        &[
            ExclusionPattern {
                regex: sm_load_regex("md_1", r"(?:疾病|病症|病情).*(?:诊断|检查|筛查|检测)"),
                type_value: "medical_diagnosis_treatment",
                type_name: "疾病的诊断和治疗方法",
                reason: "涉及疾病的诊断和治疗方法",
            },
            ExclusionPattern {
                regex: sm_load_regex("md_2", r"(?:手术|治疗|疗法|医治).*(?:方法|方案)"),
                type_value: "medical_diagnosis_treatment",
                type_name: "疾病的诊断和治疗方法",
                reason: "涉及疾病的诊断和治疗方法",
            },
            ExclusionPattern {
                regex: sm_load_regex("md_3", r"(?:药物|药品).*(?:配方|组合物).*(?:用于|治疗)"),
                type_value: "medical_diagnosis_treatment",
                type_name: "疾病的诊断和治疗方法",
                reason: "涉及疾病的诊断和治疗方法",
            },
            ExclusionPattern {
                regex: sm_load_regex("md_4", r"(?:针灸|按摩|理疗).*(?:方法)"),
                type_value: "medical_diagnosis_treatment",
                type_name: "疾病的诊断和治疗方法",
                reason: "涉及疾病的诊断和治疗方法",
            },
        ],
        output,
    );
}

fn check_legality(input: &SubjectMatterInput, output: &mut SmOutput) {
    let rules = &[
        ("赌博", r"(?:赌博|博彩|赌场)"),
        ("违禁品", r"(?:毒品|违禁品|非法药物)"),
        ("违法犯罪", r"(?:诈骗|欺诈|传销)"),
    ];
    let compiled: Vec<Regex> = rules
        .iter()
        .map(|(_, fb)| sm_load_regex("leg", fb))
        .collect();

    for claim in &input.claims {
        for (i, re) in compiled.iter().enumerate() {
            if re.is_match(&claim.content) {
                output.legality_check.passed = false;
                let reason = format!("涉及{}", rules[i].0);
                output
                    .legality_check
                    .illegal_content
                    .push(SmIllegalContent {
                        claim_number: claim.number,
                        content: claim.content.clone(),
                        reason: reason.clone(),
                    });
            }
        }
    }

    check_exclusion_patterns(
        input,
        &[
            ExclusionPattern {
                regex: sm_load_regex("leg_gambling", r"(?:赌博|博彩|赌场)"),
                type_value: "illegal_content",
                type_name: "违反法律、社会公德",
                reason: "涉及赌博",
            },
            ExclusionPattern {
                regex: sm_load_regex("leg_drugs", r"(?:毒品|违禁品|非法药物)"),
                type_value: "illegal_content",
                type_name: "违反法律、社会公德",
                reason: "涉及违禁品",
            },
            ExclusionPattern {
                regex: sm_load_regex("leg_fraud", r"(?:诈骗|欺诈|传销)"),
                type_value: "illegal_content",
                type_name: "违反法律、社会公德",
                reason: "涉及违法犯罪",
            },
        ],
        output,
    );
}

fn check_other_exclusions(input: &SubjectMatterInput, output: &mut SmOutput) {
    check_exclusion_patterns(
        input,
        &[
            ExclusionPattern {
                regex: sm_load_regex(
                    "sci_disc",
                    r"(?:发现|找到).*(?:新|未知)|(?:规律|原理|定理).*(?:发现|揭示)",
                ),
                type_value: "scientific_discovery",
                type_name: "科学发现",
                reason: "属于科学发现，不是技术方案",
            },
            ExclusionPattern {
                regex: sm_load_regex(
                    "animal_plant",
                    r"(?:动物|植物).*(?:品种|变种)|(?:育种|培育).*(?:方法|技术)",
                ),
                type_value: "animal_plant_variety",
                type_name: "动物和植物品种",
                reason: "涉及动物和植物品种",
            },
            ExclusionPattern {
                regex: sm_load_regex("nuclear", r"(?:原子核|核能).*(?:变换|裂变|聚变)"),
                type_value: "nuclear_transformation",
                type_name: "原子核变换方法",
                reason: "涉及原子核变换方法",
            },
            ExclusionPattern {
                regex: sm_load_regex(
                    "computer",
                    r"(?:仅仅|仅|单纯).*(?:计算机程序|软件|代码)|(?:存储介质|载体).*(?:计算机程序)",
                ),
                type_value: "computer_program_only",
                type_name: "单纯的计算机程序",
                reason: "单纯的计算机程序不是专利保护客体",
            },
        ],
        output,
    );
}

fn generate_sm_overall_report(output: &mut SmOutput) {
    let total = output.article2_invention_definition.issues.len()
        + output.article25_exclusions.non_protectable_matters.len()
        + usize::from(!output.legality_check.illegal_content.is_empty());

    let critical = output
        .article25_exclusions
        .non_protectable_matters
        .iter()
        .filter(|m| m.r#type == "illegal_content" || m.r#type == "computer_program_only")
        .count();

    let is_protectable =
        output.article2_invention_definition.passed && output.article25_exclusions.passed;

    output.overall_report.total_issues = total;
    output.overall_report.critical_issues = critical;
    output.overall_report.is_protectable_subject_matter = is_protectable;
    output.overall_report.passed = is_protectable && critical == 0;

    let mut recs = Vec::new();
    if !output.article2_invention_definition.passed {
        recs.push("建议修改权利要求，使其包含技术手段、解决技术问题并产生技术效果".into());
    }
    if output
        .intellectual_activity_check
        .has_intellectual_activity_rules
    {
        recs.push("建议在权利要求中增加技术手段，避免仅涉及智力活动规则".into());
    }
    if output
        .article25_exclusions
        .non_protectable_matters
        .iter()
        .any(|m| m.r#type == "computer_program_only")
    {
        recs.push("建议修改权利要求，使其包含硬件技术特征，或采用\"方法+装置\"的撰写方式".into());
    }
    if output
        .article25_exclusions
        .non_protectable_matters
        .iter()
        .any(|m| m.r#type == "medical_diagnosis_treatment")
    {
        recs.push("疾病的诊断和治疗方法不能授予专利，但诊断设备和治疗器械可以申请专利".into());
    }
    if !output.legality_check.passed {
        recs.push("专利申请不得违反法律、社会公德或者妨害公共利益".into());
    }
    if is_protectable {
        recs.push("该申请属于专利保护客体".into());
    }
    output.overall_report.recommendations = recs;
}

fn sm_extract_technical_features(content: &str) -> Vec<String> {
    let mut features = Vec::new();

    let quote_re = sm_load_regex(
        "technical_feature_quoted",
        r#"["'「」『』]([^"'「」『』]{2,})["'「」『』]"#,
    );
    for cap in quote_re.captures_iter(content) {
        let f = cap[1].to_string();
        if !features.contains(&f) {
            features.push(f);
        }
    }

    let comp_re = sm_load_regex(
        "technical_feature_component",
        r"[一-龥a-zA-Z0-9]{2,6}(?:装置|设备|机构|系统|单元|模块|组件|器件|器|电路|网络|总线|接口)",
    );
    for cap in comp_re.captures_iter(content) {
        let f = cap[0].to_string();
        if !features.contains(&f) {
            features.push(f);
        }
    }

    features
}

fn sm_has_technical_means(content: &str) -> bool {
    let patterns = [
        sm_load_regex(
            "technical_means_component",
            r"装置|设备|机构|系统|单元|模块|组件|器件|电路|网络",
        ),
        sm_load_regex(
            "technical_means_action",
            r"采用|利用|使用|设置|配置|安装|连接",
        ),
        sm_load_regex("technical_means_channel", r"通过|经由|基于"),
    ];
    patterns.iter().any(|re| re.is_match(content))
}

fn sm_infer_technical_problem(input: &SubjectMatterInput) -> Option<String> {
    let spec = input.specification.as_ref()?;
    let bg = spec.background_art.as_ref()?;
    let re = sm_load_regex(
        "technical_problem",
        r"问题|不足|缺点|缺陷|困难|需要|缺乏|缺少",
    );
    if re.is_match(bg) {
        Some("存在技术问题".into())
    } else {
        None
    }
}

fn sm_infer_technical_effect(input: &SubjectMatterInput) -> Option<String> {
    let spec = input.specification.as_ref()?;
    let content = spec.invention_content.as_ref()?;
    let re = sm_load_regex(
        "technical_effect",
        r"提高|改善|增强|优化|减少|降低|节省|效率|精度|性能|质量|稳定性",
    );
    if re.is_match(content) {
        Some("产生技术效果".into())
    } else {
        None
    }
}

/// 执行保护客体检查。
pub fn execute_subject_matter_check(input: &SubjectMatterInput) -> Result<Value, String> {
    let mut output = SmOutput {
        article2_invention_definition: SmArticle2Check {
            passed: true,
            is_technical_solution: false,
            issues: Vec::new(),
        },
        article25_exclusions: SmArticle25Check {
            passed: true,
            non_protectable_matters: Vec::new(),
        },
        technical_solution_analysis: SmTechSolutionAnalysis {
            independent_claims_analysis: Vec::new(),
            has_technical_solution: false,
        },
        intellectual_activity_check: SmIntellectualActivityCheck {
            has_intellectual_activity_rules: false,
            detected_rules: Vec::new(),
        },
        legality_check: SmLegalityCheck {
            passed: true,
            illegal_content: Vec::new(),
        },
        overall_report: SmOverallReport {
            passed: false,
            total_issues: 0,
            critical_issues: 0,
            is_protectable_subject_matter: false,
            recommendations: Vec::new(),
        },
    };

    analyze_technical_solutions(input, &mut output);
    check_intellectual_activity_rules(input, &mut output);
    check_medical_diagnosis(input, &mut output);
    check_legality(input, &mut output);
    check_other_exclusions(input, &mut output);

    output.article25_exclusions.passed = output
        .article25_exclusions
        .non_protectable_matters
        .is_empty();

    generate_sm_overall_report(&mut output);

    serde_json::to_value(output).map_err(|e| e.to_string())
}
