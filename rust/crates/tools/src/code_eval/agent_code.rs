use crate::code_eval::{quality_level, AgentCodeEvalInput, CodeEvalOutput, CodeIssue};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

pub fn execute(input: &AgentCodeEvalInput) -> Result<Value, String> {
    let issues = analyze_agent_code(&input.code, &input.language, &input.task_description);
    let scores = calculate_scores(&issues, &input.code, &input.task_description);
    let suggestions = generate_suggestions(&issues, &input.language);

    let output = CodeEvalOutput {
        overall_score: scores.overall,
        correctness_score: scores.correctness,
        efficiency_score: scores.efficiency,
        maintainability_score: scores.maintainability,
        documentation_score: scores.documentation,
        issues,
        suggestions,
        quality_level: quality_level(scores.overall).to_string(),
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

struct Scores {
    overall: f64,
    correctness: f64,
    efficiency: f64,
    maintainability: f64,
    documentation: f64,
}

fn analyze_agent_code(code: &str, language: &str, task: &str) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    if code.is_empty() {
        issues.push(CodeIssue {
            category: "代码完整性".to_string(),
            severity: "critical".to_string(),
            message: "生成的代码为空".to_string(),
            line: None,
            column: None,
        });
        return issues;
    }

    match language.to_lowercase().as_str() {
        "rust" => issues.extend(analyze_rust_agent_code(code, task)),
        "python" => issues.extend(analyze_python_agent_code(code, task)),
        _ => {}
    }

    issues
}

fn analyze_rust_agent_code(code: &str, _task: &str) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    static UNWRAP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.unwrap\(\)").unwrap());
    static PANIC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bpanic!\(").unwrap());

    for (line_num, line) in code.lines().enumerate() {
        let line_idx = line_num + 1;

        if UNWRAP_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码健壮性".to_string(),
                severity: "medium".to_string(),
                message: "使用 .unwrap() 可能导致运行时 panic".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if PANIC_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码健壮性".to_string(),
                severity: "high".to_string(),
                message: "使用 panic!() 不适合生产环境代码".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }
    }

    if !code.contains("fn ") && !code.contains("#[") {
        issues.push(CodeIssue {
            category: "代码结构".to_string(),
            severity: "medium".to_string(),
            message: "未发现函数定义，代码结构不清晰".to_string(),
            line: None,
            column: None,
        });
    }

    if !code.contains("//") && !code.contains("///") {
        issues.push(CodeIssue {
            category: "文档".to_string(),
            severity: "low".to_string(),
            message: "缺少代码注释，建议添加关键逻辑说明".to_string(),
            line: None,
            column: None,
        });
    }

    issues
}

fn analyze_python_agent_code(code: &str, _task: &str) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    static EXCEPT_BARE_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\s*except\s*:\s*$").unwrap());

    for (line_num, line) in code.lines().enumerate() {
        let line_idx = line_num + 1;

        if EXCEPT_BARE_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码健壮性".to_string(),
                severity: "medium".to_string(),
                message: "空的 except 子句会隐藏错误".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }
    }

    if !code.contains("def ") && !code.contains("class ") {
        issues.push(CodeIssue {
            category: "代码结构".to_string(),
            severity: "medium".to_string(),
            message: "未发现函数或类定义，代码结构不清晰".to_string(),
            line: None,
            column: None,
        });
    }

    if !code.contains("#") {
        issues.push(CodeIssue {
            category: "文档".to_string(),
            severity: "low".to_string(),
            message: "缺少代码注释，建议添加关键逻辑说明".to_string(),
            line: None,
            column: None,
        });
    }

    issues
}

fn calculate_scores(issues: &[CodeIssue], code: &str, _task: &str) -> Scores {
    let mut correctness: f64 = 100.0;
    let mut efficiency: f64 = 100.0;
    let mut maintainability: f64 = 100.0;
    let mut documentation: f64 = 100.0;

    for issue in issues {
        match issue.severity.as_str() {
            "critical" => {
                correctness -= 30.0;
                maintainability -= 20.0;
            }
            "high" => {
                correctness -= 15.0;
                maintainability -= 10.0;
            }
            "medium" => {
                correctness -= 5.0;
                maintainability -= 5.0;
            }
            "low" => {
                maintainability -= 2.0;
            }
            _ => {}
        }
    }

    if code.lines().count() > 0 {
        let comment_ratio = if code.contains("//") || code.contains("#") || code.contains("///") {
            code.lines()
                .filter(|l| {
                    l.trim().starts_with("//")
                        || l.trim().starts_with("#")
                        || l.trim().starts_with("///")
                })
                .count() as f64
                / code.lines().count() as f64
        } else {
            0.0
        };

        if comment_ratio < 0.05 {
            documentation -= 40.0;
        } else if comment_ratio < 0.1 {
            documentation -= 20.0;
        }
    }

    if code.lines().count() < 5 {
        maintainability -= 20.0;
    }

    correctness = correctness.clamp(0.0, 100.0);
    efficiency = efficiency.clamp(0.0, 100.0);
    maintainability = maintainability.clamp(0.0, 100.0);
    documentation = documentation.clamp(0.0, 100.0);

    let overall =
        (correctness * 0.4 + efficiency * 0.2 + maintainability * 0.25 + documentation * 0.15)
            .round();

    Scores {
        overall,
        correctness,
        efficiency,
        maintainability,
        documentation,
    }
}

fn generate_suggestions(issues: &[CodeIssue], language: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    let robustness_issues: Vec<_> = issues
        .iter()
        .filter(|i| i.category == "代码健壮性")
        .collect();
    if !robustness_issues.is_empty() {
        suggestions.push(format!(
            "发现 {} 个健壮性问题，建议使用适当的错误处理机制（{}）",
            robustness_issues.len(),
            if language == "rust" {
                "Result, Option, ? 运算符"
            } else if language == "python" {
                "try-except, 具体异常类型"
            } else {
                "语言特定错误处理"
            }
        ));
    }

    let structure_issues: Vec<_> = issues.iter().filter(|i| i.category == "代码结构").collect();
    if !structure_issues.is_empty() {
        suggestions.push("建议将代码逻辑封装到函数或类中，提高可读性和可复用性".to_string());
    }

    let doc_issues: Vec<_> = issues.iter().filter(|i| i.category == "文档").collect();
    if !doc_issues.is_empty() {
        suggestions.push("建议添加代码注释，说明关键逻辑和算法思路".to_string());
    }

    suggestions
}
