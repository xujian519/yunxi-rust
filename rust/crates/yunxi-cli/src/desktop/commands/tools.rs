//! 专利工具 IPC（供前端 slash 命令等直接调用）。

use intent::IntentClassifier;
use memory::unified::UnifiedMemory;
use serde_json::{json, Value};

use tools::execute_tool;

#[tauri::command]
pub fn claim_parse(claims: String) -> Result<String, String> {
    execute_tool(
        "ClaimParse",
        &json!({
            "claims": claims,
        }),
    )
}

/// 通用工具执行命令 - 允许前端直接调用任意后端工具
#[tauri::command]
pub fn execute_tool_raw(tool_name: String, tool_input: Value) -> Result<String, String> {
    execute_tool(&tool_name, &tool_input)
}

#[tauri::command]
pub fn patent_search(query: String, limit: Option<usize>) -> Result<String, String> {
    execute_tool(
        "PatentSearch",
        &json!({
            "query": query,
            "limit": limit.unwrap_or(10),
        }),
    )
}

#[tauri::command]
pub fn patent_compare(
    target_title: String,
    target_claims: Vec<String>,
    prior_title: String,
    prior_claims: Vec<String>,
) -> Result<String, String> {
    execute_tool(
        "PatentCompare",
        &json!({
            "mode": "diff",
            "target": {
                "title": target_title,
                "claims": target_claims,
            },
            "priorArt": {
                "title": prior_title,
                "claims": prior_claims,
            },
        }),
    )
}

#[tauri::command]
pub fn knowledge_search(query: String) -> Result<String, String> {
    execute_tool(
        "KnowledgeSearch",
        &json!({
            "query": query,
            "limit": 8,
        }),
    )
}

#[tauri::command]
pub fn memory_search(query: String, limit: Option<usize>) -> Result<String, String> {
    yunxi_cli::memory_bridge::search_memory_report(&query, limit.unwrap_or(10))
}

#[tauri::command]
pub fn oa_parse(content: String, application_number: Option<String>) -> Result<String, String> {
    execute_tool(
        "OaParse",
        &json!({
            "content": content,
            "application_number": application_number,
            "document_type": "cn",
        }),
    )
}

#[tauri::command]
pub fn novelty_analysis(
    claims: String,
    prior_art: String,
    analysis_mode: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "NoveltyAnalysis",
        &json!({
            "claims": claims,
            "prior_art": prior_art,
            "analysis_mode": analysis_mode.unwrap_or_else(|| "standard".to_string()),
        }),
    )
}

#[tauri::command]
pub fn inventiveness_analysis(
    claims: String,
    prior_art: String,
    technical_field: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "InventivenessAnalysis",
        &json!({
            "claims": claims,
            "prior_art": prior_art,
            "technical_field": technical_field,
        }),
    )
}

#[tauri::command]
pub fn claim_generator(
    technical_features: String,
    claim_type: Option<String>,
    scope: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "ClaimGenerator",
        &json!({
            "technical_features": technical_features,
            "claim_type": claim_type.unwrap_or_else(|| "independent".to_string()),
            "scope": scope,
        }),
    )
}

#[tauri::command]
pub fn abstract_drafter(
    invention_title: String,
    technical_field: String,
    technical_problem: String,
    technical_solution: String,
    beneficial_effects: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "AbstractDrafter",
        &json!({
            "invention_title": invention_title,
            "technical_field": technical_field,
            "technical_problem": technical_problem,
            "technical_solution": technical_solution,
            "beneficial_effects": beneficial_effects,
        }),
    )
}

#[tauri::command]
pub fn specification_drafter(
    claims: String,
    abstract_text: String,
    technical_field: String,
    background: Option<String>,
    detailed_description: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "SpecificationDrafter",
        &json!({
            "claims": claims,
            "abstract": abstract_text,
            "technical_field": technical_field,
            "background": background,
            "detailed_description": detailed_description,
        }),
    )
}

#[tauri::command]
pub fn quality_scorer(
    claims: String,
    abstract_text: Option<String>,
    specification: Option<String>,
    score_mode: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "QualityScorer",
        &json!({
            "claims": claims,
            "abstract": abstract_text,
            "specification": specification,
            "score_mode": score_mode.unwrap_or_else(|| "comprehensive".to_string()),
        }),
    )
}

#[tauri::command]
pub fn quality_checker(
    claims: String,
    abstract_text: Option<String>,
    specification: Option<String>,
    check_mode: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "QualityChecker",
        &json!({
            "claims": claims,
            "abstract": abstract_text,
            "specification": specification,
            "check_mode": check_mode.unwrap_or_else(|| "comprehensive".to_string()),
        }),
    )
}

#[tauri::command]
pub fn formal_check(
    patent_text: String,
    check_type: Option<String>,
    jurisdiction: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "FormalCheck",
        &json!({
            "patent_text": patent_text,
            "check_type": check_type.unwrap_or_else(|| "comprehensive".to_string()),
            "jurisdiction": jurisdiction.unwrap_or_else(|| "cn".to_string()),
        }),
    )
}

#[tauri::command]
pub fn claim_formality_check(
    claims: String,
    jurisdiction: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "ClaimFormalityCheck",
        &json!({
            "claims": claims,
            "jurisdiction": jurisdiction.unwrap_or_else(|| "cn".to_string()),
        }),
    )
}

#[tauri::command]
pub fn spec_formality_check(
    specification: String,
    jurisdiction: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "SpecFormalityCheck",
        &json!({
            "specification": specification,
            "jurisdiction": jurisdiction.unwrap_or_else(|| "cn".to_string()),
        }),
    )
}

#[tauri::command]
pub fn oa_strategy(
    claims: String,
    prior_art: String,
    rejection_reasons: String,
    jurisdiction: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "OaStrategy",
        &json!({
            "claims": claims,
            "prior_art": prior_art,
            "rejection_reasons": rejection_reasons,
            "jurisdiction": jurisdiction.unwrap_or_else(|| "cn".to_string()),
        }),
    )
}

#[tauri::command]
pub fn response_template(
    oa_type: String,
    rejection_reasons: String,
    strategy: Option<String>,
    jurisdiction: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "ResponseTemplate",
        &json!({
            "oa_type": oa_type,
            "rejection_reasons": rejection_reasons,
            "strategy": strategy,
            "jurisdiction": jurisdiction.unwrap_or_else(|| "cn".to_string()),
        }),
    )
}

#[tauri::command]
pub fn success_predictor(
    claims: String,
    prior_art: String,
    technical_field: String,
    jurisdiction: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "SuccessPredictor",
        &json!({
            "claims": claims,
            "prior_art": prior_art,
            "technical_field": technical_field,
            "jurisdiction": jurisdiction.unwrap_or_else(|| "cn".to_string()),
        }),
    )
}

#[tauri::command]
pub fn infringement_analysis(
    patent_claims: String,
    product_features: String,
    analysis_mode: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "InfringementAnalysis",
        &json!({
            "patent_claims": patent_claims,
            "product_features": product_features,
            "analysis_mode": analysis_mode.unwrap_or_else(|| "standard".to_string()),
        }),
    )
}

#[tauri::command]
pub fn legal_reasoning(
    legal_question: String,
    jurisdiction: Option<String>,
    context: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "LegalReasoning",
        &json!({
            "legal_question": legal_question,
            "jurisdiction": jurisdiction.unwrap_or_else(|| "cn".to_string()),
            "context": context,
        }),
    )
}

#[tauri::command]
pub fn examiner_simulate(
    claims: String,
    prior_art: String,
    technical_field: String,
    simulate_mode: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "ExaminerSimulate",
        &json!({
            "claims": claims,
            "prior_art": prior_art,
            "technical_field": technical_field,
            "simulate_mode": simulate_mode.unwrap_or_else(|| "strict".to_string()),
        }),
    )
}

#[tauri::command]
pub fn hybrid_retrieval(
    query: String,
    vector_weight: Option<f64>,
    graph_weight: Option<f64>,
    legal_weight: Option<f64>,
    top_k: Option<usize>,
) -> Result<String, String> {
    execute_tool(
        "HybridRetrieval",
        &json!({
            "query": query,
            "vector_weight": vector_weight,
            "graph_weight": graph_weight,
            "legal_weight": legal_weight,
            "top_k": top_k,
        }),
    )
}
/// 记录用户对某意图的偏好（用户修正分类时调用）。
#[tauri::command]
pub fn record_intent_preference(intent_type: String) -> Result<String, String> {
    let intent: intent::IntentType = serde_json::from_value(Value::String(intent_type.clone()))
        .map_err(|_| format!("未知意图类型: {intent_type}"))?;
    let memory = UnifiedMemory::default_paths()?;
    IntentClassifier::record_user_preference(&memory, intent);
    Ok(format!("已记录意图偏好: {intent_type}"))
}
/// 法律条款查询。
#[tauri::command]
pub fn law_query(query: String) -> Result<String, String> {
    execute_tool(
        "LawQuery",
        &json!({
            "query": query,
        }),
    )
}

/// 知识卡片查询。
#[tauri::command]
pub fn knowledge_card(topic: String) -> Result<String, String> {
    execute_tool(
        "KnowledgeCard",
        &json!({
            "topic": topic,
        }),
    )
}

/// 超级推理计划。
#[tauri::command]
pub fn super_reasoning_plan(query: String, _context: Option<String>) -> Result<String, String> {
    execute_tool(
        "SuperReasoningPlan",
        &json!({
            "query": query,
        }),
    )
}

/// 创新性评估。
#[tauri::command]
pub fn innovation_evaluator(
    invention_title: String,
    technical_field: String,
    technical_problem: String,
    technical_solution: String,
) -> Result<String, String> {
    execute_tool(
        "InnovationEvaluator",
        &json!({
            "invention_title": invention_title,
            "technical_field": technical_field,
            "technical_solution": technical_solution,
            "technical_problem": technical_problem,
        }),
    )
}

/// 语义对比分析。
#[tauri::command]
pub fn semantic_compare(
    target_text: String,
    prior_text: String,
    mode: Option<String>,
) -> Result<String, String> {
    execute_tool(
        "SemanticCompare",
        &json!({
            "target": target_text,
            "prior_art": prior_text,
            "mode": mode,
        }),
    )
}
