//! 知识库工具：统一检索、法律推理、法规查询、知识卡片、超级推理计划。

use knowledge::search::{SearchConfig, SearchMode, UnifiedSearch};
use knowledge::KnowledgePaths;
use patent_domain::legal_reasoning::LegalReasoningEngine;
use patent_domain::sqlite_graph::SqliteKnowledgeGraph;
use reasoning::{PipelineConfig, ReasoningPipeline};
use serde::Deserialize;
use serde_json::{json, Value};

fn search_engine() -> UnifiedSearch {
    let paths = KnowledgePaths::discover();
    UnifiedSearch::new(
        paths.patent_kg_db.as_deref(),
        paths.laws_db.as_deref(),
        paths.card_index.as_deref(),
    )
}

fn open_kg() -> Result<SqliteKnowledgeGraph, String> {
    let path = KnowledgePaths::discover()
        .patent_kg_db
        .ok_or_else(|| "未找到专利知识图谱数据库 (patent_kg.db)".to_string())?;
    SqliteKnowledgeGraph::open(&path).map_err(|e| e.to_string())
}

fn parse_search_mode(raw: Option<&str>) -> SearchMode {
    match raw.unwrap_or("text") {
        "semantic" => SearchMode::Semantic,
        "hybrid" => SearchMode::Hybrid,
        _ => SearchMode::Text,
    }
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeSearchInput {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_true")]
    pub search_kg: bool,
    #[serde(default = "default_true")]
    pub search_law: bool,
    #[serde(default = "default_true")]
    pub search_cards: bool,
    #[serde(default = "default_card_quality")]
    pub min_card_quality: f64,
    #[serde(default)]
    pub search_mode: Option<String>,
}

fn default_limit() -> usize {
    20
}
fn default_true() -> bool {
    true
}
fn default_card_quality() -> f64 {
    0.5
}

pub fn execute_knowledge_search(input: &KnowledgeSearchInput) -> Result<Value, String> {
    let engine = search_engine();
    let config = SearchConfig {
        query: input.query.clone(),
        limit: input.limit,
        search_kg: input.search_kg,
        search_law: input.search_law,
        search_cards: input.search_cards,
        min_card_quality: input.min_card_quality,
        mode: parse_search_mode(input.search_mode.as_deref()),
        ..Default::default()
    };
    let results = engine.search(&config);
    Ok(json!({
        "query": input.query,
        "total": results.len(),
        "results": results,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LegalReasoningInput {
    pub query: String,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_path_limit")]
    pub path_limit: usize,
}

fn default_method() -> String {
    "novelty_three_step".to_string()
}
fn default_path_limit() -> usize {
    5
}

pub fn execute_legal_reasoning(input: &LegalReasoningInput) -> Result<Value, String> {
    let kg = open_kg()?;
    let engine = LegalReasoningEngine::new(&kg);
    let paths = engine
        .find_reasoning_paths(&input.query, 3, input.path_limit)
        .unwrap_or_default();

    let conclusion = match input.method.as_str() {
        "inventiveness_problem_solution" => engine.inventiveness_problem_solution(&input.query),
        "infringement_elements" => {
            let elements: Vec<String> = input
                .query
                .split(['、', ',', ';'])
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            let elements = if elements.is_empty() {
                vec![input.query.clone()]
            } else {
                elements
            };
            engine.infringement_analysis(&elements)
        }
        _ => engine.novelty_three_step(&input.query),
    };

    Ok(json!({
        "method": input.method,
        "query": input.query,
        "paths": paths,
        "conclusion": conclusion,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LawQueryInput {
    pub keyword: String,
    #[serde(default = "default_law_mode")]
    pub mode: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_law_mode() -> String {
    "content".to_string()
}

pub fn execute_law_query(input: &LawQueryInput) -> Result<Value, String> {
    let paths = KnowledgePaths::discover();
    let db_path = paths
        .laws_db
        .ok_or_else(|| "未找到法律法规数据库 (laws.db)".to_string())?;
    let db = knowledge::law_db::LawDatabase::open(&db_path).map_err(|e| e.to_string())?;

    let docs = match input.mode.as_str() {
        "name" => db.search_by_name(&input.keyword, input.limit),
        "level" => {
            let level = input
                .level
                .as_deref()
                .ok_or_else(|| "mode=level 时需要 level 参数".to_string())?;
            db.list_by_level(level, input.limit)
        }
        "levels" => {
            let levels = db.list_levels().map_err(|e| e.to_string())?;
            return Ok(json!({ "levels": levels }));
        }
        _ => db.search_by_content(&input.keyword, input.limit),
    }
    .map_err(|e| e.to_string())?;

    Ok(json!({
        "mode": input.mode,
        "keyword": input.keyword,
        "total": docs.len(),
        "documents": docs,
    }))
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeCardInput {
    pub keyword: String,
    #[serde(default = "default_card_limit")]
    pub limit: usize,
    #[serde(default)]
    pub min_quality: Option<f64>,
    #[serde(default)]
    pub load_content: bool,
}

fn default_card_limit() -> usize {
    10
}

pub fn execute_knowledge_card(input: &KnowledgeCardInput) -> Result<Value, String> {
    let engine = search_engine();
    let config = SearchConfig {
        query: input.keyword.clone(),
        limit: input.limit,
        search_kg: false,
        search_law: false,
        search_cards: true,
        min_card_quality: input.min_quality.unwrap_or(0.0),
        mode: SearchMode::Text,
        ..Default::default()
    };
    let results = engine.search(&config);
    Ok(json!({
        "keyword": input.keyword,
        "load_content": input.load_content,
        "total": results.len(),
        "cards": results,
    }))
}

#[derive(Debug, Deserialize)]
pub struct SuperReasoningPlanInput {
    pub problem: String,
    #[serde(default = "default_hypotheses")]
    pub max_hypotheses: usize,
    #[serde(default = "default_iterations")]
    pub max_iterations: usize,
}

fn default_hypotheses() -> usize {
    5
}
fn default_iterations() -> usize {
    3
}

pub fn execute_super_reasoning_plan(input: &SuperReasoningPlanInput) -> Result<Value, String> {
    let pipeline = ReasoningPipeline::new(PipelineConfig {
        max_hypotheses: input.max_hypotheses,
        max_iterations: input.max_iterations,
        ..Default::default()
    });
    let steps = pipeline.plan(&input.problem);
    Ok(json!({
        "problem": input.problem,
        "phases": steps,
        "instructions": "按计划逐阶段调用 KnowledgeSearch / LegalReasoning / KnowledgeGraphQuery 收集证据，再综合结论。",
    }))
}
