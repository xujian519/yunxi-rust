use crate::code_eval::{quality_level, CodeEvalOutput, CodeExecutionInput, CodeIssue};
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use tempfile::tempdir;
use tempfile::NamedTempFile;

pub fn execute(input: &CodeExecutionInput) -> Result<Value, String> {
    if input.test_cases.is_empty() {
        return Err("至少需要一个测试用例".to_string());
    }

    let results = run_test_cases(input);
    let issues = extract_issues(&results);
    let scores = calculate_scores(&results, input);

    let suggestions = generate_suggestions(&results);

    let output = CodeEvalOutput {
        overall_score: scores.overall,
        correctness_score: scores.correctness,
        efficiency_score: scores.efficiency,
        maintainability_score: scores.maintainability,
        documentation_score: 100.0,
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

fn run_test_cases(input: &CodeExecutionInput) -> Vec<TestCaseResult> {
    let mut results = Vec::new();

    let timeout = input.timeout_ms.max(5000);

    match input.language.to_lowercase().as_str() {
        "rust" => {
            if let Ok(test_results) = run_rust_tests(&input.code, &input.test_cases, timeout) {
                results.extend(test_results);
            }
        }
        "python" => {
            if let Ok(test_results) = run_python_tests(&input.code, &input.test_cases, timeout) {
                results.extend(test_results);
            }
        }
        _ => {
            for tc in &input.test_cases {
                results.push(TestCaseResult {
                    input: tc.input.clone(),
                    expected: tc.expected_output.clone(),
                    actual: "不支持的语言".to_string(),
                    success: false,
                    execution_time_ms: 0,
                });
            }
        }
    }

    results
}

fn run_rust_tests(
    code: &str,
    test_cases: &[crate::code_eval::TestCase],
    _timeout_ms: u64,
) -> Result<Vec<TestCaseResult>, String> {
    let dir = tempdir().map_err(|e| e.to_string())?;
    let project_dir = dir.path();

    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;

    let main_rs = src_dir.join("main.rs");
    std::fs::write(&main_rs, code).map_err(|e| e.to_string())?;

    let cargo_toml = project_dir.join("Cargo.toml");
    std::fs::write(
        &cargo_toml,
        r#"
[package]
name = "code_eval"
version = "0.1.0"
edition = "2021"

[profile.dev]
opt-level = 0

[dependencies]
"#,
    )
    .map_err(|e| e.to_string())?;

    let mut results = Vec::new();

    for tc in test_cases {
        let start = std::time::Instant::now();
        let output = Command::new("cargo")
            .args(["run", "--quiet"])
            .current_dir(project_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        let elapsed = start.elapsed().as_millis() as u64;

        match output {
            Ok(Output { stdout, .. }) => {
                let actual = String::from_utf8_lossy(&stdout).to_string();
                let success = actual.trim() == tc.expected_output.trim();
                results.push(TestCaseResult {
                    input: tc.input.clone(),
                    expected: tc.expected_output.clone(),
                    actual,
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

    Ok(results)
}

fn run_python_tests(
    code: &str,
    test_cases: &[crate::code_eval::TestCase],
    _timeout_ms: u64,
) -> Result<Vec<TestCaseResult>, String> {
    let mut results = Vec::new();

    for tc in test_cases {
        let mut temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;

        write!(temp_file, "{}", code).map_err(|e| e.to_string())?;

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

                if !error.is_empty() {
                    results.push(TestCaseResult {
                        input: tc.input.clone(),
                        expected: tc.expected_output.clone(),
                        actual: format!("错误: {}", error),
                        success: false,
                        execution_time_ms: elapsed,
                    });
                } else {
                    results.push(TestCaseResult {
                        input: tc.input.clone(),
                        expected: tc.expected_output.clone(),
                        actual,
                        success,
                        execution_time_ms: elapsed,
                    });
                }
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

    Ok(results)
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

    let slow_tests: Vec<_> = results
        .iter()
        .filter(|r| r.execution_time_ms > 1000)
        .collect();
    if !slow_tests.is_empty() {
        issues.push(CodeIssue {
            category: "性能".to_string(),
            severity: "medium".to_string(),
            message: format!("{} 个测试用例执行时间超过 1 秒", slow_tests.len()),
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
}

fn calculate_scores(results: &[TestCaseResult], input: &CodeExecutionInput) -> Scores {
    let total = results.len();
    if total == 0 {
        return Scores {
            overall: 0.0,
            correctness: 0.0,
            efficiency: 0.0,
            maintainability: 0.0,
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

    let overall = (correctness * 0.6 + efficiency * 0.2 + maintainability * 0.2).round();

    Scores {
        overall,
        correctness,
        efficiency,
        maintainability,
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
        suggestions.push(format!(
            "{} 个测试用例执行较慢，建议优化算法或减少循环次数",
            slow_count
        ));
    }

    if total < 3 {
        suggestions.push("测试覆盖率不足，建议增加边界条件和异常情况的测试用例".to_string());
    }

    suggestions
}
