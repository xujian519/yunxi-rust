use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write as _;

use super::{
    default_detail_level, default_language, default_llm_call, default_patent_type,
    default_spec_mode,
};

// =============================================================================
// SpecificationDrafter — 说明书起草器
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecificationDrafterInput {
    /// 技术方案描述
    pub technical_solution: String,
    /// 专利类型: "invention" | "utilityModel" | "design"
    #[serde(default = "default_patent_type")]
    pub patent_type: String,
    /// 起草模式: "full" | "background" | "summary" | "`detailed_description`" | "embodiments"
    #[serde(default = "default_spec_mode")]
    pub mode: String,
    /// 技术领域（可选）
    #[serde(default)]
    pub field: Option<String>,
    /// 现有技术背景（可选）
    #[serde(default)]
    pub prior_art: Option<String>,
    /// 期望的技术效果（可选）
    #[serde(default)]
    pub technical_effects: Option<Vec<String>>,
    /// 输出语言: "chinese" | "english"
    #[serde(default = "default_language")]
    pub language: String,
    /// 详细程度: "concise" | "standard" | "detailed"（默认standard）
    #[serde(default = "default_detail_level")]
    pub detail_level: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SpecificationDrafterOutput {
    sections: Vec<SpecSection>,
    total_word_count: usize,
    language: String,
    draft_notes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpecSection {
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) word_count: usize,
}

pub(crate) fn build_spec_system_prompt(
    language: &str,
    patent_type: &str,
    mode: &str,
    detail: &str,
) -> String {
    let type_desc = match patent_type {
        "utilityModel" => "实用新型专利",
        "design" => "外观设计专利",
        _ => "发明专利",
    };
    let lang_instr = if language == "english" {
        "Please write the specification in English."
    } else {
        "请用中文撰写说明书，使用规范的专利法术语和书面语。"
    };

    let mode_desc = match mode {
        "background" => "仅撰写「技术领域」和「背景技术」部分。",
        "summary" => {
            "仅撰写「发明内容」/「实用新型内容」部分（包括技术问题、技术方案、有益效果）。"
        }
        "detailed_description" => "仅撰写「具体实施方式」部分。",
        "embodiments" => "仅撰写「具体实施例」部分。",
        _ => "撰写完整的说明书，包括：技术领域、背景技术、发明内容、附图说明、具体实施方式。",
    };

    let detail_desc = match detail {
        "concise" => "撰写应简明扼要，只包含必要的技术信息。",
        "detailed" => "撰写应尽可能详细，提供丰富的实施细节和变型方案。",
        _ => "撰写应详细且完整，符合标准专利申请文件的要求。",
    };

    format!(
        "你是一名资深专利代理师，专精于{type_desc}的说明书撰写。\
        {lang_instr}\
        {mode_desc}\
        {detail_desc}\
        \n\
        撰写要求：\
        1. 说明书应当对发明作出清楚、完整的说明，以所属技术领域的技术人员能够实现为准。\
        2. 技术领域部分应明确指出本发明所属或直接应用的技术领域。\
        3. 背景技术部分应写明对发明的理解、检索、审查有用的背景技术，并引证反映这些背景技术的文件。\
        4. 发明内容部分应写明发明所要解决的技术问题、解决该技术问题的技术方案以及有益效果。\
        5. 具体实施方式部分应详细描述实现发明的优选方式，必要时举例说明。\
        6. 使用规范的技术术语，避免使用商业性宣传用语。\
        \n\
        输出格式要求：\
        你必须严格按照以下格式输出，每个部分用===标记：\
        \n\
        ===技术领域===\
        [内容]\
        \n\
        ===背景技术===\
        [内容]\
        \n\
        ===发明内容===\
        [内容]\
        \n\
        ===附图说明===\
        [内容]\
        \n\
        ===具体实施方式===\
        [内容]\
        \n\
        ===撰写建议===\
        - [建议1]\
        - [建议2]"
    )
}

pub(crate) fn build_spec_user_prompt(input: &SpecificationDrafterInput) -> String {
    let mut prompt = format!("技术方案描述：\n{}\n", input.technical_solution);
    let _ = write!(prompt, "\n专利类型：{}\n", input.patent_type);
    let _ = write!(prompt, "\n起草模式：{}\n", input.mode);
    let _ = write!(prompt, "\n详细程度：{}\n", input.detail_level);

    if let Some(ref field) = input.field {
        let _ = writeln!(prompt, "\n技术领域：{field}");
    }

    if let Some(ref prior) = input.prior_art {
        if !prior.is_empty() {
            let _ = writeln!(prompt, "\n现有技术背景：\n{prior}");
        }
    }

    if let Some(ref effects) = input.technical_effects {
        if !effects.is_empty() {
            prompt.push_str("\n期望的技术效果：\n");
            for e in effects {
                let _ = writeln!(prompt, "- {e}");
            }
        }
    }

    prompt
}

pub(crate) fn parse_specification(text: &str, language: &str) -> (Vec<SpecSection>, Vec<String>) {
    let mut sections = Vec::new();
    let mut notes = Vec::new();

    let section_markers = [
        ("技术领域", "Technical Field"),
        ("背景技术", "Background Art"),
        ("发明内容", "Summary"),
        ("实用新型内容", "Utility Model Content"),
        ("附图说明", "Description of Drawings"),
        ("具体实施方式", "Detailed Description"),
        ("具体实施例", "Embodiments"),
        ("撰写建议", "Drafting Notes"),
    ];

    let mut current_title = String::new();
    let mut current_content = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if this is a section marker
        let is_marker = section_markers.iter().any(|(cn, en)| {
            trimmed == format!("==={cn}===")
                || trimmed == format!("=== {cn} ===")
                || trimmed == format!("==={en}===")
                || trimmed == format!("=== {en} ===")
        });

        if is_marker {
            // Save previous section
            if !current_title.is_empty() && !current_content.trim().is_empty() {
                let content = current_content.trim().to_string();
                let word_count = if language == "english" {
                    content.split_whitespace().count()
                } else {
                    content.chars().count()
                };

                if current_title.contains("建议") || current_title.contains("Notes") {
                    // Parse notes from content
                    for note_line in content.lines() {
                        let note = note_line
                            .trim_start_matches('-')
                            .trim_start_matches('•')
                            .trim();
                        if !note.is_empty() {
                            notes.push(note.to_string());
                        }
                    }
                } else {
                    sections.push(SpecSection {
                        title: current_title.clone(),
                        content,
                        word_count,
                    });
                }
            }

            // Extract title from marker
            current_title = trimmed
                .trim_start_matches("===")
                .trim_end_matches("===")
                .trim()
                .to_string();
            current_content.clear();
            continue;
        }

        if !current_content.is_empty() {
            current_content.push('\n');
        }
        current_content.push_str(trimmed);
    }

    // Save last section
    if !current_title.is_empty() && !current_content.trim().is_empty() {
        let content = current_content.trim().to_string();
        let word_count = if language == "english" {
            content.split_whitespace().count()
        } else {
            content.chars().count()
        };

        if current_title.contains("建议") || current_title.contains("Notes") {
            for note_line in content.lines() {
                let note = note_line
                    .trim_start_matches('-')
                    .trim_start_matches('•')
                    .trim();
                if !note.is_empty() {
                    notes.push(note.to_string());
                }
            }
        } else {
            sections.push(SpecSection {
                title: current_title,
                content,
                word_count,
            });
        }
    }

    (sections, notes)
}

pub(crate) fn execute_specification_drafter_with_caller<F>(
    input: &SpecificationDrafterInput,
    caller: F,
) -> Result<Value, String>
where
    F: Fn(&str, &str, u32) -> Result<String, String>,
{
    if input.technical_solution.trim().is_empty() {
        return Err("技术方案描述不能为空".to_string());
    }

    let system = build_spec_system_prompt(
        &input.language,
        &input.patent_type,
        &input.mode,
        &input.detail_level,
    );
    let user = build_spec_user_prompt(input);
    // max_tokens: 说明书是专利文档中最长的部分（通常 3000-8000 字），需最大 token 预算。
    let max_tokens = if input.language == "english" {
        24_000
    } else {
        20_000
    };

    let llm_response = caller(&system, &user, max_tokens)?;
    let (sections, notes) = parse_specification(&llm_response, &input.language);

    if sections.is_empty() {
        return Err("LLM未能生成有效的说明书内容，请检查输入或重试".to_string());
    }

    let total_word_count = sections.iter().map(|s| s.word_count).sum();

    let output = SpecificationDrafterOutput {
        sections,
        total_word_count,
        language: input.language.clone(),
        draft_notes: notes,
    };

    serde_json::to_value(output).map_err(|e| format!("序列化失败: {e}"))
}

pub fn execute_specification_drafter(input: &SpecificationDrafterInput) -> Result<Value, String> {
    execute_specification_drafter_with_caller(input, default_llm_call)
}
