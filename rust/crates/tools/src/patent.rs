//! 专利专用工具集
//!
//! 将 patent-domain / patent-knowledge / patent-workflow 的能力
//! 封装为 LLM 可调用的工具。

use std::collections::HashSet;

use patent_domain::claim_parser::ClaimParser;
use patent_domain::drafting::DraftQualityReport;
use patent_domain::models::ClaimType;
use patent_domain::rule_engine::{CaseContext, QualitativeRuleEngine};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ==================== ClaimParse 工具 ====================

/// 权利要求解析工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct ClaimParseInput {
    pub claim_text: String,
    #[serde(default)]
    pub claim_number: u32,
}

/// 权利要求解析工具输出
#[derive(Debug, Clone, Serialize)]
pub struct ClaimParseOutput {
    pub claim_number: u32,
    pub claim_type: String,
    pub preamble: String,
    pub transition_word: String,
    pub body: String,
    pub features: Vec<ParsedFeatureOutput>,
    pub dependent_from: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedFeatureOutput {
    pub id: String,
    pub description: String,
    pub feature_type: String,
    pub component: Option<String>,
    pub parameters: Vec<String>,
}

pub fn execute_claim_parse(input: &ClaimParseInput) -> Result<Value, String> {
    let parser = ClaimParser::new();
    let parsed = parser.parse(input.claim_number.max(1), &input.claim_text);

    let output = ClaimParseOutput {
        claim_number: parsed.claim_number,
        claim_type: format!("{:?}", parsed.claim_type),
        preamble: parsed.preamble,
        transition_word: parsed.transition_word,
        body: parsed.body,
        features: parsed
            .features
            .into_iter()
            .map(|f| ParsedFeatureOutput {
                id: f.id,
                description: f.description,
                feature_type: format!("{:?}", f.feature_type),
                component: f.component,
                parameters: f.parameters,
            })
            .collect(),
        dependent_from: parsed.dependent_from,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

// ==================== ClaimCompare 工具 ====================

/// 权利要求对比工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct ClaimCompareInput {
    pub claim_a: String,
    pub claim_b: String,
}

/// 权利要求对比工具输出
#[derive(Debug, Clone, Serialize)]
pub struct ClaimCompareOutput {
    pub similarity: f64,
    pub correspondence_type: String,
    pub reasoning: String,
}

pub fn execute_claim_compare(input: &ClaimCompareInput) -> Result<Value, String> {
    let parser = ClaimParser::new();
    let feat_a = parser.parse(1, &input.claim_a);
    let feat_b = parser.parse(2, &input.claim_b);

    // 取第一个特征进行对比（简化实现）
    let sim = if let (Some(a), Some(b)) = (feat_a.features.first(), feat_b.features.first()) {
        ClaimParser::feature_similarity(a, b)
    } else {
        0.0
    };

    let corr = ClaimParser::classify_correspondence(sim);
    let reasoning = match corr {
        patent_domain::models::CorrespondenceType::Exact => "特征完全相同".into(),
        patent_domain::models::CorrespondenceType::Equivalent => "特征等同".into(),
        patent_domain::models::CorrespondenceType::Different => "特征存在差异".into(),
        patent_domain::models::CorrespondenceType::Missing => "特征缺失".into(),
    };

    let output = ClaimCompareOutput {
        similarity: sim,
        correspondence_type: format!("{corr:?}"),
        reasoning,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

// ==================== NoveltyAnalysis 工具 ====================

/// 新颖性分析工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct NoveltyAnalysisInput {
    pub invention_description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prior_art_descriptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub differences: Vec<String>,
}

/// 新颖性分析工具输出
#[derive(Debug, Clone, Serialize)]
pub struct NoveltyAnalysisOutput {
    pub conclusion: String,
    pub net_score: f64,
    pub confidence: f64,
    pub applicable_rules: Vec<String>,
}

pub fn execute_novelty_analysis(input: &NoveltyAnalysisInput) -> Result<Value, String> {
    let mut engine = QualitativeRuleEngine::new();
    let context = CaseContext {
        invention: Some(input.invention_description.clone()),
        prior_art_contains_all: Some(
            input.prior_art_descriptions.len() == 1
                && input
                    .invention_description
                    .contains(&input.prior_art_descriptions[0]),
        ),
        differences: if input.differences.is_empty() {
            None
        } else {
            Some(input.differences.clone())
        },
        ..Default::default()
    };

    let result = engine
        .analyze_novelty(&context)
        .map_err(|e| e.to_string())?;

    let applicable_rules: Vec<String> = result
        .applied_rules
        .into_iter()
        .filter(|r| r.applies)
        .map(|r| format!("{}: {}", r.rule_name, r.conclusion))
        .collect();

    let output = NoveltyAnalysisOutput {
        conclusion: result.conclusion,
        net_score: result.net_score,
        confidence: result.confidence,
        applicable_rules,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

// ==================== InventivenessAnalysis 工具 ====================

/// 创造性分析工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct InventivenessAnalysisInput {
    pub invention_description: String,
    pub technical_effect: String,
    #[serde(default)]
    pub performance_improvement: Option<f64>,
    #[serde(default)]
    pub obviousness: Option<bool>,
}

pub fn execute_inventiveness_analysis(input: &InventivenessAnalysisInput) -> Result<Value, String> {
    let mut engine = QualitativeRuleEngine::new();
    let context = CaseContext {
        invention: Some(input.invention_description.clone()),
        technical_effect: Some(input.technical_effect.clone()),
        performance_improvement: input.performance_improvement,
        obviousness: input.obviousness,
        ..Default::default()
    };

    let result = engine
        .analyze_inventiveness(&context)
        .map_err(|e| e.to_string())?;

    let applicable_rules: Vec<String> = result
        .applied_rules
        .into_iter()
        .filter(|r| r.applies)
        .map(|r| format!("{}: {}", r.rule_name, r.conclusion))
        .collect();

    Ok(json!({
        "conclusion": result.conclusion,
        "net_score": result.net_score,
        "confidence": result.confidence,
        "applicable_rules": applicable_rules,
    }))
}

// ==================== OaStrategy 工具 ====================

/// OA 答复策略建议工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct OaStrategyInput {
    pub rejection_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub differences: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub technical_effects: Vec<String>,
    #[serde(default)]
    pub prior_art_different_field: Option<bool>,
}

pub fn execute_oa_strategy(input: &OaStrategyInput) -> Result<Value, String> {
    let mut engine = QualitativeRuleEngine::new();
    let context = CaseContext {
        rejection_type: Some(input.rejection_type.clone()),
        differences: if input.differences.is_empty() {
            None
        } else {
            Some(input.differences.clone())
        },
        technical_effects: if input.technical_effects.is_empty() {
            None
        } else {
            Some(input.technical_effects.clone())
        },
        prior_art_different_field: input.prior_art_different_field,
        ..Default::default()
    };

    let result = engine
        .suggest_oa_strategy(&context)
        .map_err(|e| e.to_string())?;

    let applicable_rules: Vec<String> = result
        .applied_rules
        .into_iter()
        .filter(|r| r.applies)
        .map(|r| format!("{}: {}", r.rule_name, r.conclusion))
        .collect();

    Ok(json!({
        "conclusion": result.conclusion,
        "net_score": result.net_score,
        "confidence": result.confidence,
        "applicable_rules": applicable_rules,
    }))
}

// ==================== FormalCheck 工具 ====================

/// 形式检查工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct FormalCheckInput {
    pub claims: Vec<String>,
    pub specification_sections: Vec<String>,
}

/// 形式检查工具输出
#[derive(Debug, Clone, Serialize)]
pub struct FormalCheckOutput {
    pub passed: bool,
    pub issues: Vec<FormalIssue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FormalIssue {
    pub severity: String,
    pub code: String,
    pub description: String,
    pub suggestion: String,
}

#[allow(clippy::cast_possible_truncation)]
pub fn execute_formal_check(input: &FormalCheckInput) -> Result<Value, String> {
    let mut issues = Vec::new();
    let parser = ClaimParser::new();

    // 1. 权利要求编号连续性检查
    let mut seen_numbers = HashSet::new();
    for (i, text) in input.claims.iter().enumerate() {
        let claim = parser.parse((i + 1) as u32, text);
        seen_numbers.insert(claim.claim_number);
    }

    // 检查是否有缺失的编号
    if let Some(&max_num) = seen_numbers.iter().max() {
        for n in 1..=max_num {
            if !seen_numbers.contains(&n) {
                issues.push(FormalIssue {
                    severity: "error".into(),
                    code: "FC-001".into(),
                    description: format!("缺少权利要求{n}"),
                    suggestion: format!("检查权利要求{n}是否遗漏"),
                });
            }
        }
    }

    // 2. 从属权利要求引用有效性检查
    for (i, text) in input.claims.iter().enumerate() {
        let claim = parser.parse((i + 1) as u32, text);
        if let Some(parent) = claim.dependent_from {
            if parent == 0 || parent as usize > input.claims.len() {
                issues.push(FormalIssue {
                    severity: "error".into(),
                    code: "FC-002".into(),
                    description: format!(
                        "权利要求{}引用了不存在的权利要求{}",
                        claim.claim_number, parent
                    ),
                    suggestion: "修正引用编号".into(),
                });
            }
        }
    }

    // 3. 说明书章节完整性检查
    if input.specification_sections.len() < 3 {
        issues.push(FormalIssue {
            severity: "warning".into(),
            code: "FC-003".into(),
            description: "说明书章节可能不完整".into(),
            suggestion: "确保包含技术领域、背景技术、发明内容、具体实施方式".into(),
        });
    }

    let output = FormalCheckOutput {
        passed: issues.is_empty(),
        issues,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

// ==================== QualityAssess 工具 ====================

/// 专利质量评估工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct QualityAssessInput {
    pub claims: Vec<String>,
    pub specification_word_count: Option<usize>,
}

#[allow(clippy::cast_possible_truncation)]
pub fn execute_quality_assess(input: &QualityAssessInput) -> Value {
    let parser = ClaimParser::new();
    let mut report = DraftQualityReport::default();

    // 独立性检查
    let independent_count = input
        .claims
        .iter()
        .enumerate()
        .filter(|(i, text)| {
            let claim = parser.parse((*i + 1) as u32, text);
            claim.claim_type == ClaimType::Independent
        })
        .count();

    if independent_count == 0 {
        report.critical_issues.push("缺少独立权利要求".into());
    }

    // 权利要求总数
    let total_claims = input.claims.len();
    if total_claims < 3 {
        report
            .warnings
            .push("权利要求数量较少，建议增加从属权利要求".into());
    }

    // 说明书字数
    if let Some(wc) = input.specification_word_count {
        if wc < 2000 {
            report
                .warnings
                .push("说明书字数偏少，可能影响充分公开".into());
        }
    }

    // 简化评分
    report.dimensions[0].score = if report.critical_issues.is_empty() {
        8.0
    } else {
        4.0
    };
    report.dimensions[1].score = if total_claims >= 3 { 8.0 } else { 6.0 };
    report.dimensions[2].score = 7.0;
    report.dimensions[3].score = if input.specification_word_count.unwrap_or(0) > 2000 {
        8.0
    } else {
        5.0
    };
    report.dimensions[4].score = 7.0;
    report.dimensions[5].score = 8.0;
    report.dimensions[6].score = 7.0;

    report.recalculate_overall_score();
    report.is_acceptable = report.overall_score >= 6.0 && report.critical_issues.is_empty();

    json!({
        "overall_score": report.overall_score,
        "is_acceptable": report.is_acceptable,
        "dimensions": report.dimensions,
        "critical_issues": report.critical_issues,
        "warnings": report.warnings,
    })
}

// ==================== KnowledgeGraphQuery 工具 ====================

/// 知识图谱查询工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeGraphQueryInput {
    pub query: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub node_type: Option<String>,
    #[serde(default)]
    pub graph_dir: Option<String>,
    /// 可选：SQLite 知识图谱数据库路径。若提供则优先使用 `SQLite` 查询。
    #[serde(default)]
    pub sqlite_path: Option<String>,
}

fn default_source() -> String {
    "all".into()
}
fn default_limit() -> usize {
    10
}

/// 知识图谱查询结果项
#[derive(Debug, Clone, Serialize)]
pub struct KgQueryResultItem {
    pub id: String,
    pub name: String,
    pub item_type: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
}

/// 知识图谱查询输出
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeGraphQueryOutput {
    pub query: String,
    pub source: String,
    pub total_found: usize,
    pub results: Vec<KgQueryResultItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<serde_json::Value>,
}

use std::sync::OnceLock;

static GUIDELINE_GRAPH: OnceLock<patent_domain::guideline_graph::GuidelineGraph> = OnceLock::new();
static LEGAL_GRAPH: OnceLock<patent_domain::guideline_graph::LegalKnowledgeGraph> = OnceLock::new();

fn get_graph_dir(input_dir: Option<&str>) -> String {
    if let Some(dir) = input_dir {
        return dir.to_string();
    }
    // 尝试从环境变量或默认路径推断
    if let Ok(home) = std::env::var("YUNXI_HOME") {
        return format!("{home}/assets/knowledge_graph");
    }
    if let Ok(cwd) = std::env::current_dir() {
        let candidates = [
            cwd.join("assets/knowledge_graph"),
            cwd.join("../assets/knowledge_graph"),
            cwd.join("../../assets/knowledge_graph"),
        ];
        for c in &candidates {
            if c.exists() {
                return c.to_string_lossy().to_string();
            }
        }
    }
    "assets/knowledge_graph".into()
}

/// 尝试从默认位置查找 `SQLite` 数据库
/// 优先级：环境变量 > 项目内路径 > 用户主目录
fn default_sqlite_path() -> Option<String> {
    // 1. 环境变量
    if let Ok(path) = std::env::var("PATENT_KG_DB") {
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }

    // 2. 项目内路径（相对于当前工作目录）
    let project_candidates = [
        "assets/knowledge_graph/patent_kg.db",
        "../assets/knowledge_graph/patent_kg.db",
        "../../assets/knowledge_graph/patent_kg.db",
    ];
    if let Ok(cwd) = std::env::current_dir() {
        for candidate in &project_candidates {
            let path = cwd.join(candidate);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    // 3. 用户主目录下的默认位置
    if let Ok(home) = std::env::var("HOME") {
        let path = format!("{home}/.openclaw/workspace/memory/patent-knowledge-graph/patent_kg.db");
        if std::path::Path::new(&path).exists() {
            return Some(path);
        }
    }
    None
}

pub fn execute_knowledge_graph_query(input: &KnowledgeGraphQueryInput) -> Result<Value, String> {
    // 优先使用 SQLite 数据源
    let sqlite_path = input.sqlite_path.clone().or_else(default_sqlite_path);

    if let Some(ref path) = sqlite_path {
        if std::path::Path::new(path).exists() {
            return query_sqlite_kg(input, path);
        }
    }

    // 回退到 JSON 文件查询
    query_json_kg(input)
}

fn query_sqlite_kg(input: &KnowledgeGraphQueryInput, path: &str) -> Result<Value, String> {
    let kg =
        patent_domain::sqlite_graph::SqliteKnowledgeGraph::open(path).map_err(|e| e.to_string())?;

    let node_type = input.node_type.as_deref();
    let nodes = kg
        .search_nodes(&input.query, node_type, input.limit)
        .map_err(|e| e.to_string())?;

    let results: Vec<KgQueryResultItem> = nodes
        .into_iter()
        .map(|node| KgQueryResultItem {
            id: node.id.clone(),
            name: if node.name.is_empty() {
                node.title.clone()
            } else {
                node.name.clone()
            },
            item_type: node.node_type.clone(),
            content: match &node.content {
                Some(c) if c.chars().count() > 500 => {
                    let truncated: String = c.chars().take(500).collect();
                    format!("{truncated}...")
                }
                Some(c) => c.clone(),
                None => String::new(),
            },
            relationship: None,
            source_type: Some("sqlite".into()),
        })
        .collect();

    let stats = kg.stats().map_err(|e| e.to_string())?;
    let type_dist = kg.node_type_distribution().map_err(|e| e.to_string())?;

    let stats_json = json!({
        "sqlite_nodes": stats.node_count,
        "sqlite_edges": stats.edge_count,
        "node_types": type_dist.into_iter().take(10).collect::<Vec<_>>(),
    });

    let output = KnowledgeGraphQueryOutput {
        query: input.query.clone(),
        source: input.source.clone(),
        total_found: results.len(),
        results,
        stats: Some(stats_json),
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

#[allow(clippy::too_many_lines)]
fn query_json_kg(input: &KnowledgeGraphQueryInput) -> Result<Value, String> {
    let graph_dir = get_graph_dir(input.graph_dir.as_deref());

    // 加载或获取审查指南图谱
    let guideline_loaded = if input.source == "all" || input.source == "guideline" {
        let graph = GUIDELINE_GRAPH.get_or_init(|| {
            let path = format!("{graph_dir}/guideline_graph.json");
            match std::fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                    patent_domain::guideline_graph::GuidelineGraph {
                        metadata: patent_domain::guideline_graph::GuidelineMetadata {
                            title: "Error".into(),
                            description: e.to_string(),
                            created: String::new(),
                            sections: vec![],
                            total_nodes: 0,
                            total_relationships: 0,
                        },
                        nodes: vec![],
                        relationships: vec![],
                        vectors: vec![],
                    }
                }),
                Err(e) => patent_domain::guideline_graph::GuidelineGraph {
                    metadata: patent_domain::guideline_graph::GuidelineMetadata {
                        title: "Error".into(),
                        description: e.to_string(),
                        created: String::new(),
                        sections: vec![],
                        total_nodes: 0,
                        total_relationships: 0,
                    },
                    nodes: vec![],
                    relationships: vec![],
                    vectors: vec![],
                },
            }
        });
        Some(graph)
    } else {
        None
    };

    // 加载或获取法律知识图谱
    let legal_loaded = if input.source == "all" || input.source == "legal" {
        let graph = LEGAL_GRAPH.get_or_init(|| {
            let entities_path = format!("{graph_dir}/legal_entities.json");
            let rels_path = format!("{graph_dir}/legal_relationships.json");
            let mut kg = patent_domain::guideline_graph::LegalKnowledgeGraph {
                entities: std::collections::HashMap::new(),
                relationships: vec![],
            };
            if let Ok(content) = std::fs::read_to_string(&entities_path) {
                if let Ok(entities) = serde_json::from_str(&content) {
                    kg.entities = entities;
                }
            }
            if let Ok(content) = std::fs::read_to_string(&rels_path) {
                if let Ok(rels) = serde_json::from_str(&content) {
                    kg.relationships = rels;
                }
            }
            kg
        });
        Some(graph)
    } else {
        None
    };

    let mut results: Vec<KgQueryResultItem> = Vec::new();

    // 查询审查指南图谱
    // TODO: 当节点数量超过千级时，构建 HashMap<String, Vec<NodeId>> 搜索索引优化为 O(1)
    if let Some(graph) = guideline_loaded {
        let kw_lower = input.query.to_lowercase();
        for node in &graph.nodes {
            let matched = if let Some(ref nt) = input.node_type {
                node.node_type.eq_ignore_ascii_case(nt)
            } else {
                node.properties
                    .title
                    .as_ref()
                    .is_some_and(|t| t.to_lowercase().contains(&kw_lower))
                    || node
                        .properties
                        .content
                        .as_ref()
                        .is_some_and(|c| c.to_lowercase().contains(&kw_lower))
                    || node
                        .properties
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&kw_lower))
            };
            if matched {
                results.push(KgQueryResultItem {
                    id: node.id.clone(),
                    name: node
                        .properties
                        .title
                        .clone()
                        .unwrap_or_else(|| node.id.clone()),
                    item_type: node.node_type.clone(),
                    content: match &node.properties.content {
                        Some(c) if c.chars().count() > 500 => {
                            let truncated: String = c.chars().take(500).collect();
                            format!("{truncated}...")
                        }
                        Some(c) => c.clone(),
                        None => String::new(),
                    },
                    relationship: None,
                    source_type: Some("guideline".into()),
                });
                if results.len() >= input.limit {
                    break;
                }
            }
        }
    }

    // 查询法律知识图谱
    // TODO: 同上，构建实体名→ID 反向索引
    if results.len() < input.limit {
        if let Some(graph) = legal_loaded {
            let kw_lower = input.query.to_lowercase();
            let remaining = input.limit - results.len();
            for entity in graph.entities.values() {
                if entity.name.to_lowercase().contains(&kw_lower)
                    || entity.description.to_lowercase().contains(&kw_lower)
                {
                    results.push(KgQueryResultItem {
                        id: entity.id.clone(),
                        name: entity.name.clone(),
                        item_type: entity.entity_type.clone(),
                        content: if entity.description.chars().count() > 500 {
                            let truncated: String = entity.description.chars().take(500).collect();
                            format!("{truncated}...")
                        } else {
                            entity.description.clone()
                        },
                        relationship: None,
                        source_type: Some("legal".into()),
                    });
                    if results.len() >= input.limit {
                        break;
                    }
                    if results.len() >= input.limit
                        || results
                            .iter()
                            .filter(|r| r.source_type.as_deref() == Some("legal"))
                            .count()
                            >= remaining
                    {
                        break;
                    }
                }
            }
        }
    }

    let stats = json!({
        "guideline_nodes": guideline_loaded.map_or(0, |g| g.nodes.len()),
        "guideline_relationships": guideline_loaded.map_or(0, |g| g.relationships.len()),
        "legal_entities": legal_loaded.map_or(0, |g| g.entities.len()),
        "legal_relationships": legal_loaded.map_or(0, |g| g.relationships.len()),
    });

    let output = KnowledgeGraphQueryOutput {
        query: input.query.clone(),
        source: input.source.clone(),
        total_found: results.len(),
        results,
        stats: Some(stats),
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}
