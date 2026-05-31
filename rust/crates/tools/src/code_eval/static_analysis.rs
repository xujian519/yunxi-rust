use crate::code_eval::{quality_level, CodeEvalInput, CodeEvalOutput, CodeIssue};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

pub fn execute(input: &CodeEvalInput) -> Result<Value, String> {
    let issues = analyze_code(&input.code, &input.language);
    let scores = calculate_scores(&issues, &input.code);
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

fn analyze_code(code: &str, language: &str) -> Vec<CodeIssue> {
    match language.to_lowercase().as_str() {
        "rust" => analyze_rust(code),
        "python" => analyze_python(code),
        _ => vec![],
    }
}

fn analyze_rust(code: &str) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    static UNWRAP_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.unwrap\(\)").unwrap());
    static EXPECT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.expect\(").unwrap());
    static PANIC_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\bpanic!\(").unwrap());
    static TODO_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)TODO|FIXME|XXX").unwrap());

    for (line_num, line) in code.lines().enumerate() {
        let line_idx = line_num + 1;

        if UNWRAP_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "错误处理".to_string(),
                severity: "medium".to_string(),
                message: "使用 .unwrap() 可能导致 panic，建议使用 ? 或 match 处理".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if EXPECT_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "错误处理".to_string(),
                severity: "medium".to_string(),
                message: "使用 .expect() 可能导致 panic，建议使用 ? 或 match 处理".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if PANIC_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "错误处理".to_string(),
                severity: "high".to_string(),
                message: "使用 panic!() 会导致程序崩溃，建议使用 Result 或 Option".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if TODO_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码完整性".to_string(),
                severity: "low".to_string(),
                message: "存在未完成的 TODO/FIXME 标记".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if line.len() > 120 {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: format!("行长度超过 120 字符（{} 字符）", line.len()),
                line: Some(line_idx),
                column: None,
            });
        }
    }

    issues
}

fn analyze_python(code: &str) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    static EXCEPT_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\s*except\s*:\s*$").unwrap());
    static GLOBAL_VAR_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[A-Z_][A-Z0-9_]*\s*=").unwrap());

    for (line_num, line) in code.lines().enumerate() {
        let line_idx = line_num + 1;

        if EXCEPT_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "错误处理".to_string(),
                severity: "medium".to_string(),
                message: "空的 except 子句会隐藏错误，建议添加异常处理或日志".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if line.starts_with("global ") {
            issues.push(CodeIssue {
                category: "代码质量".to_string(),
                severity: "medium".to_string(),
                message: "使用全局变量可能导致代码难以维护，建议使用参数传递或类成员".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if line.len() > 100 {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: format!("行长度超过 100 字符（{} 字符）", line.len()),
                line: Some(line_idx),
                column: None,
            });
        }
    }

    issues
}

fn calculate_scores(issues: &[CodeIssue], code: &str) -> Scores {
    let mut correctness: f64 = 100.0;
    let mut efficiency: f64 = 100.0;
    let mut maintainability: f64 = 100.0;
    let mut documentation: f64 = 100.0;

    for issue in issues {
        match issue.severity.as_str() {
            "high" => {
                correctness -= 10.0;
                maintainability -= 10.0;
            }
            "medium" => {
                correctness -= 5.0;
                efficiency -= 3.0;
                maintainability -= 5.0;
            }
            "low" => {
                maintainability -= 2.0;
            }
            _ => {}
        }
    }

    if code.lines().count() > 0 {
        let comment_lines = code
            .lines()
            .filter(|l| l.trim_start().starts_with("//") || l.trim_start().starts_with("#"))
            .count();
        let doc_ratio = comment_lines as f64 / code.lines().count() as f64;
        if doc_ratio < 0.1 {
            documentation -= 30.0;
        } else if doc_ratio < 0.2 {
            documentation -= 15.0;
        }
    }

    correctness = correctness.clamp(0.0, 100.0);
    efficiency = efficiency.clamp(0.0, 100.0);
    maintainability = maintainability.clamp(0.0, 100.0);
    documentation = documentation.clamp(0.0, 100.0);

    let overall =
        (correctness * 0.4 + efficiency * 0.2 + maintainability * 0.3 + documentation * 0.1)
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

    let error_handling_issues = issues.iter().filter(|i| i.category == "错误处理").count();
    if error_handling_issues > 0 {
        suggestions.push(format!(
            "发现 {} 个错误处理问题，建议使用 {} 的错误处理机制（Result/Option, ? 运算符）",
            error_handling_issues, language
        ));
    }

    let style_issues = issues.iter().filter(|i| i.category == "代码风格").count();
    if style_issues > 0 {
        suggestions.push(format!(
            "发现 {} 个代码风格问题，建议使用自动格式化工具",
            style_issues
        ));
    }

    let todo_issues = issues.iter().filter(|i| i.category == "代码完整性").count();
    if todo_issues > 0 {
        suggestions.push(format!(
            "存在 {} 个未完成的标记，请在提交前处理",
            todo_issues
        ));
    }

    suggestions
}
