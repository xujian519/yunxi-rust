//! 宪法引擎 LLM 分析 trait 与默认实现
//!
//! 当关键词匹配无法覆盖规则检查时（`_` 分支），
//! 使用 LLM 进行深度语义分析，提升审查准确性。

use crate::model::ConstitutionalRule;
use serde::{Deserialize, Serialize};

/// LLM 深度分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmAnalysisResult {
    /// 规则是否通过
    pub passed: bool,
    /// 置信度 [0.0, 1.0]
    pub confidence: f64,
    /// 详细说明
    pub details: Vec<String>,
    /// LLM 推理过程摘要
    pub reasoning: String,
}

/// 宪法引擎 LLM 分析 trait
///
/// 为规则检查提供深度语义分析能力，替代 `_` 分支的低置信度结果。
pub trait ConstitutionalLlmAnalyzer: Send + Sync {
    /// 对规则进行 LLM 深度分析。
    ///
    /// # Arguments
    /// * `rule` - 待检查的宪法规则
    /// * `input_text` - 用户输入文本
    /// * `output_text` - 工具输出文本（可选）
    /// * `phase` - 当前推理阶段
    ///
    /// # Returns
    /// LLM 分析结果，失败时返回 `Err(String)`
    fn analyze(
        &self,
        rule: &ConstitutionalRule,
        input_text: &str,
        output_text: Option<&str>,
        phase: &str,
    ) -> Result<LlmAnalysisResult, String>;
}

/// 默认 LLM 分析器实现
///
/// 通过 reqwest::blocking 同步调用 OpenAI 兼容 API。
pub struct LlmAnalyzerImpl {
    base_url: String,
    api_key: String,
    model: String,
}

impl LlmAnalyzerImpl {
    /// 创建新的 LLM 分析器。
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
        }
    }

    /// 从环境变量自动构建分析器。
    pub fn from_env(model_override: Option<&str>) -> Option<Self> {
        let base_url = std::env::var("CONSTITUTIONAL_LLM_BASE_URL")
            .ok()
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .or_else(|| std::env::var("OPENAI_API_BASE").ok())?;

        let api_key = std::env::var("CONSTITUTIONAL_LLM_API_KEY")
            .ok()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())?;

        let model = model_override
            .map(str::to_string)
            .or_else(|| std::env::var("CONSTITUTIONAL_LLM_MODEL").ok())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

        Some(Self::new(base_url, api_key, model))
    }

    /// 构建 LLM 分析提示词。
    fn build_analysis_prompt(
        rule: &ConstitutionalRule,
        input_text: &str,
        output_text: Option<&str>,
        phase: &str,
    ) -> (String, String) {
        let system_prompt = format!(
            "你是一个专业的专利审查规则分析器。你的任务是根据给定的宪法规则，\
             判断输入文本是否符合该规则。\n\n\
             规则名称：{}\n\
             规则描述：{}\n\
             严重程度：{}\n\
             动作：{}\n\
             法律依据：{}\n\
             当前阶段：{}\n\n\
             请严格按照以下 JSON 格式输出分析结果（不要输出其他内容）：\n\
             {{\n  \
               \"passed\": true/false,\n  \
               \"confidence\": 0.0-1.0,\n  \
               \"details\": [\"detail1\", \"detail2\"],\n  \
               \"reasoning\": \"推理过程摘要\"\n\
             }}",
            rule.name, rule.description, rule.severity, rule.action, rule.legal_basis, phase,
        );

        let mut user_prompt = format!("输入文本：\n{}\n", input_text);
        if let Some(output) = output_text {
            user_prompt.push_str(&format!("\n输出文本：\n{}\n", output));
        }
        user_prompt.push_str("\n请分析上述文本是否符合规则要求。");

        (system_prompt, user_prompt)
    }

    /// 解析 LLM 返回的 JSON 分析结果。
    fn parse_analysis_response(raw: &str) -> Result<LlmAnalysisResult, String> {
        // 尝试从原始文本中提取 JSON
        let json_str = if raw.contains('{') && raw.contains('}') {
            let start = raw.find('{').unwrap_or(0);
            let end = raw.rfind('}').map(|i| i + 1).unwrap_or(raw.len());
            &raw[start..end]
        } else {
            raw
        };

        let parsed: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| format!("JSON 解析失败: {e}"))?;

        let passed = parsed
            .get("passed")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let confidence = parsed
            .get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);

        let details = parsed
            .get("details")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let reasoning = parsed
            .get("reasoning")
            .and_then(|v| v.as_str())
            .unwrap_or("无推理过程")
            .to_string();

        Ok(LlmAnalysisResult {
            passed,
            confidence,
            details,
            reasoning,
        })
    }

    /// 同步调用 LLM API。
    fn call_llm(&self, system_prompt: &str, user_prompt: &str) -> Result<String, String> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.2,
            "max_tokens": 1024
        });

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .map_err(|e| format!("LLM 请求失败: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().unwrap_or_default();
            return Err(format!("LLM 返回错误 {}: {}", status, text));
        }

        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("解析 LLM 响应失败: {e}"))?;

        // 显式类型标注以避免编译器推断失败
        let choices: Option<&serde_json::Value> = json.get("choices");
        let first_choice: Option<&serde_json::Value> =
            choices.and_then(|c: &serde_json::Value| c.get(0));
        let message: Option<&serde_json::Value> =
            first_choice.and_then(|c: &serde_json::Value| c.get("message"));
        let content: Option<&serde_json::Value> =
            message.and_then(|m: &serde_json::Value| m.get("content"));
        let text: Option<&str> = content.and_then(|c: &serde_json::Value| c.as_str());

        text.map(str::to_string)
            .ok_or_else(|| "LLM 响应格式异常".to_string())
    }
}

impl ConstitutionalLlmAnalyzer for LlmAnalyzerImpl {
    fn analyze(
        &self,
        rule: &ConstitutionalRule,
        input_text: &str,
        output_text: Option<&str>,
        phase: &str,
    ) -> Result<LlmAnalysisResult, String> {
        let (system_prompt, user_prompt) =
            Self::build_analysis_prompt(rule, input_text, output_text, phase);

        eprintln!(
            "[constitutional-engine] LLM 深度分析规则: {} (phase={})",
            rule.name, phase
        );

        let raw_response = self.call_llm(&system_prompt, &user_prompt)?;

        Self::parse_analysis_response(&raw_response)
    }
}

/// 空操作分析器（LLM 不可用时的降级方案）。
///
/// 始终返回低置信度的通过结果，保持现有行为。
pub struct NoopLlmAnalyzer;

impl ConstitutionalLlmAnalyzer for NoopLlmAnalyzer {
    fn analyze(
        &self,
        rule: &ConstitutionalRule,
        _input_text: &str,
        _output_text: Option<&str>,
        _phase: &str,
    ) -> Result<LlmAnalysisResult, String> {
        Ok(LlmAnalysisResult {
            passed: true,
            confidence: 0.5,
            details: vec![format!(
                "规则 '{}' 需要深度 LLM 辅助检查（LLM 不可用）",
                rule.name
            )],
            reasoning: "LLM 服务不可用，返回默认结果".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_analysis_prompt() {
        let rule = ConstitutionalRule {
            id: "test-001".to_string(),
            name: "测试规则".to_string(),
            description: "这是一条测试规则".to_string(),
            phase: "analysis".to_string(),
            severity: "major".to_string(),
            action: "warn".to_string(),
            legal_basis: "专利法第2条".to_string(),
            check: None,
        };

        let (system, user) =
            LlmAnalyzerImpl::build_analysis_prompt(&rule, "测试输入", Some("测试输出"), "analysis");

        assert!(system.contains("测试规则"));
        assert!(system.contains("专利法第2条"));
        assert!(user.contains("测试输入"));
        assert!(user.contains("测试输出"));
    }

    #[test]
    fn test_parse_analysis_response_valid() {
        let raw = r#"{"passed": false, "confidence": 0.85, "details": ["问题1", "问题2"], "reasoning": "分析过程"}"#;
        let result = LlmAnalyzerImpl::parse_analysis_response(raw).unwrap();

        assert!(!result.passed);
        assert!((result.confidence - 0.85).abs() < 0.01);
        assert_eq!(result.details.len(), 2);
        assert_eq!(result.reasoning, "分析过程");
    }

    #[test]
    fn test_parse_analysis_response_with_surrounding_text() {
        let raw = r#"根据分析结果如下：
{"passed": true, "confidence": 0.9, "details": ["通过"], "reasoning": "OK"}
以上是分析结果。"#;
        let result = LlmAnalyzerImpl::parse_analysis_response(raw).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_parse_analysis_response_invalid() {
        let raw = "this is not json at all";
        let result = LlmAnalyzerImpl::parse_analysis_response(raw);
        assert!(result.is_err());
    }

    #[test]
    fn test_noop_analyzer() {
        let rule = ConstitutionalRule {
            id: "test-002".to_string(),
            name: "空操作测试".to_string(),
            description: "测试".to_string(),
            phase: String::new(),
            severity: "minor".to_string(),
            action: "log".to_string(),
            legal_basis: String::new(),
            check: None,
        };

        let analyzer = NoopLlmAnalyzer;
        let result = analyzer.analyze(&rule, "输入", None, "test").unwrap();
        assert!(result.passed);
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn test_from_env_returns_none_without_env() {
        // 确保测试环境中没有设置这些变量
        std::env::remove_var("CONSTITUTIONAL_LLM_BASE_URL");
        std::env::remove_var("OPENAI_BASE_URL");
        std::env::remove_var("OPENAI_API_BASE");
        let result = LlmAnalyzerImpl::from_env(None);
        assert!(result.is_none());
    }
}
