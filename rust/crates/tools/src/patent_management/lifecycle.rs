//! 专利生命周期管理与文档模板库
//!
//! 提供：
//! - 专利生命周期管理（增删改查、状态机、期限管理、费用管理）
//! - 专利文档模板库（审查意见答复、专利申请、无效宣告等）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::LazyLock;

// ==================== 1. PatentManager - 专利生命周期管理 ====================

/// 专利记录
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentRecord {
    pub id: String,
    pub title: String,
    pub applicant: String,
    pub inventor: String,
    pub filing_date: String,
    pub patent_type: String,
    pub status: String,
    pub notes: Option<String>,
}

/// 期限输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeadlineInput {
    pub deadline_type: String,
    pub date: String,
    pub description: Option<String>,
    pub priority: Option<String>,
}

/// 费用输入
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeInput {
    pub fee_type: String,
    pub amount: f64,
    pub due_date: String,
}

/// 专利管理器输入
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentManagerInput {
    pub operation: String,
    pub patent_id: Option<String>,
    pub patent: Option<PatentRecord>,
    pub new_status: Option<String>,
    pub deadline: Option<DeadlineInput>,
    pub fee: Option<FeeInput>,
}

/// 期限记录
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeadlineRecord {
    pub(crate) deadline_type: String,
    pub(crate) date: String,
    pub(crate) description: Option<String>,
    pub(crate) priority: String,
}

/// 费用记录
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeeRecord {
    pub(crate) fee_type: String,
    pub(crate) amount: f64,
    pub(crate) due_date: String,
    pub(crate) paid: bool,
}

/// 专利管理器状态
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PatentState {
    pub(crate) patent: PatentRecord,
    pub(crate) deadlines: Vec<DeadlineRecord>,
    pub(crate) fees: Vec<FeeRecord>,
}

/// 全局专利存储
pub(crate) static PATENT_STORE: LazyLock<std::sync::Mutex<HashMap<String, PatentState>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

/// 专利状态机有效转换映射
pub(crate) fn get_valid_transitions() -> HashMap<&'static str, Vec<&'static str>> {
    [
        ("draft", vec!["filed", "withdrawn"]),
        ("filed", vec!["under_exam", "withdrawn", "abandoned"]),
        (
            "under_exam",
            vec!["oa_issued", "allowed", "rejected", "abandoned"],
        ),
        (
            "oa_issued",
            vec!["amended", "allowed", "rejected", "abandoned"],
        ),
        (
            "amended",
            vec!["under_exam", "allowed", "rejected", "abandoned"],
        ),
        ("allowed", vec!["granted", "abandoned"]),
        ("granted", vec!["expired"]),
        ("rejected", vec![]),  // 终态
        ("abandoned", vec![]), // 终态
        ("withdrawn", vec![]), // 终态
        ("expired", vec![]),   // 终态
    ]
    .into_iter()
    .collect()
}

/// 执行专利管理操作
pub fn execute_patent_manager(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let valid_transitions = get_valid_transitions();

    match input.operation.as_str() {
        "add" => handle_add_patent(input),
        "update" => handle_update_patent(input),
        "remove" => handle_remove_patent(input),
        "get" => handle_get_patent(input),
        "list" => handle_list_patents(),
        "change_status" => handle_change_status(input, &valid_transitions),
        "add_deadline" => handle_add_deadline(input),
        "get_upcoming_deadlines" => handle_get_upcoming_deadlines(input),
        "add_fee" => handle_add_fee(input),
        "get_pending_fees" => handle_get_pending_fees(input),
        "get_portfolio" => handle_get_portfolio(),
        "generate_report" => handle_generate_report(),
        _ => Err(format!("Unknown operation: {}", input.operation)),
    }
}

// ==================== execute_patent_manager 辅助函数 ====================

/// 验证并获取 patent_id
fn validate_patent_id(input: &PatentManagerInput, operation: &str) -> Result<String, String> {
    input
        .patent_id
        .clone()
        .ok_or(format!("patent_id is required for {operation} operation"))
}

/// 验证并获取 patent 数据
fn validate_patent(input: &PatentManagerInput, operation: &str) -> Result<PatentRecord, String> {
    input
        .patent
        .clone()
        .ok_or(format!("patent is required for {operation} operation"))
}

/// 验证状态转换是否有效
fn validate_status_transition(
    valid_transitions: &HashMap<&str, Vec<&str>>,
    current_status: &str,
    new_status: &str,
) -> Result<(), String> {
    let allowed = valid_transitions
        .get(current_status)
        .ok_or("Invalid current status")?;

    if !allowed.contains(&new_status) {
        return Err(format!(
            "Invalid status transition from {current_status} to {new_status}. Allowed: {allowed:?}"
        ));
    }
    Ok(())
}

/// 添加专利操作
fn handle_add_patent(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let patent = validate_patent(&input, "add")?;
    let id = patent.id.clone();
    let state = PatentState {
        patent,
        deadlines: Vec::new(),
        fees: Vec::new(),
    };
    let mut store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    if store.contains_key(&id) {
        return Err(format!("Patent with id {id} already exists"));
    }
    store.insert(id.clone(), state);
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Patent {} added successfully", id),
        "patent_id": id
    }))
}

/// 更新专利操作
fn handle_update_patent(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let patent = validate_patent(&input, "update")?;
    let id = patent.id.clone();
    let mut store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let state = store.get_mut(&id).ok_or("Patent not found")?;
    state.patent = patent;
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Patent {} updated successfully", id),
        "patent_id": id
    }))
}

/// 删除专利操作
fn handle_remove_patent(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let id = validate_patent_id(&input, "remove")?;
    let mut store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    store.remove(&id).ok_or("Patent not found")?;
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Patent {} removed successfully", id),
        "patent_id": id
    }))
}

/// 获取专利操作
fn handle_get_patent(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let id = validate_patent_id(&input, "get")?;
    let store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let state = store.get(&id).ok_or("Patent not found")?;
    serde_json::to_value(state).map_err(|e| e.to_string())
}

/// 列出所有专利操作
fn handle_list_patents() -> Result<serde_json::Value, String> {
    let store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let patents: Vec<&PatentRecord> = store.values().map(|s| &s.patent).collect();
    Ok(serde_json::json!({
        "total": patents.len(),
        "patents": patents
    }))
}

/// 更改专利状态操作
fn handle_change_status(
    input: PatentManagerInput,
    valid_transitions: &HashMap<&str, Vec<&str>>,
) -> Result<serde_json::Value, String> {
    let id = validate_patent_id(&input, "change_status")?;
    let new_status = input
        .new_status
        .ok_or("new_status is required for change_status operation")?;
    let mut store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let state = store.get_mut(&id).ok_or("Patent not found")?;
    let current_status = state.patent.status.clone();

    validate_status_transition(valid_transitions, &current_status, &new_status)?;

    state.patent.status.clone_from(&new_status);
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Patent {} status changed from {} to {}", id, current_status, new_status),
        "patent_id": id,
        "old_status": current_status,
        "new_status": new_status
    }))
}

/// 添加期限操作
fn handle_add_deadline(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let id = validate_patent_id(&input, "add_deadline")?;
    let deadline_input = input
        .deadline
        .ok_or("deadline is required for add_deadline operation")?;
    let mut store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let state = store.get_mut(&id).ok_or("Patent not found")?;

    let deadline = DeadlineRecord {
        deadline_type: deadline_input.deadline_type,
        date: deadline_input.date,
        description: deadline_input.description,
        priority: deadline_input
            .priority
            .unwrap_or_else(|| String::from("medium")),
    };

    state.deadlines.push(deadline);
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Deadline added to patent {}", id),
        "patent_id": id,
        "total_deadlines": state.deadlines.len()
    }))
}

/// 获取即将到来的期限操作
fn handle_get_upcoming_deadlines(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let id = validate_patent_id(&input, "get_upcoming_deadlines")?;
    let store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let state = store.get(&id).ok_or("Patent not found")?;

    let mut deadlines = state.deadlines.clone();
    deadlines.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(serde_json::json!({
        "patent_id": id,
        "deadlines": deadlines,
        "total": deadlines.len()
    }))
}

/// 添加费用操作
fn handle_add_fee(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let id = validate_patent_id(&input, "add_fee")?;
    let fee_input = input.fee.ok_or("fee is required for add_fee operation")?;
    let mut store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let state = store.get_mut(&id).ok_or("Patent not found")?;

    let fee = FeeRecord {
        fee_type: fee_input.fee_type,
        amount: fee_input.amount,
        due_date: fee_input.due_date,
        paid: false,
    };

    state.fees.push(fee);
    Ok(serde_json::json!({
        "success": true,
        "message": format!("Fee added to patent {}", id),
        "patent_id": id,
        "total_fees": state.fees.len()
    }))
}

/// 获取待支付费用操作
fn handle_get_pending_fees(input: PatentManagerInput) -> Result<serde_json::Value, String> {
    let id = validate_patent_id(&input, "get_pending_fees")?;
    let store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let state = store.get(&id).ok_or("Patent not found")?;

    let pending: Vec<&FeeRecord> = state.fees.iter().filter(|f| !f.paid).collect();
    Ok(serde_json::json!({
        "patent_id": id,
        "pending_fees": pending,
        "total_pending": pending.len()
    }))
}

/// 获取专利组合操作
fn handle_get_portfolio() -> Result<serde_json::Value, String> {
    let store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let mut patents: Vec<&PatentRecord> = store.values().map(|s| &s.patent).collect();

    patents.sort_by(|a, b| a.filing_date.cmp(&b.filing_date));

    let status_counts: std::collections::HashMap<&str, usize> =
        patents
            .iter()
            .fold(std::collections::HashMap::new(), |mut acc, p| {
                *acc.entry(p.status.as_str()).or_insert(0) += 1;
                acc
            });

    Ok(serde_json::json!({
        "total_patents": patents.len(),
        "patents": patents,
        "status_breakdown": status_counts
    }))
}

/// 生成报告操作
fn handle_generate_report() -> Result<serde_json::Value, String> {
    let store = PATENT_STORE
        .lock()
        .map_err(|e| format!("Failed to acquire patent store lock: {e}"))?;
    let total_patents = store.len();
    let total_deadlines: usize = store.values().map(|s| s.deadlines.len()).sum();
    let total_fees: usize = store.values().map(|s| s.fees.len()).sum();
    let pending_fees: usize = store
        .values()
        .map(|s| s.fees.iter().filter(|f| !f.paid).count())
        .sum();

    Ok(serde_json::json!({
        "report_type": "portfolio_summary",
        "total_patents": total_patents,
        "total_deadlines": total_deadlines,
        "total_fees": total_fees,
        "pending_fees": pending_fees,
        "generated_at": chrono_timestamp()
    }))
}

// ==================== 2. TemplateLibrary - 专利文档模板库 ====================

/// 模板库输入
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateLibraryInput {
    pub operation: String,
    pub template_type: Option<String>,
    pub template_id: Option<String>,
    pub variables: Option<HashMap<String, String>>,
}

/// 模板定义
#[derive(Debug, Serialize, Deserialize)]
struct Template {
    id: String,
    name: String,
    description: String,
    required_vars: Vec<String>,
    content: String,
}

/// 内置模板
fn get_builtin_templates() -> Vec<Template> {
    serde_json::from_str(include_str!("builtin_templates.json"))
        .expect("builtin templates JSON should be valid")
}

/// 执行模板库操作
pub fn execute_template_library(input: TemplateLibraryInput) -> Result<serde_json::Value, String> {
    let templates = get_builtin_templates();

    match input.operation.as_str() {
        "list" => {
            let template_list: Vec<serde_json::Value> = templates
                .iter()
                .map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "description": t.description,
                        "required_vars": t.required_vars
                    })
                })
                .collect();

            Ok(serde_json::json!({
                "templates": template_list,
                "total": template_list.len()
            }))
        }

        "load" => {
            let template_id = input
                .template_id
                .or(input.template_type)
                .ok_or("template_id or template_type is required for load operation")?;

            let template = templates
                .iter()
                .find(|t| t.id == template_id)
                .ok_or("Template not found")?;

            Ok(serde_json::json!({
                "id": template.id,
                "name": template.name,
                "description": template.description,
                "required_vars": template.required_vars,
                "content": template.content
            }))
        }

        "render" => {
            let template_id = input
                .template_id
                .or(input.template_type)
                .ok_or("template_id or template_type is required for render operation")?;

            let variables = input
                .variables
                .ok_or("variables is required for render operation")?;

            let template = templates
                .iter()
                .find(|t| t.id == template_id)
                .ok_or("Template not found")?;

            // 检查必填变量
            let mut missing_vars = Vec::new();
            for var in &template.required_vars {
                if !variables.contains_key(var) {
                    missing_vars.push(var.to_string());
                }
            }

            if !missing_vars.is_empty() {
                return Err(format!(
                    "Missing required variables: {}",
                    missing_vars.join(", ")
                ));
            }

            // 渲染模板
            let mut rendered = template.content.clone();
            let mut unfilled_vars = Vec::new();

            for (key, value) in &variables {
                let placeholder = format!("{{{{{key}}}}}");
                rendered = rendered.replace(&placeholder, value);
            }

            // 检测未填充的变量
            #[allow(clippy::items_after_statements)]
            static RE_UNFILLED: LazyLock<regex::Regex> =
                LazyLock::new(|| regex::Regex::new(r"\{\{(\w+)\}\}").unwrap());
            for cap in RE_UNFILLED.captures_iter(&rendered) {
                let var_name = cap
                    .get(1)
                    .expect("capture group 1 always exists for this regex")
                    .as_str();
                if !unfilled_vars.contains(&var_name.to_string()) {
                    unfilled_vars.push(var_name.to_string());
                }
            }

            Ok(serde_json::json!({
                "template_id": template_id,
                "rendered": rendered,
                "unfilled_vars": unfilled_vars,
                "warnings": if unfilled_vars.is_empty() { None } else { Some(format!("Unfilled variables: {}", unfilled_vars.join(", "))) }
            }))
        }

        _ => Err(format!("Unknown operation: {}", input.operation)),
    }
}

// ==================== 辅助函数 ====================

/// 获取当前时间戳（ISO 8601格式）
pub(crate) fn chrono_timestamp() -> String {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};
    if let Ok(output) = Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    // 回退到Unix时间戳
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
