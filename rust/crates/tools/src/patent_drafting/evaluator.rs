use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Write as _;

use super::{default_eval_mode, default_language, default_llm_call};

// =============================================================================
// InnovationEvaluator — 创新度评估器
// =============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnovationEvaluatorInput {
    /// 技术方案描述
    pub technical_solution: String,
    /// 现有技术描述（可选，用于对比评估）
    #[serde(default)]
    pub prior_art: Option<Vec<String>>,
    /// 技术领域（可选）
    #[serde(default)]
    pub field: Option<String>,
    /// 评估维度: "full" | "novelty" | "inventiveness" | "`technical_effect`" | "`market_value`"
    #[serde(default = "default_eval_mode")]
    pub mode: String,
    /// 输出语言: "chinese" | "english"
    #[serde(default = "default_language")]
    pub language: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InnovationEvaluatorOutput {
    pub(crate) overall_score: f64,
    pub(crate) novelty_score: f64,
    pub(crate) inventiveness_score: f64,
    pub(crate) technical_effect_score: f64,
    pub(crate) market_potential_score: f64,
    pub(crate) score_level: String,
    pub(crate) novelty_assessment: String,
    pub(crate) inventiveness_assessment: String,
    pub(crate) technical_effects: Vec<String>,
    pub(crate) risk_assessment: String,
    pub(crate) recommendations: Vec<String>,
    pub(crate) language: String,
}

pub(crate) fn build_eval_system_prompt(language: &str, mode: &str) -> String {
    let lang_instr = if language == "english" {
        "Please provide the evaluation in English."
    } else {
        "请用中文进行评估，使用专业的专利分析术语。"
    };

    let mode_desc = match mode {
        "novelty" => "仅评估新颖性。",
        "inventiveness" => "仅评估创造性。",
        "technical_effect" => "仅评估技术效果。",
        "market_value" => "仅评估市场价值。",
        _ => "进行全面的创新度评估，包括新颖性、创造性、技术效果和市场潜力。",
    };

    format!(
        "你是一名资深专利分析师和专利审查员，具有多年专利审查和专利分析经验。\
        {lang_instr}\
        {mode_desc}\
        \n\
        评估标准：\
        1. 新颖性（0-100分）：技术方案相对于现有技术是否具有新的技术特征。\
        2. 创造性（0-100分）：技术方案对于本领域技术人员是否非显而易见。\
        3. 技术效果（0-100分）：技术方案是否产生了预料不到的技术效果。\
        4. 市场潜力（0-100分）：技术方案的商业化前景和市场价值。\
        \n\
        评分等级：\
        - 90-100分：卓越（具有重大创新价值和商业前景）\
        - 80-89分：优秀（具有较强的创新性和市场竞争力）\
        - 70-79分：良好（具有一定的创新性，建议申请专利保护）\
        - 60-69分：一般（创新性有限，需要进一步挖掘）\
        - 0-59分：较弱（创新性不足，建议重新评估技术方案）\
        \n\
        输出格式要求：\
        你必须严格按照以下格式输出：\
        \n\
        ===总体评分===\
        [0-100之间的数字]\
        \n\
        ===各维度评分===\
        新颖性：[分数]\
        创造性：[分数]\
        技术效果：[分数]\
        市场潜力：[分数]\
        \n\
        ===评估结论===\
        新颖性评估：[详细评估]\
        创造性评估：[详细评估]\
        \n\
        ===技术效果===\
        - [效果1]\
        - [效果2]\
        \n\
        ===风险评估===\
        [风险分析]\
        \n\
        ===建议===\
        - [建议1]\
        - [建议2]"
    )
}

pub(crate) fn build_eval_user_prompt(input: &InnovationEvaluatorInput) -> String {
    let mut prompt = format!("技术方案描述：\n{}\n", input.technical_solution);

    if let Some(ref field) = input.field {
        let _ = writeln!(prompt, "\n技术领域：{field}");
    }

    let _ = write!(prompt, "\n评估模式：{}\n", input.mode);

    if let Some(ref prior) = input.prior_art {
        if !prior.is_empty() {
            prompt.push_str("\n现有技术：\n");
            for (i, art) in prior.iter().enumerate() {
                let _ = writeln!(prompt, "{}. {}", i + 1, art);
            }
        }
    }

    prompt
}

#[allow(clippy::too_many_lines)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvalSection {
    None,
    Overall,
    Dimensions,
    Conclusion,
    Effects,
    Risk,
    Recommendations,
}

fn detect_eval_section(line: &str) -> EvalSection {
    if line.starts_with("===总体评分===") {
        EvalSection::Overall
    } else if line.starts_with("===各维度评分===") {
        EvalSection::Dimensions
    } else if line.starts_with("===评估结论===") {
        EvalSection::Conclusion
    } else if line.starts_with("===技术效果===") {
        EvalSection::Effects
    } else if line.starts_with("===风险评估===") {
        EvalSection::Risk
    } else if line.starts_with("===建议===") {
        EvalSection::Recommendations
    } else {
        EvalSection::None
    }
}

#[derive(Debug, Default)]
struct DimensionScores {
    novelty: f64,
    inventiveness: f64,
    technical_effect: f64,
    market_potential: f64,
}

fn parse_dimension_line(line: &str, scores: &mut DimensionScores) {
    let Some((key, val)) = line.split_once('：') else {
        return;
    };
    let Ok(score) = val.trim().parse::<f64>() else {
        return;
    };
    let score = score.clamp(0.0, 100.0);
    match key.trim() {
        "新颖性" | "Novelty" => scores.novelty = score,
        "创造性" | "Inventiveness" | "非显而易见性" => scores.inventiveness = score,
        "技术效果" | "Technical Effect" => scores.technical_effect = score,
        "市场潜力" | "Market Potential" | "市场价值" => scores.market_potential = score,
        _ => {}
    }
}

fn parse_conclusion_line(line: &str, novelty: &mut String, inventiveness: &mut String) {
    if line.starts_with("新颖性评估") || line.starts_with("Novelty") {
        if let Some((_, val)) = line.split_once('：') {
            *novelty = val.trim().to_string();
        }
    } else if line.starts_with("创造性评估") || line.starts_with("Inventiveness") {
        if let Some((_, val)) = line.split_once('：') {
            *inventiveness = val.trim().to_string();
        }
    } else if novelty.is_empty() {
        novelty.push_str(line);
    } else if inventiveness.is_empty() {
        inventiveness.push_str(line);
    }
}

pub(crate) fn score_level(score: f64) -> &'static str {
    if score >= 90.0 {
        "excellent"
    } else if score >= 80.0 {
        "good"
    } else if score >= 70.0 {
        "fair"
    } else if score >= 60.0 {
        "average"
    } else {
        "weak"
    }
}

pub(crate) fn parse_evaluation(text: &str) -> InnovationEvaluatorOutput {
    let mut overall_score = 70.0; // 默认"fair"等级，避免无评分时输出极端值
    let mut dims = DimensionScores::default();
    let mut novelty_assessment = String::new();
    let mut inventiveness_assessment = String::new();
    let mut technical_effects = Vec::new();
    let mut risk_assessment = String::new();
    let mut recommendations = Vec::new();

    let mut section = EvalSection::None;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let detected = detect_eval_section(trimmed);
        if detected != EvalSection::None {
            section = detected;
            continue;
        }

        match section {
            EvalSection::Overall => {
                if let Ok(score) = trimmed.parse::<f64>() {
                    overall_score = score.clamp(0.0, 100.0);
                }
            }
            EvalSection::Dimensions => parse_dimension_line(trimmed, &mut dims),
            EvalSection::Conclusion => {
                parse_conclusion_line(
                    trimmed,
                    &mut novelty_assessment,
                    &mut inventiveness_assessment,
                );
            }
            EvalSection::Effects => {
                let effect = trimmed
                    .trim_start_matches('-')
                    .trim_start_matches('•')
                    .trim();
                if !effect.is_empty() {
                    technical_effects.push(effect.to_string());
                }
            }
            EvalSection::Risk => {
                if !risk_assessment.is_empty() {
                    risk_assessment.push(' ');
                }
                risk_assessment.push_str(trimmed);
            }
            EvalSection::Recommendations => {
                let rec = trimmed
                    .trim_start_matches('-')
                    .trim_start_matches('•')
                    .trim();
                if !rec.is_empty() {
                    recommendations.push(rec.to_string());
                }
            }
            EvalSection::None => {}
        }
    }

    InnovationEvaluatorOutput {
        overall_score,
        novelty_score: dims.novelty,
        inventiveness_score: dims.inventiveness,
        technical_effect_score: dims.technical_effect,
        market_potential_score: dims.market_potential,
        score_level: score_level(overall_score).to_string(),
        novelty_assessment,
        inventiveness_assessment,
        technical_effects,
        risk_assessment,
        recommendations,
        language: String::new(),
    }
}

pub(crate) fn execute_innovation_evaluator_with_caller<F>(
    input: &InnovationEvaluatorInput,
    caller: F,
) -> Result<Value, String>
where
    F: Fn(&str, &str, u32) -> Result<String, String>,
{
    if input.technical_solution.trim().is_empty() {
        return Err("技术方案描述不能为空".to_string());
    }

    let system = build_eval_system_prompt(&input.language, &input.mode);
    let user = build_eval_user_prompt(input);
    // max_tokens: 创新评估输出为结构化评分+简短结论，10k-12k 足够覆盖。
    let max_tokens = if input.language == "english" {
        12_000
    } else {
        10_000
    };

    let llm_response = caller(&system, &user, max_tokens)?;
    let mut output = parse_evaluation(&llm_response);
    output.language.clone_from(&input.language);

    serde_json::to_value(output).map_err(|e| format!("序列化失败: {e}"))
}

pub fn execute_innovation_evaluator(input: &InnovationEvaluatorInput) -> Result<Value, String> {
    execute_innovation_evaluator_with_caller(input, default_llm_call)
}
