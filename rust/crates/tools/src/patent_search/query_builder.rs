// 工具2: SearchQueryBuilder - 3阶段查询构建器

use serde::Deserialize;
use serde_json::{json, Value};

use super::synonym::build_search_query;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQueryBuilderInput {
    pub keywords: Vec<String>,
    #[serde(default)]
    pub ipc_codes: Option<Vec<String>>,
    #[serde(default)]
    #[allow(dead_code)]
    pub technical_field: Option<String>, // 保留原因: 预留给未来技术领域过滤
    #[serde(default)]
    pub technical_problem: Option<String>,
    #[serde(default)]
    pub domain_strategy: Option<String>,
}

pub fn search_query_builder(input: SearchQueryBuilderInput) -> Result<Value, String> {
    let keywords = input.keywords;
    let ipc_codes = input.ipc_codes.unwrap_or_default();
    let technical_problem = input.technical_problem.unwrap_or_default();
    let strategy = input
        .domain_strategy
        .unwrap_or_else(|| "focused".to_string());

    // 阶段1: 初始检索 (前3个关键词，高精度)
    let stage1_keywords: Vec<_> = keywords.iter().take(3).cloned().collect();
    let stage1_query = build_search_query(&stage1_keywords, "High", "all", &[])?;
    let mut stage1_final = stage1_query["query"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    // 添加IPC分类号
    if !ipc_codes.is_empty() {
        let ipc_query = ipc_codes.join(" OR ");
        stage1_final = format!("({stage1_final}) AND ({ipc_query})");
    }

    // 阶段2: 精化检索 (5个关键词，中等精度，加入技术问题)
    let stage2_keywords: Vec<_> = keywords.iter().take(5).cloned().collect();
    let stage2_query = build_search_query(&stage2_keywords, "Medium", "all", &[])?;
    let mut stage2_final = stage2_query["query"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    if !technical_problem.is_empty() {
        stage2_final = format!("({stage2_final}) AND ({technical_problem})");
    }

    // 阶段3: 补充检索 (根据策略添加领域相关术语)
    let stage3_query = build_search_query(&keywords, "Low", "all", &[])?;
    let mut stage3_final = stage3_query["query"]
        .as_str()
        .unwrap_or_default()
        .to_string();

    match strategy.as_str() {
        "broad" => {
            // 宽泛策略: 建议非专利文献
            stage3_final = format!("{stage3_final} AND (scholar OR wikipedia OR standards)");
        }
        "precise" => {
            // 精确策略: 限定在权利要求书
            let result = build_search_query(&keywords, "Medium", "inclaims", &[])?;
            if let Some(q) = result.get("query").and_then(|v| v.as_str()) {
                stage3_final = q.to_string();
            }
        }
        _ => {
            // 默认聚焦策略: 混合检索
        }
    }

    Ok(json!({
        "domain_strategy": strategy,
        "stages": [
            {
                "stage": 1,
                "name": "初始检索",
                "description": "高精度检索核心关键词",
                "query": stage1_final,
                "expected_results": "精确匹配，结果量少"
            },
            {
                "stage": 2,
                "name": "精化检索",
                "description": "中等精度，加入技术问题相关词",
                "query": stage2_final,
                "expected_results": "平衡相关性和召回率"
            },
            {
                "stage": 3,
                "name": "补充检索",
                "description": match strategy.as_str() {
                    "broad" => "低精度，建议非专利文献",
                    "precise" => "限定权利要求书检索",
                    _ => "低精度扩展检索"
                },
                "query": stage3_final,
                "expected_results": "高召回率，可能包含噪声"
            }
        ],
        "recommended_strategy": strategy
    }))
}
