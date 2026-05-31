use crate::code_eval::{quality_level, CodeEvalOutput, CodeIssue, CodeStyleInput};
use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

pub fn execute(input: &CodeStyleInput) -> Result<Value, String> {
    let issues = analyze_style(&input.code, &input.language);
    let scores = calculate_scores(&issues);
    let suggestions = generate_suggestions(&issues, &input.language);

    let output = CodeEvalOutput {
        overall_score: scores.overall,
        correctness_score: 100.0,
        efficiency_score: 100.0,
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
    maintainability: f64,
    documentation: f64,
}

fn analyze_style(code: &str, language: &str) -> Vec<CodeIssue> {
    match language.to_lowercase().as_str() {
        "rust" => analyze_rust_style(code),
        "python" => analyze_python_style(code),
        _ => vec![],
    }
}

fn analyze_rust_style(code: &str) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    static LONG_LINE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r".{121,}").unwrap());
    static TRAILING_SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" +$").unwrap());
    static TAB_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\t").unwrap());
    static MAGIC_NUMBER_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\b(?:[2-9]|\d{2,})\d*\b").unwrap());

    for (line_num, line) in code.lines().enumerate() {
        let line_idx = line_num + 1;

        if LONG_LINE_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: "行长度超过 120 字符，建议拆分".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if TRAILING_SPACE_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: "行尾有多余空格".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if TAB_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: "使用 Tab 缩进，建议使用 4 个空格".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if MAGIC_NUMBER_RE.is_match(line) && !line.contains("//") {
            issues.push(CodeIssue {
                category: "代码质量".to_string(),
                severity: "low".to_string(),
                message: "使用魔法数字，建议定义为常量".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }
    }

    if !code.contains("fn ") && !code.contains("#[") {
        issues.push(CodeIssue {
            category: "代码结构".to_string(),
            severity: "medium".to_string(),
            message: "未发现函数定义，建议将逻辑封装到函数中".to_string(),
            line: None,
            column: None,
        });
    }

    issues
}

fn analyze_python_style(code: &str) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    static LONG_LINE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r".{101,}").unwrap());
    static TRAILING_SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" +$").unwrap());
    static TAB_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\t").unwrap());
    static MIXED_INDENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[ ]+\t").unwrap());

    for (line_num, line) in code.lines().enumerate() {
        let line_idx = line_num + 1;

        if LONG_LINE_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: "行长度超过 100 字符，建议拆分".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if TRAILING_SPACE_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: "行尾有多余空格".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if TAB_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: "使用 Tab 缩进，PEP8 建议使用 4 个空格".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }

        if MIXED_INDENT_RE.is_match(line) {
            issues.push(CodeIssue {
                category: "代码风格".to_string(),
                severity: "low".to_string(),
                message: "混合使用空格和 Tab 缩进".to_string(),
                line: Some(line_idx),
                column: None,
            });
        }
    }

    if !code.contains("def ") && !code.contains("class ") {
        issues.push(CodeIssue {
            category: "代码结构".to_string(),
            severity: "medium".to_string(),
            message: "未发现函数或类定义，建议将逻辑封装".to_string(),
            line: None,
            column: None,
        });
    }

    issues
}

fn calculate_scores(issues: &[CodeIssue]) -> Scores {
    let mut maintainability: f64 = 100.0;
    let documentation: f64 = 100.0;

    let high_issues = issues.iter().filter(|i| i.severity == "high").count();
    let medium_issues = issues.iter().filter(|i| i.severity == "medium").count();
    let low_issues = issues.iter().filter(|i| i.severity == "low").count();

    maintainability -= high_issues as f64 * 10.0;
    maintainability -= medium_issues as f64 * 5.0;
    maintainability -= low_issues as f64 * 2.0;

    maintainability = maintainability.clamp(0.0, 100.0);

    let overall = maintainability.round();

    Scores {
        overall,
        maintainability,
        documentation,
    }
}

fn generate_suggestions(issues: &[CodeIssue], language: &str) -> Vec<String> {
    let mut suggestions = Vec::new();

    let style_issues: Vec<_> = issues.iter().filter(|i| i.category == "代码风格").collect();
    if !style_issues.is_empty() {
        suggestions.push(format!(
            "发现 {} 个代码风格问题，建议使用自动格式化工具（{}）",
            style_issues.len(),
            if language == "rust" {
                "rustfmt"
            } else if language == "python" {
                "black, isort"
            } else {
                "相应格式化工具"
            }
        ));
    }

    let structure_issues: Vec<_> = issues.iter().filter(|i| i.category == "代码结构").collect();
    if !structure_issues.is_empty() {
        suggestions.push("建议将复杂逻辑封装到函数或类中，提高代码可读性和复用性".to_string());
    }

    suggestions
}
