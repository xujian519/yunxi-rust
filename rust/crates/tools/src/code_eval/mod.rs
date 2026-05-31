pub mod agent_code;
pub mod execution;
pub mod python_eval;
pub mod static_analysis;
pub mod style;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn execute_code_static_analysis(input: &CodeEvalInput) -> Result<Value, String> {
    static_analysis::execute(input)
}

pub fn execute_code_execution_test(input: &CodeExecutionInput) -> Result<Value, String> {
    execution::execute(input)
}

pub fn execute_python_eval(input: &PythonEvalInput) -> Result<Value, String> {
    python_eval::execute(input)
}

pub fn execute_code_style_eval(input: &CodeStyleInput) -> Result<Value, String> {
    style::execute(input)
}

pub fn execute_agent_code_eval(input: &AgentCodeEvalInput) -> Result<Value, String> {
    agent_code::execute(input)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEvalInput {
    pub code: String,
    pub language: String,
    #[serde(default)]
    pub filename: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeExecutionInput {
    pub code: String,
    pub language: String,
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
    #[serde(default)]
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub input: String,
    pub expected_output: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonEvalInput {
    pub code: String,
    #[serde(default)]
    pub test_cases: Vec<TestCase>,
    #[serde(default)]
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeStyleInput {
    pub code: String,
    pub language: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCodeEvalInput {
    pub code: String,
    pub task_description: String,
    pub language: String,
    #[serde(default)]
    pub reference_solution: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeEvalOutput {
    pub overall_score: f64,
    pub correctness_score: f64,
    pub efficiency_score: f64,
    pub maintainability_score: f64,
    pub documentation_score: f64,
    pub issues: Vec<CodeIssue>,
    pub suggestions: Vec<String>,
    pub quality_level: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeIssue {
    pub category: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
}

pub fn quality_level(score: f64) -> &'static str {
    if score >= 90.0 {
        "excellent"
    } else if score >= 75.0 {
        "good"
    } else if score >= 60.0 {
        "fair"
    } else {
        "poor"
    }
}
