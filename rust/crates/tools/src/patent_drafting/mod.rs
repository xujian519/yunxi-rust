//! 专利 LLM 撰写工具
//!
//! 提供基于大语言模型的专利文本生成能力：
//! - `ClaimGenerator`: 权利要求书生成
//! - `AbstractDrafter`: 专利摘要起草
//! - `SpecificationDrafter`: 说明书起草
//! - `InnovationEvaluator`: 创新度评估

use api::{AnthropicClient, InputMessage, MessageRequest, OutputContentBlock};

mod abstract_draft;
mod claims;
mod evaluator;
mod specification;

pub use abstract_draft::*;
pub use claims::*;
pub use evaluator::*;
pub use specification::*;

// =============================================================================
// LLM 调用基础设施
// =============================================================================

/// 默认模型标识
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

/// 同步调用 LLM 的底层函数（测试中可注入 mock）
///
/// 注意：使用 `Handle::try_current()` 检测当前是否在 tokio 运行时中；
/// 若不在，则在线程中创建临时运行时执行，避免全局运行时导致测试中的丢弃问题。
fn default_llm_call(system: &str, user: &str, max_tokens: u32) -> Result<String, String> {
    let client = AnthropicClient::from_env().map_err(|e| format!("创建LLM客户端失败: {e}"))?;

    let request = MessageRequest {
        model: DEFAULT_MODEL.to_string(),
        max_tokens,
        messages: vec![InputMessage::user_text(user)],
        system: Some(system.to_string()),
        tools: None,
        tool_choice: None,
        stream: false,
    };

    let response = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        // 已在 tokio 运行时中，直接使用当前运行时
        std::thread::scope(|s| {
            s.spawn(|| handle.block_on(async { client.send_message(&request).await }))
                .join()
                .map_err(|e| format!("LLM线程panic: {e:?}"))?
        })
    } else {
        // 不在 tokio 运行时中，创建临时运行时
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建运行时失败: {e}"))?;
        rt.block_on(async { client.send_message(&request).await })
    }
    .map_err(|e| format!("LLM请求失败: {e}"))?;

    let text = response
        .content
        .into_iter()
        .filter_map(|block| match block {
            OutputContentBlock::Text { text } => Some(text),
            OutputContentBlock::ToolUse { .. } => None,
        })
        .collect::<String>();

    Ok(text)
}

// =============================================================================
// 默认值辅助函数
// =============================================================================

pub(super) fn default_patent_type() -> String {
    "invention".to_string()
}

pub(super) fn default_language() -> String {
    "chinese".to_string()
}

pub(super) fn default_one() -> u8 {
    1
}

pub(super) fn default_five() -> u8 {
    5
}

pub(super) fn default_three_hundred() -> u16 {
    300
}

pub(super) fn default_spec_mode() -> String {
    "full".to_string()
}

pub(super) fn default_detail_level() -> String {
    "standard".to_string()
}

pub(super) fn default_eval_mode() -> String {
    "full".to_string()
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::too_many_lines)]
mod tests {
    use super::*;

    // --- Mock LLM caller ---
    #[allow(clippy::unnecessary_wraps)]
    fn mock_llm_claim_generator(
        _system: &str,
        _user: &str,
        _max_tokens: u32,
    ) -> Result<String, String> {
        Ok(r"
===独立权利要求===
1. 一种智能温控系统，包括温度传感器、控制器和加热元件，其特征在于，所述控制器采用机器学习算法根据历史温度数据预测未来温度变化趋势，并提前调节加热元件的输出功率。

===从属权利要求===
2. 根据权利要求1所述的智能温控系统，其特征在于，所述温度传感器包括多个分布式传感器节点，各节点通过无线通信方式与控制器连接。
3. 根据权利要求1所述的智能温控系统，其特征在于，所述机器学习算法采用长短期记忆网络LSTM模型。

===撰写建议===
- 建议增加关于传感器精度的具体参数
- 建议补充系统节能效果的数据
"
        .to_string())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn mock_llm_abstract_drafter(
        _system: &str,
        _user: &str,
        _max_tokens: u32,
    ) -> Result<String, String> {
        Ok(r"
===摘要===
本发明公开了一种智能温控系统，包括温度传感器、控制器和加热元件。所述控制器采用机器学习算法根据历史温度数据预测未来温度变化趋势，并提前调节加热元件的输出功率。本发明的系统能够实现精准温控，降低能耗，提高用户体验。

===关键词===
- 智能温控
- 机器学习
- 温度预测
- 节能

===撰写建议===
- 摘要中可补充具体应用领域
"
        .to_string())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn mock_llm_spec_drafter(
        _system: &str,
        _user: &str,
        _max_tokens: u32,
    ) -> Result<String, String> {
        Ok(r"
===技术领域===
本发明涉及智能控制技术领域，尤其涉及一种基于机器学习的智能温控系统。

===背景技术===
传统的温控系统通常采用PID控制算法，根据当前温度与设定温度的偏差进行调节。这种反应式控制方式存在滞后性，无法提前应对温度变化，导致能耗较高且温度波动较大。

===发明内容===
针对现有技术的不足，本发明提供一种智能温控系统，能够提前预测温度变化并主动调节。

===附图说明===
图1为本发明的系统结构示意图。

===具体实施方式===
下面结合附图对本发明作进一步详细说明。

===撰写建议===
- 建议补充更多实施例
"
        .to_string())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn mock_llm_evaluator(_system: &str, _user: &str, _max_tokens: u32) -> Result<String, String> {
        Ok(r"
===总体评分===
85

===各维度评分===
新颖性：88
创造性：82
技术效果：86
市场潜力：84

===评估结论===
新颖性评估：该方案将机器学习引入温控领域，具有明确的新颖性。
创造性评估：对于本领域技术人员而言，该方案具有一定的非显而易见性。

===技术效果===
- 能耗降低30%
- 温度波动减少50%
- 响应速度提升2倍

===风险评估===
主要风险在于机器学习模型的训练数据获取和模型泛化能力。

===建议===
- 建议补充对比实验数据
- 建议增加模型鲁棒性分析
"
        .to_string())
    }

    fn mock_llm_error(_system: &str, _user: &str, _max_tokens: u32) -> Result<String, String> {
        Err("模拟LLM调用失败".to_string())
    }

    // --- ClaimGenerator tests ---
    #[test]
    fn test_claim_generator_basic() {
        let input = ClaimGeneratorInput {
            technical_solution: "一种智能温控系统，使用机器学习预测温度变化。".to_string(),
            patent_type: "invention".to_string(),
            field: Some("智能控制".to_string()),
            existing_claims: None,
            language: "chinese".to_string(),
            independent_claim_count: 1,
            dependent_claim_max: 5,
        };

        let result = execute_claim_generator_with_caller(&input, mock_llm_claim_generator).unwrap();
        let independents = result["independentClaims"].as_array().unwrap();
        let dependents = result["dependentClaims"].as_array().unwrap();

        assert!(!independents.is_empty());
        assert!(!dependents.is_empty());
        assert_eq!(
            result["claimCount"].as_u64().unwrap(),
            independents.len() as u64 + dependents.len() as u64
        );
        assert_eq!(result["language"].as_str().unwrap(), "chinese");
    }

    #[test]
    fn test_claim_generator_empty_input_fails() {
        let input = ClaimGeneratorInput {
            technical_solution: String::new(),
            patent_type: "invention".to_string(),
            field: None,
            existing_claims: None,
            language: "chinese".to_string(),
            independent_claim_count: 1,
            dependent_claim_max: 5,
        };

        assert!(execute_claim_generator_with_caller(&input, mock_llm_claim_generator).is_err());
    }

    #[test]
    fn test_claim_generator_llm_error() {
        let input = ClaimGeneratorInput {
            technical_solution: "一种智能温控系统。".to_string(),
            patent_type: "invention".to_string(),
            field: None,
            existing_claims: None,
            language: "chinese".to_string(),
            independent_claim_count: 1,
            dependent_claim_max: 5,
        };

        assert!(execute_claim_generator_with_caller(&input, mock_llm_error).is_err());
    }

    #[test]
    fn test_claim_parse_with_existing_claims() {
        let input = ClaimGeneratorInput {
            technical_solution: "改进的温控系统".to_string(),
            patent_type: "utilityModel".to_string(),
            field: None,
            existing_claims: Some(vec!["1. 一种温控系统，包括传感器和控制器。".to_string()]),
            language: "chinese".to_string(),
            independent_claim_count: 1,
            dependent_claim_max: 3,
        };

        let result = execute_claim_generator_with_caller(&input, mock_llm_claim_generator).unwrap();
        assert!(!result["independentClaims"].as_array().unwrap().is_empty());
    }

    // --- AbstractDrafter tests ---
    #[test]
    fn test_abstract_drafter_basic() {
        let input = AbstractDrafterInput {
            technical_solution: "一种智能温控系统，使用机器学习预测温度变化。".to_string(),
            patent_type: "invention".to_string(),
            key_features: Some(vec!["机器学习预测".to_string(), "提前调节".to_string()]),
            language: "chinese".to_string(),
            max_words: 300,
        };

        let result =
            execute_abstract_drafter_with_caller(&input, mock_llm_abstract_drafter).unwrap();
        let abstract_text = result["abstractText"].as_str().unwrap();
        let keywords = result["keywords"].as_array().unwrap();

        assert!(!abstract_text.is_empty());
        assert!(!keywords.is_empty());
        assert_eq!(result["language"].as_str().unwrap(), "chinese");
    }

    #[test]
    fn test_abstract_drafter_empty_input_fails() {
        let input = AbstractDrafterInput {
            technical_solution: String::new(),
            patent_type: "invention".to_string(),
            key_features: None,
            language: "chinese".to_string(),
            max_words: 300,
        };

        assert!(execute_abstract_drafter_with_caller(&input, mock_llm_abstract_drafter).is_err());
    }

    // --- SpecificationDrafter tests ---
    #[test]
    fn test_spec_drafter_basic() {
        let input = SpecificationDrafterInput {
            technical_solution: "一种智能温控系统，使用机器学习预测温度变化。".to_string(),
            patent_type: "invention".to_string(),
            mode: "full".to_string(),
            field: Some("智能控制".to_string()),
            prior_art: Some("传统PID控制".to_string()),
            technical_effects: Some(vec!["节能".to_string(), "精准温控".to_string()]),
            language: "chinese".to_string(),
            detail_level: "standard".to_string(),
        };

        let result =
            execute_specification_drafter_with_caller(&input, mock_llm_spec_drafter).unwrap();
        let sections = result["sections"].as_array().unwrap();

        assert!(!sections.is_empty());
        assert!(result["totalWordCount"].as_u64().unwrap() > 0);
        assert_eq!(result["language"].as_str().unwrap(), "chinese");
    }

    #[test]
    fn test_spec_drafter_partial_mode() {
        let input = SpecificationDrafterInput {
            technical_solution: "一种智能温控系统。".to_string(),
            patent_type: "invention".to_string(),
            mode: "summary".to_string(),
            field: None,
            prior_art: None,
            technical_effects: None,
            language: "chinese".to_string(),
            detail_level: "concise".to_string(),
        };

        let result =
            execute_specification_drafter_with_caller(&input, mock_llm_spec_drafter).unwrap();
        assert!(!result["sections"].as_array().unwrap().is_empty());
    }

    // --- InnovationEvaluator tests ---
    #[test]
    fn test_innovation_evaluator_basic() {
        let input = InnovationEvaluatorInput {
            technical_solution: "一种智能温控系统，使用机器学习预测温度变化。".to_string(),
            prior_art: Some(vec!["传统PID控制".to_string()]),
            field: Some("智能控制".to_string()),
            mode: "full".to_string(),
            language: "chinese".to_string(),
        };

        let result = execute_innovation_evaluator_with_caller(&input, mock_llm_evaluator).unwrap();

        assert!(result["overallScore"].as_f64().unwrap() > 0.0);
        assert!(result["noveltyScore"].as_f64().unwrap() > 0.0);
        assert!(result["inventivenessScore"].as_f64().unwrap() > 0.0);
        assert_eq!(result["scoreLevel"].as_str().unwrap(), "good");
        assert!(!result["technicalEffects"].as_array().unwrap().is_empty());
        assert!(!result["recommendations"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_innovation_evaluator_novelty_mode() {
        let input = InnovationEvaluatorInput {
            technical_solution: "一种智能温控系统。".to_string(),
            prior_art: None,
            field: None,
            mode: "novelty".to_string(),
            language: "chinese".to_string(),
        };

        let result = execute_innovation_evaluator_with_caller(&input, mock_llm_evaluator).unwrap();
        assert!(result["overallScore"].as_f64().unwrap() >= 0.0);
    }

    // --- Parser tests ---
    #[test]
    fn test_parse_claims_edge_cases() {
        let text = r"
===独立权利要求===
1. 一种方法，包括步骤A和步骤B。

===从属权利要求===
2. 根据权利要求1所述的方法，其特征在于，还包括步骤C。

===撰写建议===
- 建议补充
";
        let (ind, dep, notes) = parse_claims(text);
        assert_eq!(ind.len(), 1);
        assert_eq!(dep.len(), 1);
        assert_eq!(dep[0].depends_on, Some(1));
        assert!(!notes.is_empty());
    }

    #[test]
    fn test_parse_claims_multiple_independent() {
        let text = r"
===独立权利要求===
1. 一种装置，包括X。
2. 一种方法，包括步骤Y。
";
        let (ind, dep, _) = parse_claims(text);
        assert_eq!(ind.len(), 2);
        assert!(dep.is_empty());
        assert_eq!(ind[0].number, 1);
        assert_eq!(ind[1].number, 2);
    }

    #[test]
    fn test_parse_abstract_multiline() {
        let text = r"
===摘要===
本发明公开了一种智能温控系统。
该系统包括温度传感器和控制器。

===关键词===
- 温控
- 智能

===撰写建议===
- 建议1
";
        let (abs, kws, notes) = parse_abstract(text, "chinese");
        assert!(abs.contains("智能温控系统"));
        assert!(abs.contains("温度传感器"));
        assert_eq!(kws.len(), 2);
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn test_parse_specification_sections() {
        let text = r"
===技术领域===
本发明涉及A领域。

===背景技术===
现有技术存在B问题。

===撰写建议===
- 建议补充C
";
        let (sections, notes) = parse_specification(text, "chinese");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "技术领域");
        assert_eq!(sections[1].title, "背景技术");
        assert_eq!(notes.len(), 1);
    }

    #[test]
    fn test_parse_evaluation_scores() {
        let text = r"
===总体评分===
92

===各维度评分===
新颖性：95
创造性：90
技术效果：93
市场潜力：90
";
        let output = parse_evaluation(text);
        assert_eq!(output.overall_score, 92.0);
        assert_eq!(output.novelty_score, 95.0);
        assert_eq!(output.inventiveness_score, 90.0);
        assert_eq!(output.technical_effect_score, 93.0);
        assert_eq!(output.market_potential_score, 90.0);
        assert_eq!(output.score_level, "excellent");
    }

    #[test]
    fn test_extract_depends_on() {
        assert_eq!(extract_depends_on("根据权利要求1所述的方法"), Some(1));
        assert_eq!(extract_depends_on("如权利要求3所述的装置"), Some(3));
        assert_eq!(extract_depends_on("一种独立的装置"), None);
    }
}
