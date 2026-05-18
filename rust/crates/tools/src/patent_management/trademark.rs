//! 商标分析与专利下载工具
//!
//! 提供：
//! - 商标可注册性分析（基于规则评分）
//! - 专利下载（单件/批量，存根实现）

use serde::Deserialize;

// ==================== 3. TrademarkAnalysis - 商标可注册性分析 ====================

/// 商标分析输入
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrademarkAnalysisInput {
    pub trademark_name: String,
    pub goods_services: Option<String>,
    pub trademark_type: Option<String>,
}

/// 执行商标分析
pub fn execute_trademark_analysis(
    input: TrademarkAnalysisInput,
) -> Result<serde_json::Value, String> {
    let name = input.trademark_name.trim();
    if name.is_empty() {
        return Err("trademark_name cannot be empty".to_string());
    }

    let goods_services = input.goods_services.unwrap_or_default();
    let trademark_type = input.trademark_type.unwrap_or_else(|| String::from("word"));

    let mut score = 0.0;
    let mut factors = Vec::new();

    // 1. 长度评分
    let length_score = if name.len() <= 2 {
        0.3
    } else if name.len() <= 4 {
        0.5
    } else {
        0.7
    };
    score += length_score;
    factors.push(format!(
        "长度评分: {:.2} ({} 字符)",
        length_score,
        name.len()
    ));

    // 2. 常见词汇检测
    let common_words = vec![
        "优质", "最佳", "超级", "特级", "顶级", "第一", "首选", "名牌", "王", "皇",
    ];
    let common_penalty = common_words.iter().any(|w| name.contains(w));
    if common_penalty {
        score -= 0.2;
        factors.push("常见词汇扣分: -0.2".to_string());
    }

    // 3. 描述性检测
    if !goods_services.is_empty() {
        // 检查商标名称是否直接描述商品/服务特点
        let direct_description = goods_services.contains(name);
        if direct_description {
            score -= 0.15;
            factors.push("描述性扣分: -0.15".to_string());
        }
    }

    // 4. 类型加成
    let type_bonus = match trademark_type.as_str() {
        "design" => 0.1,
        "composite" => 0.05,
        _ => 0.0,
    };
    score += type_bonus;
    factors.push(format!("类型加成: {type_bonus:.2}"));

    // 5. 显著性评级
    let distinctiveness = if score >= 0.8 {
        "高"
    } else if score >= 0.5 {
        "中"
    } else {
        "低"
    };

    // 6. 生成建议
    let mut recommendations = Vec::new();
    if score < 0.5 {
        recommendations.push("建议重新设计商标，增强显著性".to_string());
        recommendations.push("避免使用常见词汇或直接描述产品特点".to_string());
    }
    if common_penalty {
        recommendations.push("避免使用绝对化用语，可能违反广告法".to_string());
    }
    if name.len() <= 2 {
        recommendations.push("商标过短，可能缺乏显著性".to_string());
    }
    if recommendations.is_empty() {
        recommendations.push("商标具备较好的可注册性".to_string());
    }

    Ok(serde_json::json!({
        "trademark_name": name,
        "goods_services": goods_services,
        "trademark_type": trademark_type,
        "registrability_score": score,
        "distinctiveness": distinctiveness,
        "factors": factors,
        "recommendations": recommendations,
        "overall_assessment": if score >= 0.7 { "良好" } else if score >= 0.5 { "一般" } else { "较差" }
    }))
}

// ==================== 4. PatentDownload - 单件专利下载 ====================

/// 专利下载输入
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentDownloadInput {
    pub patent_id: String,
    pub output_dir: Option<String>,
    pub format: Option<String>,
}

/// 执行单件专利下载（存根）
#[allow(clippy::unnecessary_wraps)]
pub fn execute_patent_download(input: PatentDownloadInput) -> Result<serde_json::Value, String> {
    let format = input.format.unwrap_or_else(|| String::from("pdf"));

    Ok(serde_json::json!({
        "message": "专利下载服务尚未配置",
        "patent_id": input.patent_id,
        "output_dir": input.output_dir.unwrap_or_else(|| String::from("./downloads")),
        "format": format,
        "status": "pending_configuration",
        "note": "需要配置专利数据库访问密钥和API端点"
    }))
}

// ==================== 5. BatchPatentDownload - 批量专利下载 ====================

/// 批量专利下载输入
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchPatentDownloadInput {
    pub patent_ids: Vec<String>,
    pub output_dir: Option<String>,
    pub format: Option<String>,
}

/// 执行批量专利下载（存根）
#[allow(clippy::unnecessary_wraps)]
pub fn execute_batch_patent_download(
    input: BatchPatentDownloadInput,
) -> Result<serde_json::Value, String> {
    let total = input.patent_ids.len();
    let format = input.format.unwrap_or_else(|| String::from("pdf"));

    Ok(serde_json::json!({
        "message": "批量专利下载服务尚未配置",
        "total": total,
        "downloaded": 0,
        "failed": total,
        "output_dir": input.output_dir.unwrap_or_else(|| String::from("./downloads")),
        "format": format,
        "status": "pending_configuration",
        "note": "需要配置专利数据库访问密钥和API端点",
        "patent_ids": input.patent_ids
    }))
}
