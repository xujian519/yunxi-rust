// analysis.rs - 侵权分析、协同测试、法律问答

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::compare::{extract_keywords, EQUIVALENT_THRESHOLD, MIN_OVERLAP_RATIO, STOPWORDS};

// ============================================================================
// 工具2: InfringementAnalysis - 专利侵权分析
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfringementAnalysisInput {
    pub patent_claims: Vec<String>,
    pub accused_product: String,
    #[serde(default)]
    pub analysis_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ClaimResult {
    claim_number: usize,
    elements: Vec<String>,
    literal_infringement: bool,
    equivalent_infringement: bool,
    missing_elements: Vec<String>,
    risk_level: String,
}

#[allow(clippy::unnecessary_wraps)]
pub fn infringement_analysis(input: InfringementAnalysisInput) -> Result<Value, String> {
    let analysis_type = input.analysis_type.unwrap_or_else(|| "full".to_string());
    let claims = input.patent_claims;
    let product = &input.accused_product;

    let mut claim_results = Vec::new();
    let mut high_risk_count = 0;

    let product_keywords = extract_keywords(product);

    for (i, claim) in claims.iter().enumerate() {
        let elements = extract_claim_elements(claim);

        let mut literal_infringement = true;
        let mut equivalent_infringement = true;
        let mut missing_elements = Vec::new();

        for element in &elements {
            let element_keywords = extract_keywords(element);

            // 计算关键词重叠度
            let overlap = calculate_keyword_overlap(&element_keywords, &product_keywords);

            if overlap == 0.0 {
                literal_infringement = false;
                missing_elements.push(element.clone());
            }

            if overlap < EQUIVALENT_THRESHOLD {
                equivalent_infringement = false;
            }
        }

        let risk_level = if literal_infringement {
            high_risk_count += 1;
            "high"
        } else if equivalent_infringement {
            "medium"
        } else {
            "low"
        };

        claim_results.push(ClaimResult {
            claim_number: i + 1,
            elements,
            literal_infringement,
            equivalent_infringement,
            missing_elements,
            risk_level: risk_level.to_string(),
        });
    }

    let overall_risk = if high_risk_count > 0 {
        "high"
    } else if claim_results.iter().any(|r| r.risk_level == "medium") {
        "medium"
    } else {
        "low"
    };

    let recommendations = generate_infringement_recommendations(&claim_results, overall_risk);

    Ok(json!({
        "claims_analyzed": claims.len(),
        "analysis_type": analysis_type,
        "claim_results": claim_results,
        "overall_risk": overall_risk,
        "recommendations": recommendations,
        "legal_basis": "专利法第59条 - 发明或者实用新型专利权的保护范围以其权利要求的内容为准"
    }))
}

fn extract_claim_elements(claim: &str) -> Vec<String> {
    // 按照常见分隔符拆分权利要求
    let separators = ["；", ";", "，", ",", "。", ".", "其中", "所述"];
    let mut result = String::from(claim);

    for sep in &separators {
        result = result.replace(sep, "|");
    }

    result
        .split('|')
        .map(|s| s.trim().to_string())
        .filter(|s| s.len() > 2)
        .collect()
}

fn calculate_keyword_overlap(elem_keywords: &[String], product_keywords: &[String]) -> f64 {
    if elem_keywords.is_empty() || product_keywords.is_empty() {
        return 0.0;
    }

    let elem_set: std::collections::HashSet<_> = elem_keywords.iter().collect();
    let prod_set: std::collections::HashSet<_> = product_keywords.iter().collect();

    let intersection: std::collections::HashSet<_> =
        elem_set.intersection(&prod_set).copied().collect();

    #[allow(clippy::cast_precision_loss)]
    {
        intersection.len() as f64 / elem_set.len() as f64
    }
}

fn generate_infringement_recommendations(
    results: &[ClaimResult],
    overall_risk: &str,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    match overall_risk {
        "high" => {
            recommendations.push("高风险：建议进行FTO (Freedom to Operate) 分析".to_string());
            recommendations.push("考虑设计绕过方案，避免字面侵权".to_string());
            recommendations.push("评估专利有效性，考虑提起无效宣告".to_string());
        }
        "medium" => {
            recommendations.push("中风险：建议详细分析等同侵权可能性".to_string());
            recommendations.push("准备等同侵权抗辩策略".to_string());
        }
        _ => {
            recommendations.push("低风险：但仍需持续关注相关专利动态".to_string());
        }
    }

    // 针对具体权利要求的建议
    for result in results {
        if result.risk_level == "high" {
            recommendations.push(format!(
                "权利要求{}存在高风险，重点分析: {}",
                result.claim_number,
                result.elements.join(", ")
            ));
        }
    }

    recommendations
}

// ============================================================================
// 工具3: SynergyAnalysis - 技术特征协同测试
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechUnit {
    pub id: String,
    pub name: String,
    pub source_text: String,
    #[serde(default)]
    pub technical_function: Option<String>,
    #[serde(default)]
    pub technical_effect: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SynergyAnalysisInput {
    pub units: Vec<TechUnit>,
    #[serde(default)]
    pub apply_merge: Option<bool>,
}

pub fn synergy_analysis(input: SynergyAnalysisInput) -> Result<Value, String> {
    let apply_merge = input.apply_merge.unwrap_or(false);
    let units = input.units;

    if units.len() < 2 {
        return Err("至少需要2个技术单元才能进行协同分析".to_string());
    }

    let mut synergistic_pairs = Vec::new();
    let mut independent_units = Vec::new();
    let mut merged_units = Vec::new();

    // 标记是否已被合并
    let mut merged_indices = std::collections::HashSet::new();

    for i in 0..units.len() {
        if merged_indices.contains(&i) {
            continue;
        }

        let mut merged_with = vec![i];

        for j in (i + 1)..units.len() {
            if merged_indices.contains(&j) {
                continue;
            }

            let unit_a = &units[i];
            let unit_b = &units[j];

            // 三条件测试
            let same_problem = test_same_problem(unit_a, unit_b);
            let synergistic_effect = test_synergistic_effect(unit_a, unit_b);
            let mutual_dependence = test_mutual_dependence(unit_a, unit_b);

            let is_synergistic = same_problem && synergistic_effect && mutual_dependence;

            if is_synergistic {
                let merged_name = format!("{} + {}", unit_a.name, unit_b.name);

                synergistic_pairs.push(json!({
                    "unit_a": unit_a.id,
                    "unit_b": unit_b.id,
                    "same_problem": same_problem,
                    "synergistic_effect": synergistic_effect,
                    "mutual_dependence": mutual_dependence,
                    "merged_name": merged_name
                }));

                if apply_merge {
                    merged_with.push(j);
                    merged_indices.insert(j);

                    merged_units.push(json!({
                        "id": format!("merged_{}_{}", unit_a.id, unit_b.id),
                        "name": merged_name,
                        "constituent_units": [unit_a.id, unit_b.id],
                        "source_text": format!("{} {}", unit_a.source_text, unit_b.source_text),
                        "technical_function": format!("{}; {}",
                            unit_a.technical_function.as_deref().unwrap_or(""),
                            unit_b.technical_function.as_deref().unwrap_or("")
                        ),
                        "technical_effect": format!("{}; {}",
                            unit_a.technical_effect.as_deref().unwrap_or(""),
                            unit_b.technical_effect.as_deref().unwrap_or("")
                        )
                    }));
                }
            }
        }

        if merged_with.len() == 1 && !merged_indices.contains(&i) {
            independent_units.push(&units[i]);
        }

        if apply_merge && !merged_with.is_empty() {
            merged_indices.insert(i);
        }
    }

    Ok(json!({
        "total_units": units.len(),
        "pairs_tested": units.len() * (units.len() - 1) / 2,
        "synergistic_pairs": synergistic_pairs,
        "merged_units": if apply_merge { Some(merged_units) } else { None },
        "independent_units": independent_units.iter().map(|u| json!({
            "id": u.id,
            "name": u.name
        })).collect::<Vec<_>>()
    }))
}

fn test_same_problem(unit_a: &TechUnit, unit_b: &TechUnit) -> bool {
    let keywords_a: Vec<_> = extract_keywords(&unit_a.source_text)
        .into_iter()
        .filter(|k| k.len() >= 2 && !STOPWORDS.contains(&k.as_str()))
        .collect();

    let keywords_b: Vec<_> = extract_keywords(&unit_b.source_text)
        .into_iter()
        .filter(|k| k.len() >= 2 && !STOPWORDS.contains(&k.as_str()))
        .collect();

    // 计算共同关键词数量
    let set_a: std::collections::HashSet<_> = keywords_a.iter().collect();
    let set_b: std::collections::HashSet<_> = keywords_b.iter().collect();

    let common_count = set_a.intersection(&set_b).count();

    #[allow(clippy::cast_precision_loss)]
    let overlap_ratio = if !set_a.is_empty() && !set_b.is_empty() {
        (common_count as f64) / (set_a.len().min(set_b.len()) as f64)
    } else {
        0.0
    };
    overlap_ratio >= MIN_OVERLAP_RATIO
}

fn test_synergistic_effect(unit_a: &TechUnit, unit_b: &TechUnit) -> bool {
    let effect_a = unit_a.technical_effect.as_deref().unwrap_or("");
    let effect_b = unit_b.technical_effect.as_deref().unwrap_or("");

    if effect_a.is_empty() || effect_b.is_empty() {
        return false;
    }

    // 检查技术效果的关键词重叠
    let keywords_a = extract_keywords(effect_a);
    let keywords_b = extract_keywords(effect_b);

    let set_a: std::collections::HashSet<_> = keywords_a.iter().collect();
    let set_b: std::collections::HashSet<_> = keywords_b.iter().collect();

    let common_count = set_a.intersection(&set_b).count();

    common_count >= 1
}

fn test_mutual_dependence(unit_a: &TechUnit, unit_b: &TechUnit) -> bool {
    let text_a = &unit_a.source_text;
    let text_b = &unit_b.source_text;

    let a_refs_b = text_a.contains(&format!("所述{}", unit_b.name))
        || text_a.contains(&format!("该{}", unit_b.name));

    let b_refs_a = text_b.contains(&format!("所述{}", unit_a.name))
        || text_b.contains(&format!("该{}", unit_a.name));

    a_refs_b || b_refs_a
}

// ============================================================================
// 工具4: LegalQA - 知识产权法律问答
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalQAInput {
    pub question: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub context: Option<String>, // 保留原因: 预留给复杂查询的上下文支持
}

#[derive(Debug, Clone)]
struct LawArticle {
    article: &'static str,
    content: &'static str,
    keywords: &'static [&'static str],
    domain: &'static str,
}

// 法律知识库
const LAW_DATABASE: &[LawArticle] = &[
    // 专利法
    LawArticle {
        article: "专利法第2条",
        content: "本法所称的发明创造是指发明、实用新型和外观设计。发明，是指对产品、方法或者其改进所提出的新的技术方案。实用新型，是指对产品的形状、构造或者其结合所提出的适于实用的新的技术方案。外观设计，是指对产品的整体或者局部的形状、图案或者其结合以及色彩与形状、图案的结合所作出的富有美感并适于工业应用的新设计。",
        keywords: &["发明", "实用新型", "外观设计", "定义", "技术方案"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第22条第1款",
        content: "授予专利权的发明和实用新型，应当具备新颖性、创造性和实用性。",
        keywords: &["新颖性", "创造性", "实用性", "授权条件"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第22条第2款",
        content: "新颖性，是指该发明或者实用新型不属于现有技术；也没有任何单位或者个人就同样的发明或者实用新型在申请日以前向国务院专利行政部门提出过申请，并记载在申请日以后公布的专利申请文件或者公告的专利文件中。",
        keywords: &["新颖性", "现有技术", "抵触申请"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第22条第3款",
        content: "创造性，是指与现有技术相比，该发明具有突出的实质性特点和显著的进步，该实用新型具有实质性特点和进步。",
        keywords: &["创造性", "实质性特点", "进步", "显而易见"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第23条",
        content: "授予专利权的外观设计，应当不属于现有设计；也没有任何单位或者个人就同样的外观设计在申请日以前向国务院专利行政部门提出过申请，并记载在申请日以后公告的专利文件中。授予专利权的外观设计与现有设计或者现有设计特征的组合相比，应当具有明显区别。",
        keywords: &["外观设计", "新颖性", "明显区别"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第26条第3款",
        content: "说明书应当对发明或者实用新型作出清楚、完整的说明，以所属技术领域的技术人员能够实现为准。",
        keywords: &["说明书", "充分公开", "实现", "清楚完整"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第26条第4款",
        content: "权利要求书应当以说明书为依据，清楚、简要地限定要求专利保护的范围。",
        keywords: &["权利要求书", "支持", "保护范围", "依据"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第33条",
        content: "申请人可以对其专利申请文件进行修改，但是，对发明和实用新型专利申请文件的修改不得超出原说明书和权利要求书记载的范围，对外观设计专利申请文件的修改不得超出原图片或者照片表示的范围。",
        keywords: &["修改", "超范围", "原范围"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第59条第1款",
        content: "发明或者实用新型专利权的保护范围以其权利要求的内容为准，说明书及附图可以用于解释权利要求的内容。",
        keywords: &["保护范围", "权利要求", "解释", "侵权判定"],
        domain: "patent",
    },
    LawArticle {
        article: "专利法第62条",
        content: "在专利侵权纠纷中，被控侵权人有证据证明其实施的技术或者设计属于现有技术或者现有设计的，不构成侵犯专利权。",
        keywords: &["现有技术抗辩", "侵权", "不侵权"],
        domain: "patent",
    },
    // 商标法
    LawArticle {
        article: "商标法第8条",
        content: "任何能够将自然人、法人或者其他组织的商品与他人的商品区别开的可视性标志，包括文字、图形、字母、数字、三维标志、颜色组合和声音等，以及上述要素的组合，均可以作为商标申请注册。",
        keywords: &["商标", "标志", "注册", "要素"],
        domain: "trademark",
    },
    LawArticle {
        article: "商标法第9条",
        content: "申请注册的商标，应当有显著特征，便于识别，并不得与他人在先取得的合法权利相冲突。",
        keywords: &["显著性", "识别", "在先权利", "冲突"],
        domain: "trademark",
    },
    LawArticle {
        article: "商标法第30条",
        content: "申请注册的商标，凡不符合本法有关规定或者同他人在同一种商品或者类似商品上已经注册的或者初步审定的商标相同或者近似的，由商标局驳回申请，不予公告。",
        keywords: &["相同", "近似", "驳回", "类似商品"],
        domain: "trademark",
    },
    LawArticle {
        article: "商标法第31条",
        content: "两个或者两个以上的商标注册申请人，在同一种商品或者类似商品上，以相同或者近似的商标申请注册的，初步审定并公告申请在先的商标；同一天申请的，初步审定并公告使用在先的商标，驳回其他人的申请，不予公告。",
        keywords: &["申请在先", "使用在先", "驳回"],
        domain: "trademark",
    },
    LawArticle {
        article: "商标法第32条",
        content: "申请商标注册不得损害他人现有的在先权利，也不得以不正当手段抢先注册他人已经使用并有一定影响的商标。",
        keywords: &["在先权利", "抢注", "恶意注册", "有一定影响"],
        domain: "trademark",
    },
    LawArticle {
        article: "商标法第57条",
        content: "有下列行为之一的，均属侵犯注册商标专用权：（一）未经商标注册人许可，在同一种商品上使用与其注册商标相同的商标的；（二）未经商标注册人许可，在类似商品上使用与其注册商标相同或者近似的商标，容易导致混淆的；（三）销售侵犯注册商标专用权商品的；（四）伪造、擅自制造他人注册商标标识或者销售伪造、擅自制造的注册商标标识的；（五）未经商标注册人同意，更换其注册商标并将该更换商标的商品又投入市场的；（六）故意为侵犯他人商标专用权行为提供仓储、运输、邮寄、印制、隐匿、经营场所、网络商品交易平台等便利条件的；（七）给他人的注册商标专用权造成其他损害的。",
        keywords: &["侵权", "商标侵权", "专用权", "相同", "近似"],
        domain: "trademark",
    },
    // 著作权法
    LawArticle {
        article: "著作权法第3条",
        content: "本法所称的作品，是指文学、艺术和科学领域内具有独创性并能以一定形式表现的智力成果，包括：（一）文字作品；（二）口述作品；（三）音乐、戏剧、曲艺、舞蹈、杂技艺术作品；（四）美术、建筑作品；（五）摄影作品；（六）电影作品和以类似摄制电影的方法创作的作品；（七）工程设计图、产品设计图、地图、示意图等图形作品和模型作品；（八）计算机软件；（九）法律、行政法规规定的其他作品。",
        keywords: &["作品", "独创性", "著作权", "保护客体"],
        domain: "copyright",
    },
    LawArticle {
        article: "著作权法第10条",
        content: "著作权包括下列人身权和财产权：（一）发表权；（二）署名权；（三）修改权；（四）保护作品完整权；（五）复制权；（六）发行权；（七）出租权；（八）展览权；（九）表演权；（十）放映权；（十一）广播权；（十二）信息网络传播权；（十三）摄制权；（十四）改编权；（十五）翻译权；（十六）汇编权；（十七）应当由著作权人享有的其他权利。",
        keywords: &["人身权", "财产权", "复制权", "发行权", "信息网络传播权"],
        domain: "copyright",
    },
    LawArticle {
        article: "著作权法第24条",
        content: "使用他人作品应当同著作权人订立许可使用合同，本法规定可以不经许可的除外。许可使用合同包括下列主要内容：（一）许可使用的权利种类；（二）许可使用的权利是专有使用权或者非专有使用权；（三）许可使用的地域范围、期间；（四）付酬标准和办法；（五）违约责任；（六）双方认为需要约定的其他内容。",
        keywords: &["许可", "许可合同", "专有使用权", "付酬"],
        domain: "copyright",
    },
    LawArticle {
        article: "著作权法第52条",
        content: "有下列侵权行为的，应当根据情况，承担停止侵害、消除影响、赔礼道歉、赔偿损失等民事责任：（一）未经著作权人许可，发表其作品的；（二）未经合作作者许可，将与他人合作创作的作品当作自己单独创作的作品发表的；（三）没有参加创作，为谋取个人名利，在他人作品上署名的；（四）歪曲、篡改他人作品的；（五）剽窃他人作品的；（六）未经著作权人许可，以展览、摄制电影和以类似摄制电影的方法使用作品，或者以改编、翻译、注释等方式使用作品的，本法另有规定的除外；（七）使用他人作品，应当支付报酬而未支付的；（八）未经电影作品和以类似摄制电影的方法创作的作品、计算机软件、录音录像制品的著作权人或者与著作权有关的权利人许可，出租其作品或者录音录像制品的，本法另有规定的除外；（九）未经出版者许可，使用其出版的图书、期刊的版式设计的；（十）未经表演者许可，现场直播或者公开传送其现场表演，或者录制其表演的；（十一）其他侵犯著作权以及与著作权有关的权益的行为。",
        keywords: &["侵权", "剽窃", "署名", "赔偿", "民事责任"],
        domain: "copyright",
    },
];

#[allow(clippy::unnecessary_wraps)]
pub fn legal_qa(input: LegalQAInput) -> Result<Value, String> {
    let domain = input.domain.unwrap_or_else(|| "all".to_string());
    let question = &input.question;

    // 提取问题关键词
    let question_keywords = extract_keywords(question);

    // 匹配相关法律条文
    let mut applicable_rules = Vec::new();

    for law in LAW_DATABASE {
        // 检查域过滤
        if domain != "all" && law.domain != domain {
            continue;
        }

        // 计算关键词匹配度
        let mut match_count = 0;
        let mut matched_keywords: Vec<String> = Vec::new();

        for kw in law.keywords {
            if question.contains(kw) {
                match_count += 1;
                matched_keywords.push((*kw).to_string());
            }
        }

        // 检查问题关键词是否在法律条文中出现
        for qkw in &question_keywords {
            if law.content.contains(qkw) {
                match_count += 1;
                matched_keywords.push(qkw.clone());
            }
        }

        if match_count > 0 {
            #[allow(clippy::cast_precision_loss)]
            let relevance = f64::from(match_count) / (law.keywords.len() as f64).max(1.0);

            applicable_rules.push(json!({
                "article": law.article,
                "content": law.content,
                "domain": law.domain,
                "relevance": relevance,
                "matched_keywords": matched_keywords
            }));
        }
    }

    // 按相关性排序
    applicable_rules.sort_by(|a, b| {
        let r_a = b["relevance"].as_f64().unwrap_or(0.0);
        let r_b = a["relevance"].as_f64().unwrap_or(0.0);
        r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // 提取前5条最相关的
    let top_rules: Vec<_> = applicable_rules.iter().take(5).cloned().collect();

    // 生成答案摘要
    let summary = generate_qa_summary(question, &top_rules);

    // 提取法律依据
    let legal_basis: Vec<_> = top_rules
        .iter()
        .filter_map(|r| r["article"].as_str())
        .collect();

    // 生成建议
    let recommendations = generate_qa_recommendations(question, &top_rules);

    // 计算置信度
    let confidence = if top_rules.is_empty() {
        0.0
    } else if top_rules[0]["relevance"].as_f64().unwrap_or(0.0) >= 0.5 {
        0.8
    } else {
        0.5
    };

    Ok(json!({
        "question": question,
        "domain": domain,
        "answer": {
            "summary": summary,
            "legal_basis": legal_basis,
            "applicable_rules": top_rules,
            "recommendations": recommendations
        },
        "confidence": confidence
    }))
}

fn generate_qa_summary(question: &str, rules: &[Value]) -> String {
    if rules.is_empty() {
        return "未找到直接相关的法律条文，建议提供更多背景信息或咨询专业知识产权律师。"
            .to_string();
    }

    let first_rule = &rules[0];
    let article = first_rule["article"].as_str().unwrap_or("");
    let relevance = first_rule["relevance"].as_f64().unwrap_or(0.0);

    if relevance >= 0.5 {
        format!(
            "根据{}，该问题涉及{}。",
            article,
            if question.contains("侵权") {
                "侵权判定"
            } else if question.contains("新颖性") {
                "新颖性判断"
            } else if question.contains("创造性") {
                "创造性判断"
            } else {
                "相关法律规定"
            }
        )
    } else {
        format!("可能与{article}等相关法律条款有关，建议进一步分析具体案情。")
    }
}

fn generate_qa_recommendations(question: &str, rules: &[Value]) -> Vec<String> {
    let mut recommendations = Vec::new();

    if rules.is_empty() {
        recommendations.push("建议咨询专业知识产权律师获取更准确的法律意见。".to_string());
        return recommendations;
    }

    // 基于问题类型的建议
    if question.contains("侵权") {
        recommendations.push("建议进行详细的侵权比对分析，确认是否落入保护范围。".to_string());
        recommendations.push("考虑进行FTO (Freedom to Operant) 分析评估风险。".to_string());
    } else if question.contains("新颖性") || question.contains("创造性") {
        recommendations.push("建议进行全面现有技术检索，分析对比文件。".to_string());
        recommendations.push("关注技术方案的实质性特点和进步。".to_string());
    } else if question.contains("申请") {
        recommendations.push("建议在申请前进行专利检索，避免重复授权。".to_string());
        recommendations.push("确保说明书充分公开，权利要求得到说明书支持。".to_string());
    }

    recommendations.push("本回答仅供参考，具体案件请咨询专业知识产权律师。".to_string());

    recommendations
}
