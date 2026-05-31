use crate::code_eval::{quality_level, CodeEvalOutput, CodeIssue, PythonEvalInput};
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use tempfile::NamedTempFile;

pub fn execute(input: &PythonEvalInput) -> Result<Value, String> {
    if input.test_cases.is_empty() {
        return Err("至少需要一个测试用例".to_string());
    }

    let results = run_python_tests(&input.code, &input.test_cases, input.timeout_ms.max(5000));
    let issues = extract_issues(&results);
    let scores = calculate_scores(&results, input);

    let suggestions = generate_suggestions(&results);

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

struct TestCaseResult {
    input: String,
    expected: String,
    actual: String,
    success: bool,
    execution_time_ms: u64,
}

fn run_python_tests(
    code: &str,
    test_cases: &[crate::code_eval::TestCase],
    _timeout_ms: u64,
) -> Vec<TestCaseResult> {
    let mut results = Vec::new();

    for tc in test_cases {
        let mut temp_file = NamedTempFile::new().unwrap();

        write!(temp_file, "{}", code).unwrap();

        let start = std::time::Instant::now();
        let output = Command::new("python3")
            .arg(temp_file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let elapsed = start.elapsed().as_millis() as u64;

        match output {
            Ok(Output {
                stdout,
                stderr,
                status,
            }) => {
                let actual = String::from_utf8_lossy(&stdout).to_string();
                let error = String::from_utf8_lossy(&stderr).to_string();
                let success = status.success() && actual.trim() == tc.expected_output.trim();

                results.push(TestCaseResult {
                    input: tc.input.clone(),
                    expected: tc.expected_output.clone(),
                    actual: if !error.is_empty() {
                        format!("错误: {}", error)
                    } else {
                        actual
                    },
                    success,
                    execution_time_ms: elapsed,
                });
            }
            Err(e) => {
                results.push(TestCaseResult {
                    input: tc.input.clone(),
                    expected: tc.expected_output.clone(),
                    actual: format!("执行错误: {}", e),
                    success: false,
                    execution_time_ms: elapsed,
                });
            }
        }
    }

    results
}

fn extract_issues(results: &[TestCaseResult]) -> Vec<CodeIssue> {
    let mut issues = Vec::new();

    let failures = results.iter().filter(|r| !r.success).count();
    if failures > 0 {
        issues.push(CodeIssue {
            category: "正确性".to_string(),
            severity: "high".to_string(),
            message: format!("{} / {} 个测试用例失败", failures, results.len()),
            line: None,
            column: None,
        });
    }

    let syntax_errors: Vec<_> = results
        .iter()
        .filter(|r| r.actual.starts_with("错误:"))
        .collect();
    if !syntax_errors.is_empty() {
        issues.push(CodeIssue {
            category: "语法".to_string(),
            severity: "critical".to_string(),
            message: "存在语法错误或运行时错误".to_string(),
            line: None,
            column: None,
        });
    }

    issues
}

struct Scores {
    overall: f64,
    correctness: f64,
    efficiency: f64,
    maintainability: f64,
    documentation: f64,
}

fn calculate_scores(results: &[TestCaseResult], input: &PythonEvalInput) -> Scores {
    let total = results.len();
    if total == 0 {
        return Scores {
            overall: 0.0,
            correctness: 0.0,
            efficiency: 0.0,
            maintainability: 0.0,
            documentation: 0.0,
        };
    }

    let passed = results.iter().filter(|r| r.success).count();
    let correctness: f64 = (passed as f64 / total as f64) * 100.0;

    let total_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();
    let avg_time = total_time as f64 / total as f64;
    let efficiency = if avg_time < 100.0 {
        100.0
    } else if avg_time < 1000.0 {
        100.0 - (avg_time - 100.0) / 9.0
    } else {
        (100.0 - ((avg_time - 1000.0) / 10.0)).clamp(0.0, 90.0)
    };

    let maintainability = if input.test_cases.len() >= 3 {
        80.0
    } else {
        60.0
    };

    let comment_lines = input
        .code
        .lines()
        .filter(|l| l.trim_start().starts_with("#"))
        .count();
    let doc_ratio = if input.code.lines().count() > 0 {
        comment_lines as f64 / input.code.lines().count() as f64
    } else {
        0.0
    };
    let documentation = if doc_ratio >= 0.2 {
        100.0
    } else if doc_ratio >= 0.1 {
        75.0
    } else {
        50.0
    };

    let overall =
        (correctness * 0.5 + efficiency * 0.2 + maintainability * 0.2 + documentation * 0.1)
            .round();

    Scores {
        overall,
        correctness,
        efficiency,
        maintainability,
        documentation,
    }
}

fn generate_suggestions(results: &[TestCaseResult]) -> Vec<String> {
    let mut suggestions = Vec::new();

    let passed = results.iter().filter(|r| r.success).count();
    let total = results.len();
    if passed < total {
        suggestions.push(format!(
            "通过率: {}/{}，建议检查失败用例的输出逻辑",
            passed, total
        ));
    }

    let slow_count = results
        .iter()
        .filter(|r| r.execution_time_ms > 1000)
        .count();
    if slow_count > 0 {
        suggestions.push(format!("{} 个测试用例执行较慢，建议优化算法", slow_count));
    }

    if total < 3 {
        suggestions.push("建议增加更多测试用例覆盖边界条件".to_string());
    }

    suggestions
}
