//! 代码执行桥接 — Code-First 规划步骤。
//!
//! 参考 smolagents CodeAgent，对于规则明确的步骤（如专利形式检查、
//! 数值比较、格式校验），直接用代码执行而非 LLM 规划。
//!
//! `CodeExecutor` trait 由上层注入，支持沙箱执行（Docker/WebAssembly/本地）。

use serde::{Deserialize, Serialize};

/// 代码执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub language: String,
}

/// 代码执行器 trait。
///
/// 由上层注入真实执行能力（内置沙箱或远程执行）。
pub trait CodeExecutor: Send {
    /// 执行代码片段。
    fn execute(&mut self, language: &str, code: &str) -> Result<CodeExecutionResult, String>;

    /// 执行器名称。
    fn name(&self) -> &str;

    /// 是否支持该语言。
    fn supports(&self, language: &str) -> bool;
}

/// 沙箱代码执行策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStrategy {
    /// 直接在本地进程执行（无隔离，仅限受信代码）
    Local,
    /// 使用子进程隔离
    Subprocess,
    /// 使用 Docker 容器隔离
    Docker,
    /// 使用 WebAssembly 沙箱隔离
    Wasm,
}

/// 无操作代码执行器（测试用）。
pub struct NoopCodeExecutor {
    pub label: String,
}

impl CodeExecutor for NoopCodeExecutor {
    fn execute(&mut self, language: &str, code: &str) -> Result<CodeExecutionResult, String> {
        Ok(CodeExecutionResult {
            success: true,
            output: format!("[NOOP] 代码已执行: lang={language}, 长度={}", code.len()),
            error: None,
            language: language.to_string(),
        })
    }

    fn name(&self) -> &str {
        &self.label
    }

    fn supports(&self, _language: &str) -> bool {
        true
    }
}

/// 内置专利规则检查器 — 用于 Code-First 执行示例。
///
/// 对于专利形式检查、权利要求计数等规则明确的任务，
/// 使用硬编码 Rust 逻辑取代 LLM 生成代码。
pub struct BuiltinPatentChecker;

impl BuiltinPatentChecker {
    /// 检查权利要求数量是否符合规范。
    pub fn check_claim_count(claims_json: &str) -> Result<CodeExecutionResult, String> {
        let claims: Vec<serde_json::Value> = serde_json::from_str(claims_json).unwrap_or_default();
        let count = claims.len();

        let passed = (1..=10).contains(&count);
        let message = if passed {
            format!("权利要求数量 {count} 在 1-10 的规范范围内")
        } else {
            format!("权利要求数量 {count} 不在 1-10 的规范范围内")
        };

        Ok(CodeExecutionResult {
            success: passed,
            output: message.clone(),
            error: if passed { None } else { Some(message) },
            language: "builtin".into(),
        })
    }

    /// 检查标题/摘要字数。
    pub fn check_word_count(text: &str, max_words: usize, label: &str) -> CodeExecutionResult {
        let words: Vec<&str> = text.split_whitespace().collect();
        let passed = words.len() <= max_words;
        CodeExecutionResult {
            success: passed,
            output: format!("{label} 字数 {} / 最大 {}", words.len(), max_words),
            error: if passed {
                None
            } else {
                Some(format!("{label} 超限: {} > {}", words.len(), max_words))
            },
            language: "builtin".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_code_executor() {
        let mut exec = NoopCodeExecutor {
            label: "test".into(),
        };
        let result = exec.execute("python", "print('hello')").unwrap();
        assert!(result.success);
        assert!(result.output.contains("NOOP"));
        assert!(exec.supports("rust"));
    }

    #[test]
    fn test_claim_count_check_pass() {
        let claims = r#"[{"num":1},{"num":2},{"num":3}]"#;
        let result = BuiltinPatentChecker::check_claim_count(claims).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_claim_count_check_fail() {
        let claims = r#"[{"num":1},{"num":2},{"num":3},{"num":4},{"num":5},{"num":6},{"num":7},{"num":8},{"num":9},{"num":10},{"num":11}]"#;
        let result = BuiltinPatentChecker::check_claim_count(claims).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_word_count_check() {
        let text = "本发明涉及一种基于深度学习的专利检索方法";
        let result = BuiltinPatentChecker::check_word_count(text, 50, "摘要");
        assert!(result.success);

        let long_text = "word ".repeat(100);
        let result = BuiltinPatentChecker::check_word_count(&long_text, 50, "标题");
        assert!(!result.success);
    }
}
