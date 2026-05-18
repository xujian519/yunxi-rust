use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write as _;

use super::{default_five, default_language, default_llm_call, default_one, default_patent_type};

// =============================================================================
// ClaimGenerator — 权利要求书生成器
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimGeneratorInput {
    /// 技术方案描述（详细的技术交底书内容）
    pub technical_solution: String,
    /// 专利类型: "invention" | "utilityModel"
    #[serde(default = "default_patent_type")]
    pub patent_type: String,
    /// 技术领域（可选）
    #[serde(default)]
    pub field: Option<String>,
    /// 现有权利要求（用于改写/扩展，可选）
    #[serde(default)]
    pub existing_claims: Option<Vec<String>>,
    /// 输出语言: "chinese" | "english"（默认 chinese）
    #[serde(default = "default_language")]
    pub language: String,
    /// 期望的独立权利要求数量（默认1）
    #[serde(default = "default_one")]
    pub independent_claim_count: u8,
    /// 每项独立权利要求对应的从属权利要求数量上限（默认5）
    #[serde(default = "default_five")]
    pub dependent_claim_max: u8,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClaimGeneratorOutput {
    independent_claims: Vec<ClaimOutput>,
    dependent_claims: Vec<ClaimOutput>,
    claim_count: usize,
    draft_notes: Vec<String>,
    language: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaimOutput {
    pub(crate) number: u32,
    pub(crate) r#type: String,
    pub(crate) content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) depends_on: Option<u32>,
}

pub(crate) fn build_claim_system_prompt(language: &str, patent_type: &str) -> String {
    let type_desc = match patent_type {
        "utilityModel" => "实用新型专利",
        _ => "发明专利",
    };
    let lang_instr = if language == "english" {
        "请用英文撰写权利要求书。"
    } else {
        "请用中文撰写权利要求书，使用规范的专利法术语。"
    };

    format!(
        "你是一名资深专利代理师，专精于{type_desc}的权利要求书撰写。\
        {lang_instr}\
        你需要根据提供的技术方案，撰写符合中国专利法及专利审查指南要求的权利要求书。\
        \n\
        撰写要求：\
        1. 独立权利要求应当从整体上反映发明的技术方案，记载解决技术问题的必要技术特征。\
        2. 从属权利要求应当用附加的技术特征，对引用的权利要求作进一步限定。\
        3. 权利要求书应当清楚、简要地表述请求保护的范围。\
        4. 避免使用功能性限定，除非无法用结构特征限定。\
        5. 权利要求之间的引用关系应当合理，避免多项引多项。\
        6. 保护范围应当适中，既不过宽（容易被无效），也不过窄（失去保护价值）。\
        \n\
        输出格式要求：\
        你必须严格按照以下格式输出，不要添加任何额外说明：\
        \n\
        ===独立权利要求===\
        1. [独立权利要求1的内容]\
        2. [独立权利要求2的内容（如有）]\
        \n\
        ===从属权利要求===\
        2. [从属权利要求1的内容，引用权利要求1]\
        3. [从属权利要求2的内容，引用权利要求1或2]\
        ...\
        \n\
        ===撰写建议===\
        - [建议1]\
        - [建议2]\
        ..."
    )
}

pub(crate) fn build_claim_user_prompt(input: &ClaimGeneratorInput) -> String {
    let mut prompt = format!("技术方案描述：\n{}\n", input.technical_solution);

    if let Some(ref field) = input.field {
        let _ = writeln!(prompt, "\n技术领域：{field}");
    }

    let _ = write!(prompt, "\n专利类型：{}\n", input.patent_type);

    if let Some(ref existing) = input.existing_claims {
        if !existing.is_empty() {
            prompt.push_str("\n现有权利要求（供参考/改写）：\n");
            for (i, claim) in existing.iter().enumerate() {
                let _ = writeln!(prompt, "{}. {}", i + 1, claim);
            }
        }
    }

    let _ = write!(
        prompt,
        "\n要求：\n- 独立权利要求数量：{}\n- 每项独立权利要求最多{}项从属权利要求\n",
        input.independent_claim_count, input.dependent_claim_max
    );

    prompt
}

static CLAIM_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^(\d+)[\.、]\s*(.+)$").unwrap());

pub(crate) fn parse_claims(text: &str) -> (Vec<ClaimOutput>, Vec<ClaimOutput>, Vec<String>) {
    let mut independents = Vec::new();
    let mut dependents = Vec::new();
    let mut notes = Vec::new();

    let mut in_independent = false;
    let mut in_dependent = false;
    let mut in_notes = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("===独立权利要求===") {
            in_independent = true;
            in_dependent = false;
            in_notes = false;
            continue;
        }
        if trimmed.starts_with("===从属权利要求===") {
            in_independent = false;
            in_dependent = true;
            in_notes = false;
            continue;
        }
        if trimmed.starts_with("===撰写建议===") {
            in_independent = false;
            in_dependent = false;
            in_notes = true;
            continue;
        }

        if in_notes {
            let note = trimmed
                .trim_start_matches('-')
                .trim_start_matches('•')
                .trim();
            if !note.is_empty() {
                notes.push(note.to_string());
            }
            continue;
        }

        // Parse claim lines like "1. xxx" or "1、xxx"
        if let Some(caps) = CLAIM_RE.captures(trimmed) {
            let number: u32 = caps[1].parse().unwrap_or(0);
            let content = caps[2].trim().to_string();

            // Determine if dependent by checking for explicit reference to another claim
            // Note: "其特征在于" appears in BOTH independent and dependent claims in Chinese patents,
            // so we must NOT use it as a sole indicator.
            let has_dep_reference = content.contains("根据权利要求")
                || content.contains("引用权利要求")
                || content.contains("如权利要求");

            let depends_on = if has_dep_reference {
                extract_depends_on(&content)
            } else {
                None
            };

            // Trust the section markers from LLM output
            let is_indep_claim = in_independent || (!in_dependent && !has_dep_reference);

            let claim = ClaimOutput {
                number,
                r#type: if is_indep_claim {
                    "independent".to_string()
                } else {
                    "dependent".to_string()
                },
                content,
                depends_on,
            };

            if is_indep_claim {
                independents.push(claim);
            } else {
                dependents.push(claim);
            }
        }
    }

    (independents, dependents, notes)
}

pub(crate) fn extract_depends_on(content: &str) -> Option<u32> {
    // Match patterns like "根据权利要求1" or "如权利要求1所述"
    static RE: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"权利要求\s*(\d+)").unwrap());
    RE.captures(content).and_then(|caps| caps[1].parse().ok())
}

pub(crate) fn execute_claim_generator_with_caller<F>(
    input: &ClaimGeneratorInput,
    caller: F,
) -> Result<Value, String>
where
    F: Fn(&str, &str, u32) -> Result<String, String>,
{
    if input.technical_solution.trim().is_empty() {
        return Err("技术方案描述不能为空".to_string());
    }

    let system = build_claim_system_prompt(&input.language, &input.patent_type);
    let user = build_claim_user_prompt(input);
    // max_tokens: 英文专利权利要求通常更长（术语+连接词），故分配更多 token。
    let max_tokens = if input.language == "english" {
        16_000
    } else {
        12_000
    };

    let llm_response = caller(&system, &user, max_tokens)?;
    let (independents, dependents, notes) = parse_claims(&llm_response);

    if independents.is_empty() {
        return Err("LLM未能生成有效的独立权利要求，请检查输入或重试".to_string());
    }

    let claim_count = independents.len() + dependents.len();

    let output = ClaimGeneratorOutput {
        independent_claims: independents,
        dependent_claims: dependents,
        claim_count,
        draft_notes: notes,
        language: input.language.clone(),
    };

    serde_json::to_value(output).map_err(|e| format!("序列化失败: {e}"))
}

pub fn execute_claim_generator(input: &ClaimGeneratorInput) -> Result<Value, String> {
    execute_claim_generator_with_caller(input, default_llm_call)
}
