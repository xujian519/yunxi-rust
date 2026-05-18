// 工具 2: ResponseTemplate - OA 答复模板库

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// 答复模板工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseTemplateInput {
    pub operation: String, // list/filter/render
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>, // argue/amend/both
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

/// 答复模板定义
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResponseTemplate {
    id: String,
    name: String,
    description: String,
    rejection_type: String,
    strategy: String,
    success_rate: f64,
    content: String,
    variables: Vec<String>,
}

/// 内置模板库
fn builtin_templates() -> Vec<ResponseTemplate> {
    serde_json::from_str(include_str!("builtin_templates.json")).expect("builtin templates JSON should be valid")
}

pub fn execute_response_template(input: &ResponseTemplateInput) -> Result<Value, String> {
    let templates = builtin_templates();

    match input.operation.as_str() {
        "list" => {
            let result: Vec<_> = templates
                .into_iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "name": t.name,
                        "description": t.description,
                        "rejection_type": t.rejection_type,
                        "strategy": t.strategy,
                        "success_rate": t.success_rate,
                        "variables": t.variables,
                    })
                })
                .collect();
            Ok(json!({ "templates": result }))
        }
        "filter" => {
            let filtered: Vec<_> = templates
                .into_iter()
                .filter(|t| {
                    if let Some(ref rt) = input.rejection_type {
                        if t.rejection_type != *rt {
                            return false;
                        }
                    }
                    if let Some(ref st) = input.strategy {
                        if t.strategy != *st {
                            return false;
                        }
                    }
                    true
                })
                .map(|t| {
                    json!({
                        "id": t.id,
                        "name": t.name,
                        "description": t.description,
                        "success_rate": t.success_rate,
                    })
                })
                .collect();
            Ok(json!({ "filtered_templates": filtered }))
        }
        "render" => {
            let template_id = input
                .template_id
                .as_ref()
                .ok_or("render 操作需要 template_id 参数")?;
            let template = templates
                .iter()
                .find(|t| t.id == *template_id)
                .ok_or(format!("未找到模板: {template_id}"))?;

            let variables = input
                .variables
                .as_ref()
                .ok_or("render 操作需要 variables 参数")?;
            let mut content = template.content.clone();

            // 替换变量
            let mut missing_vars = Vec::new();
            for var in &template.variables {
                let placeholder = format!("{{{var}}}");
                if let Some(value) = variables.get(var) {
                    content = content.replace(&placeholder, value);
                } else {
                    missing_vars.push(var.clone());
                    content = content.replace(&placeholder, &format!("[[缺失: {var}]]"));
                }
            }

            Ok(json!({
                "template_id": template_id,
                "rendered_content": content,
                "missing_variables": missing_vars,
                "warnings": if missing_vars.is_empty() {
                    vec![]
                } else {
                    vec![format!("缺少 {} 个变量", missing_vars.len())]
                }
            }))
        }
        _ => Err(format!(
            "不支持的操作: {}, 支持: list/filter/render",
            input.operation
        )),
    }
}
