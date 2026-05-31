use crate::reflection::{ActionIssue, ReflectionConfig, ReflectionResult, Reflector};
use serde_json::Value;

/// LLM 自反思（Anthropic Extended Thinking 模式）。
///
/// 在 LLM 生成最终输出前插入反思步骤。
pub struct LLMReflectionReflector {
    config: ReflectionConfig,
}

impl LLMReflectionReflector {
    pub fn new(config: ReflectionConfig) -> Self {
        Self { config }
    }

    pub fn with_default() -> Self {
        Self {
            config: ReflectionConfig::default(),
        }
    }

    /// 构建 LLM 反思提示词模板。
    pub fn build_reflection_prompt(
        &self,
        task_description: &str,
        draft_output: &str,
        evaluation_criteria: &[&str],
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("你正在进行自我反思，以改进你的输出质量。\n\n");
        prompt.push_str(&format!("任务描述：\n{}\n\n", task_description));
        prompt.push_str(&format!("你的草稿输出：\n{}\n\n", draft_output));

        prompt.push_str("评估标准：\n");
        for (i, criterion) in evaluation_criteria.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", i + 1, criterion));
        }

        prompt.push_str(
            "\n请按照以下步骤进行自我反思：\n\
            1. 审查草稿输出，识别不足之处\n\
            2. 检查是否符合所有评估标准\n\
            3. 提出具体的改进建议\n\
            4. 评估是否需要重新生成完整输出\n\
            \n\
            严格按照以下格式输出：\n\
            ===反思分析===\n\
            [对草稿输出的详细分析]\n\
            \n\
            ===识别的问题===\n\
            [问题列表，每行一个]\n\
            \n\
            ===改进建议===\n\
            [具体改进建议列表]\n\
            \n\
            ===是否需要重新生成===\n\
            [yes/no]\n\
            \n\
            ===改进后的输出===\n\
            [如果需要重新生成，在此提供；否则保持原样]",
        );

        prompt
    }

    /// 解析 LLM 反思输出。
    pub fn parse_reflection_output(
        llm_output: &str,
    ) -> (Vec<String>, Vec<String>, bool, Option<String>) {
        let mut current_section = String::new();
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut needs_regeneration = false;
        let mut improved_output = None;

        for line in llm_output.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("===") {
                current_section = trimmed.to_string();
                continue;
            }

            match current_section.as_str() {
                "===识别的问题===" => {
                    if !trimmed.is_empty() {
                        issues.push(trimmed.to_string());
                    }
                }
                "===改进建议===" => {
                    if !trimmed.is_empty() {
                        suggestions.push(trimmed.to_string());
                    }
                }
                "===是否需要重新生成===" => {
                    needs_regeneration = trimmed.to_lowercase().starts_with("yes");
                }
                "===改进后的输出===" => {
                    if !trimmed.is_empty() {
                        improved_output = Some(trimmed.to_string());
                    }
                }
                _ => {}
            }
        }

        (issues, suggestions, needs_regeneration, improved_output)
    }

    /// 执行 LLM 反思。
    fn reflect_on_llm_output(&self, draft_output: &str, context: &str) -> ReflectionResult {
        // 构建反思提示词
        let _prompt = self.build_reflection_prompt(
            context,
            draft_output,
            &["完整性", "准确性", "清晰性", "专业性"],
        );

        // STUB: 返回硬编码模拟结果，待集成 LLM 模块后替换。
        let llm_output = "===反思分析===\n\
            草稿输出在完整性方面表现良好，但在某些细节上需要改进。\n\
            \n\
            ===识别的问题===\n\
            技术描述不够详细\n\
            缺少具体实施方式的描述\n\
            \n\
            ===改进建议===\n\
            增加技术细节的描述\n\
            添加具体的实施步骤\n\
            \n\
            ===是否需要重新生成===\n\
            no"
        .to_string();

        let (issues, suggestions, needs_regeneration, _) =
            Self::parse_reflection_output(&llm_output);

        let success = !needs_regeneration;
        let score = if issues.is_empty() { 85.0 } else { 65.0 };

        let action_issues: Vec<ActionIssue> = issues
            .iter()
            .map(|issue| ActionIssue {
                category: "内容质量".to_string(),
                severity: "medium".to_string(),
                description: issue.clone(),
                location: None,
            })
            .collect();

        ReflectionResult {
            action_type: "llm_generation".to_string(),
            success,
            score,
            issues: action_issues,
            improvement_suggestions: suggestions,
            should_retry: needs_regeneration,
            retry_strategy: if needs_regeneration {
                Some("根据反思建议重新生成".to_string())
            } else {
                None
            },
        }
    }
}

impl Reflector for LLMReflectionReflector {
    fn reflect(&self, result: &Value, action_type: &str) -> Result<ReflectionResult, String> {
        if action_type == "llm_generation" {
            if let Some(output) = result.get("output") {
                if let Some(draft) = output.as_str() {
                    if let Some(context) = result.get("context") {
                        if let Some(ctx_str) = context.as_str() {
                            return Ok(self.reflect_on_llm_output(draft, ctx_str));
                        }
                    }
                }
            }
        }

        // 默认反射
        Ok(ReflectionResult {
            action_type: action_type.to_string(),
            success: true,
            score: 80.0,
            issues: vec![],
            improvement_suggestions: vec!["执行良好，无需改进".to_string()],
            should_retry: false,
            retry_strategy: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_reflection_prompt() {
        let reflector = LLMReflectionReflector::with_default();
        let criteria = vec!["完整性", "准确性"];

        let prompt = reflector.build_reflection_prompt("测试任务", "测试草稿", &criteria);

        assert!(prompt.contains("测试任务"));
        assert!(prompt.contains("测试草稿"));
        assert!(prompt.contains("完整性"));
        assert!(prompt.contains("自我反思"));
    }

    #[test]
    fn test_parse_reflection_output() {
        let output = "===反思分析===\n\
            分析内容\n\
            \n\
            ===识别的问题===\n\
            问题1\n\
            问题2\n\
            \n\
            ===改进建议===\n\
            建议1\n\
            建议2\n\
            \n\
            ===是否需要重新生成===\n\
            yes";

        let (issues, suggestions, needs_reg, improved) =
            LLMReflectionReflector::parse_reflection_output(output);

        assert_eq!(issues.len(), 2);
        assert_eq!(suggestions.len(), 2);
        assert!(needs_reg);
        assert!(improved.is_none());
    }

    #[test]
    fn test_reflect_on_llm_output() {
        let reflector = LLMReflectionReflector::with_default();
        let reflection = reflector.reflect_on_llm_output("草稿内容", "任务上下文");

        assert!(reflection.action_type == "llm_generation");
        assert!(reflection.score > 0.0);
        assert!(!reflection.improvement_suggestions.is_empty());
    }
}
