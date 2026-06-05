// 工具3-6: PatentSearch, GooglePatentsFetch, HighCitationPatents, IterativeSearch

use serde::Deserialize;
use serde_json::{json, Value};

use super::synonym::expand_synonyms;

// ============================================================================
// 工具3: PatentSearch - 本地专利检索（patent_db / PostgreSQL）
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentSearchInput {
    pub query: String,
    #[serde(default)]
    pub search_type: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

fn psql_binary() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/xujian".to_string());
    let candidates = [
        format!("{home}/.local/bin/psql"),
        "/opt/homebrew/bin/psql".to_string(),
        "/usr/local/bin/psql".to_string(),
        "/opt/homebrew/Cellar/postgresql@17/17.10/bin/psql".to_string(),
    ];
    for c in &candidates {
        if std::path::Path::new(c).exists() {
            return c.clone();
        }
    }
    "psql".to_string()
}

fn load_db_config() -> (String, String, String, String) {
    let env_path = std::env::var("HOME")
        .map(|h| format!("{h}/.infra/infra.env"))
        .unwrap_or_default();
    let content = std::fs::read_to_string(&env_path).unwrap_or_default();

    let host = content
        .lines()
        .find(|l| l.starts_with("PGHOST="))
        .and_then(|l| l.trim_start_matches("PGHOST=").split('#').next())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let port = content
        .lines()
        .find(|l| l.starts_with("PGPORT="))
        .and_then(|l| l.trim_start_matches("PGPORT=").split('#').next())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "6432".to_string());

    let user = content
        .lines()
        .find(|l| l.starts_with("PGUSER="))
        .and_then(|l| l.trim_start_matches("PGUSER=").split('#').next())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "xujian".to_string());

    (host, port, user, "patent_db".to_string())
}

pub fn patent_search(input: PatentSearchInput) -> Result<Value, String> {
    let limit = input.limit.unwrap_or(10).min(100);
    let offset = input.offset.unwrap_or(0);
    let search_type = input.search_type.as_deref().unwrap_or("keyword");

    let (host, port, user, db) = load_db_config();
    let psql = psql_binary();

    let query = input.query.replace('\'', "''");

    let sql = match search_type {
        "applicant" => format!(
            "SELECT patent_name, application_number, applicant, ipc_main_class, application_date \
             FROM patents WHERE applicant ILIKE '%{query}%' \
             ORDER BY application_date DESC LIMIT {limit} OFFSET {offset}"
        ),
        "inventor" => format!(
            "SELECT patent_name, application_number, inventor, applicant, application_date \
             FROM patents WHERE inventor ILIKE '%{query}%' \
             ORDER BY application_date DESC LIMIT {limit} OFFSET {offset}"
        ),
        "ipc" => format!(
            "SELECT patent_name, application_number, ipc_code, applicant, application_date \
             FROM patents WHERE ipc_code ILIKE '{query}%' OR ipc_main_class ILIKE '{query}%' \
             ORDER BY application_date DESC LIMIT {limit} OFFSET {offset}"
        ),
        "fulltext" => format!(
            "SELECT patent_name, application_number, applicant, ipc_main_class, application_date \
             FROM patents WHERE search_vector @@ to_tsquery('chinese', '{query}') \
             ORDER BY application_date DESC LIMIT {limit} OFFSET {offset}"
        ),
        "detail" => format!(
            "SELECT patent_name, application_number, publication_number, authorization_number, \
             application_date, publication_date, authorization_date, \
             applicant, applicant_address, current_assignee, inventor, \
             ipc_code, ipc_main_class, abstract, citation_count, cited_count \
             FROM patents WHERE publication_number = '{query}' \
             OR application_number = '{query}' OR authorization_number = '{query}' \
             LIMIT 1"
        ),
        _ => format!(
            "SELECT patent_name, application_number, applicant, ipc_main_class, application_date \
             FROM patents WHERE patent_name ILIKE '%{query}%' OR abstract ILIKE '%{query}%' \
             ORDER BY application_date DESC LIMIT {limit} OFFSET {offset}"
        ),
    };

    let output = std::process::Command::new(&psql)
        .args(["-h", &host, "-p", &port, "-U", &user])
        .args(["-d", &db, "-t", "-A", "-F", "\t"])
        .arg("-c")
        .arg(&sql)
        .env("PGPASSWORD", "")
        .output()
        .map_err(|e| format!("psql 执行失败: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("could not connect") || stderr.contains("Connection refused") {
            return Ok(json!({
                "status": "unavailable",
                "message": "patent_db 数据库不可用，请确认 PostgreSQL 和 PgBouncer 正在运行",
                "query": input.query,
                "hint": "检查 ~/.infra/infra.env 配置和数据库服务状态"
            }));
        }
        return Err(format!("数据库查询失败: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let rows: Vec<Vec<String>> = stdout
        .trim()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| line.split('\t').map(String::from).collect())
        .collect();

    if rows.is_empty() {
        return Ok(json!({
            "status": "ok",
            "query": input.query,
            "search_type": search_type,
            "total": 0,
            "results": []
        }));
    }

    let results: Vec<Value> = if search_type == "detail" {
        rows.iter()
            .map(|r| {
                json!({
                    "patent_name": r.get(0).cloned().unwrap_or_default(),
                    "application_number": r.get(1).cloned().unwrap_or_default(),
                    "publication_number": r.get(2).cloned().unwrap_or_default(),
                    "authorization_number": r.get(3).cloned().unwrap_or_default(),
                    "application_date": r.get(4).cloned().unwrap_or_default(),
                    "publication_date": r.get(5).cloned().unwrap_or_default(),
                    "authorization_date": r.get(6).cloned().unwrap_or_default(),
                    "applicant": r.get(7).cloned().unwrap_or_default(),
                    "applicant_address": r.get(8).cloned().unwrap_or_default(),
                    "current_assignee": r.get(9).cloned().unwrap_or_default(),
                    "inventor": r.get(10).cloned().unwrap_or_default(),
                    "ipc_code": r.get(11).cloned().unwrap_or_default(),
                    "ipc_main_class": r.get(12).cloned().unwrap_or_default(),
                    "abstract": r.get(13).cloned().unwrap_or_default(),
                    "citation_count": r.get(14).cloned().unwrap_or_default(),
                    "cited_count": r.get(15).cloned().unwrap_or_default(),
                })
            })
            .collect()
    } else {
        rows.iter()
            .map(|r| {
                json!({
                    "patent_name": r.get(0).cloned().unwrap_or_default(),
                    "application_number": r.get(1).cloned().unwrap_or_default(),
                    "applicant": r.get(2).cloned().unwrap_or_default(),
                    "ipc_main_class": r.get(3).cloned().unwrap_or_default(),
                    "application_date": r.get(4).cloned().unwrap_or_default(),
                })
            })
            .collect()
    };

    Ok(json!({
        "status": "ok",
        "query": input.query,
        "search_type": search_type,
        "total": results.len(),
        "limit": limit,
        "offset": offset,
        "results": results
    }))
}

// ============================================================================
// 工具4: GooglePatentsFetch - Google专利检索
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GooglePatentsFetchInput {
    pub query: String,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub language: Option<String>,
}

#[allow(clippy::unnecessary_wraps)]
pub fn google_patents_fetch(input: GooglePatentsFetchInput) -> Result<Value, String> {
    let page = input.page.unwrap_or(1);
    let language = input.language.unwrap_or_else(|| "zh".to_string());

    // Stub实现
    Ok(json!({
        "status": "stub",
        "message": "Google Patents API尚未配置",
        "query": input.query,
        "page": page,
        "language": language,
        "note": "需要配置Google Patents API访问凭证",
        "alternative": "可以访问 https://patents.google.com 进行手动检索"
    }))
}

// ============================================================================
// 工具5: HighCitationPatents - 高被引专利查找
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HighCitationPatentsInput {
    pub technology: String,
    #[serde(default)]
    pub ipc_code: Option<String>,
    #[serde(default)]
    pub min_citations: Option<u32>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[allow(clippy::unnecessary_wraps)]
pub fn high_citation_patents(input: &HighCitationPatentsInput) -> Result<Value, String> {
    let min_citations = input.min_citations.unwrap_or(50);
    let limit = input.limit.unwrap_or(20);

    // Stub实现
    Ok(json!({
        "status": "stub",
        "message": "高被引专利检索服务尚未配置",
        "technology": input.technology,
        "ipc_code": input.ipc_code,
        "min_citations": min_citations,
        "limit": limit,
        "note": "需要集成专利引用数据库或API",
        "suggestion": "可以使用Google Patents按引用次数排序"
    }))
}

// ============================================================================
// 工具6: IterativeSearch - 迭代深度检索
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IterativeSearchInput {
    pub query: String,
    #[serde(default)]
    pub max_iterations: Option<u32>,
    #[serde(default)]
    pub search_type: Option<String>,
    #[serde(default)]
    pub width: Option<usize>,
}

#[allow(clippy::unnecessary_wraps)]
pub fn iterative_search(input: IterativeSearchInput) -> Result<Value, String> {
    let max_iterations = input.max_iterations.unwrap_or(3);
    let search_type = input.search_type.unwrap_or_else(|| "patent".to_string());
    let width = input.width.unwrap_or(3);

    // 基于规则的迭代检索计划
    let mut search_plan = Vec::new();

    // 生成查询变体 (基于同义词扩展)
    let terms: Vec<String> = input
        .query
        .split_whitespace()
        .map(std::string::ToString::to_string)
        .collect();
    if let Ok(expanded) = expand_synonyms(&terms) {
        if let Some(synonyms) = expanded["expanded_synonyms"].as_array() {
            let syn_list: Vec<_> = synonyms
                .iter()
                .filter_map(|s| s.as_str())
                .take(width * 2)
                .collect();

            for i in 0..max_iterations {
                #[allow(clippy::cast_possible_truncation)]
                let start = (i as usize) * width;
                let subset: Vec<_> = syn_list.iter().copied().skip(start).take(width).collect();

                if !subset.is_empty() {
                    search_plan.push(json!({
                        "iteration": i + 1,
                        "query_terms": subset,
                        "search_type": search_type,
                        "expected_results": format!("基于同义词扩展的第{}轮检索", i + 1)
                    }));
                }
            }
        }
    }

    Ok(json!({
        "original_query": input.query,
        "max_iterations": max_iterations,
        "search_type": search_type,
        "width": width,
        "mode": "rule_based",
        "search_plan": search_plan,
        "note": "LLM增强模式需要配置LLM访问"
    }))
}
