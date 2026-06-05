use crate::reflection::reflection_llm_client::{ReflectionLlm, ReflectionLlmClient};
use crate::reflection::{ActionIssue, ReflectionConfig, ReflectionResult, Reflector};
use memory::UnifiedMemory;
use serde_json::Value;
use std::sync::Arc;

/// LLM 自反思（Anthropic Extended Thinking 模式）。
///
/// 在 LLM 生成最终输出前插入反思步骤。
/// 支持注入 LLM 客户端和记忆系统，实现真实 LLM 调用与评估闭环。
pub struct LLMReflectionReflector {
    config: ReflectionConfig,
    llm_client: Option<Arc<dyn ReflectionLlm>>,
    memory: Option<Arc<UnifiedMemory>>,
}

impl LLMReflectionReflector {
    pub fn new(config: ReflectionConfig) -> Self {
        Self {
            config,
            llm_client: None,
            memory: None,
        }
    }

    pub fn with_default() -> Self {
        Self {
            config: ReflectionConfig::default(),
            llm_client: None,
            memory: None,
        }
    }

    /// Builder: 注入 LLM 客户端。
    pub fn with_llm_client(mut self, client: Arc<dyn ReflectionLlm>) -> Self {
        self.llm_client = Some(client);
        self
    }

    /// Builder: 注入统一记忆系统。
    pub fn with_memory(mut self, memory: Arc<UnifiedMemory>) -> Self {
        self.memory = Some(memory);
        self
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

    /// 生成默认反思结果（LLM 不可用时的降级方案）。
    fn default_reflection_result(&self, action_type: &str) -> ReflectionResult {
        eprintln!("[tools] LLM 反思器不可用，返回默认结果");
        ReflectionResult {
            action_type: action_type.to_string(),
            success: true,
            score: 75.0,
            issues: vec![],
            improvement_suggestions: vec!["LLM 反思不可用，使用默认评估".to_string()],
            should_retry: false,
            retry_strategy: None,
        }
    }

    /// 执行 LLM 反思。
    fn reflect_on_llm_output(&self, draft_output: &str, context: &str) -> ReflectionResult {
        // 构建反思提示词
        let prompt = self.build_reflection_prompt(
            context,
            draft_output,
            &["完整性", "准确性", "清晰性", "专业性"],
        );

        // 尝试调用 LLM
        let llm_output = match self.llm_client {
            Some(ref client) => {
                let system_prompt = "你是一个专业的输出质量反思分析器。请严格按照指定格式输出分析结果。";
                match client.generate(system_prompt, &prompt) {
                    Ok(output) => {
                        eprintln!("[tools] LLM 反思调用成功 (model={})", client.model_name());
                        output
                    }
                    Err(e) => {
                        eprintln!("[tools] LLM 反思调用失败: {}，降级为默认结果", e);
                        return self.default_reflection_result("llm_generation");
                    }
                }
            }
            None => {
                eprintln!("[tools] 无 LLM 客户端，使用 STUB 反思");
                // STUB: 返回硬编码模拟结果，保持向后兼容
                "===反思分析===\n\
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
                    .to_string()
            }
        };

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

    /// 将反思结果写入记忆系统（使用丰富元数据）。
    fn write_reflection_to_memory(&self, result: &ReflectionResult, session_id: Option<&str>) {
        if let Some(ref memory) = self.memory {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);

            let key = format!(
                "reflection:{}:{}",
                session_id.unwrap_or("default"),
                timestamp
            );

            let content = format!(
                "反思结果 | action_type={} | score={:.1} | success={} | should_retry={} | suggestions={:?}",
                result.action_type,
                result.score,
                result.success,
                result.should_retry,
                result.improvement_suggestions,
            );

            if let Err(e) = memory.remember_rich(
                &key,
                "reflection",
                &content,
                session_id,
                vec!["reflection".to_string(), result.action_type.clone()],
                result.score / 100.0,
                "reflection",
            ) {
                eprintln!("[tools] 写入反思记忆失败: {}", e);
            }
        }
    }
}

impl Reflector for LLMReflectionReflector {
    fn reflect(&self, result: &Value, action_type: &str) -> Result<ReflectionResult, String> {
        let reflection = if action_type == "llm_generation" {
            if let Some(output) = result.get("output") {
                if let Some(draft) = output.as_str() {
                    if let Some(context) = result.get("context") {
                        if let Some(ctx_str) = context.as_str() {
                            self.reflect_on_llm_output(draft, ctx_str)
                        } else {
                            self.default_reflection_result(action_type)
                        }
                    } else {
                        self.default_reflection_result(action_type)
                    }
                } else {
                    self.default_reflection_result(action_type)
                }
            } else {
                self.default_reflection_result(action_type)
            }
        } else {
            // 默认反射
            ReflectionResult {
                action_type: action_type.to_string(),
                success: true,
                score: 80.0,
                issues: vec![],
                improvement_suggestions: vec!["执行良好，无需改进".to_string()],
                should_retry: false,
                retry_strategy: None,
            }
        };

        // P0-4: 写入记忆闭环
        let session_id = result
            .get("session_id")
            .and_then(|v| v.as_str());
        self.write_reflection_to_memory(&reflection, session_id);

        Ok(reflection)
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
    fn test_reflect_on_llm_output_stub() {
        let reflector = LLMReflectionReflector::with_default();
        let reflection = reflector.reflect_on_llm_output("草稿内容", "任务上下文");

        assert!(reflection.action_type == "llm_generation");
        assert!(reflection.score > 0.0);
        assert!(!reflection.improvement_suggestions.is_empty());
    }

    #[test]
    fn test_reflect_without_llm_client() {
        let reflector = LLMReflectionReflector::with_default();
        let result = serde_json::json!({
            "output": "测试输出",
            "context": "测试上下文"
        });

        let reflection = reflector.reflect(&result, "llm_generation").unwrap();
        assert!(reflection.score > 0.0);
    }

    #[test]
    fn test_default_reflection_result() {
        let reflector = LLMReflectionReflector::with_default();
        let result = serde_json::json!({"unknown": "data"});
        let reflection = reflector.reflect(&result, "other_action").unwrap();

        assert_eq!(reflection.action_type, "other_action");
        assert!(reflection.success);
        assert!(!reflection.should_retry);
    }

    #[test]
    fn test_builder_pattern() {
        let reflector = LLMReflectionReflector::new(ReflectionConfig::default());
        // 不注入 LLM 和 memory 也应正常工作
        let result = serde_json::json!({
            "output": "测试",
            "context": "上下文"
        });
        let reflection = reflector.reflect(&result, "llm_generation").unwrap();
        assert!(reflection.score > 0.0);
    }
}
