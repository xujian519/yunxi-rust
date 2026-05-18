use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write as _;

use super::{default_language, default_llm_call, default_patent_type, default_three_hundred};

// =============================================================================
// AbstractDrafter — 专利摘要起草器
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbstractDrafterInput {
    /// 技术方案描述
    pub technical_solution: String,
    /// 专利类型: "invention" | "utilityModel" | "design"
    #[serde(default = "default_patent_type")]
    pub patent_type: String,
    /// 关键技术特征（可选，用于突出摘要重点）
    #[serde(default)]
    pub key_features: Option<Vec<String>>,
    /// 输出语言: "chinese" | "english"
    #[serde(default = "default_language")]
    pub language: String,
    /// 最大字数限制（默认300）
    #[serde(default = "default_three_hundred")]
    pub max_words: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AbstractDrafterOutput {
    abstract_text: String,
    word_count: usize,
    keywords: Vec<String>,
    language: String,
    draft_notes: Vec<String>,
}

pub(crate) fn build_abstract_system_prompt(
    language: &str,
    patent_type: &str,
    max_words: u16,
) -> String {
    let type_desc = match patent_type {
        "utilityModel" => "实用新型专利",
        "design" => "外观设计专利",
        _ => "发明专利",
    };
    let lang_instr = if language == "english" {
        "Please write the abstract in English."
    } else {
        "请用中文撰写摘要，使用规范的专利法术语。"
    };

    format!(
        "你是一名资深专利代理师，专精于{type_desc}的摘要撰写。\
        {lang_instr}\
        \n\
        撰写要求：\
        1. 摘要应当写明发明的名称和所属技术领域，并清楚地反映所要解决的技术问题、解决该问题的技术方案的要点以及主要用途。\
        2. 摘要全文（包括标点符号）不超过{max_words}字。\
        3. 摘要中不得使用商业性宣传用语。\
        4. 摘要应当简洁明了，突出技术方案的核心创新点。\
        5. 如果技术方案涉及装置/系统，应描述其组成和连接关系；如果涉及方法，应描述其步骤和流程。\
        \n\
        输出格式要求：\
        你必须严格按照以下格式输出：\
        \n\
        ===摘要===\
        [摘要正文，一段式]\
        \n\
        ===关键词===\
        - [关键词1]\
        - [关键词2]\
        ...\
        \n\
        ===撰写建议===\
        - [建议1]\
        - [建议2]"
    )
}

pub(crate) fn build_abstract_user_prompt(input: &AbstractDrafterInput) -> String {
    let mut prompt = format!("技术方案描述：\n{}\n", input.technical_solution);
    let _ = write!(prompt, "\n专利类型：{}\n", input.patent_type);
    let _ = write!(prompt, "\n字数限制：{}字\n", input.max_words);

    if let Some(ref features) = input.key_features {
        if !features.is_empty() {
            prompt.push_str("\n需要突出的关键技术特征：\n");
            for f in features {
                let _ = writeln!(prompt, "- {f}");
            }
        }
    }

    prompt
}

pub(crate) fn parse_abstract(text: &str, language: &str) -> (String, Vec<String>, Vec<String>) {
    let mut abstract_text = String::new();
    let mut keywords = Vec::new();
    let mut notes = Vec::new();

    let mut in_abstract = false;
    let mut in_keywords = false;
    let mut in_notes = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("===摘要===") || trimmed.starts_with("===Abstract===") {
            in_abstract = true;
            in_keywords = false;
            in_notes = false;
            continue;
        }
        if trimmed.starts_with("===关键词===") || trimmed.starts_with("===Keywords===") {
            in_abstract = false;
            in_keywords = true;
            in_notes = false;
            continue;
        }
        if trimmed.starts_with("===撰写建议===") || trimmed.starts_with("===Drafting Notes===")
        {
            in_abstract = false;
            in_keywords = false;
            in_notes = true;
            continue;
        }

        if in_abstract {
            if !abstract_text.is_empty() {
                // Only insert space for English text; Chinese text should not have extra spaces
                if language == "english" {
                    abstract_text.push(' ');
                }
            }
            abstract_text.push_str(trimmed);
        } else if in_keywords {
            let kw = trimmed
                .trim_start_matches('-')
                .trim_start_matches('•')
                .trim();
            if !kw.is_empty() {
                keywords.push(kw.to_string());
            }
        } else if in_notes {
            let note = trimmed
                .trim_start_matches('-')
                .trim_start_matches('•')
                .trim();
            if !note.is_empty() {
                notes.push(note.to_string());
            }
        }
    }

    (abstract_text.trim().to_string(), keywords, notes)
}

pub(crate) fn execute_abstract_drafter_with_caller<F>(
    input: &AbstractDrafterInput,
    caller: F,
) -> Result<Value, String>
where
    F: Fn(&str, &str, u32) -> Result<String, String>,
{
    if input.technical_solution.trim().is_empty() {
        return Err("技术方案描述不能为空".to_string());
    }

    let system = build_abstract_system_prompt(&input.language, &input.patent_type, input.max_words);
    let user = build_abstract_user_prompt(input);
    // max_tokens: 摘要长度固定（中文约 300 字/英文约 150 词），6k-8k 足够生成。
    let max_tokens = if input.language == "english" {
        8_000
    } else {
        6_000
    };

    let llm_response = caller(&system, &user, max_tokens)?;
    let (abstract_text, keywords, notes) = parse_abstract(&llm_response, &input.language);

    if abstract_text.is_empty() {
        return Err("LLM未能生成有效的摘要，请检查输入或重试".to_string());
    }

    let word_count = if input.language == "english" {
        abstract_text.split_whitespace().count()
    } else {
        abstract_text.chars().count()
    };

    let output = AbstractDrafterOutput {
        abstract_text,
        word_count,
        keywords,
        language: input.language.clone(),
        draft_notes: notes,
    };

    serde_json::to_value(output).map_err(|e| format!("序列化失败: {e}"))
}

pub fn execute_abstract_drafter(input: &AbstractDrafterInput) -> Result<Value, String> {
    execute_abstract_drafter_with_caller(input, default_llm_call)
}
