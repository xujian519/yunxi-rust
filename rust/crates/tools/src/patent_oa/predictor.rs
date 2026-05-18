// 工具 3: SuccessPredictor - OA 成功率预测器

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 成功率预测工具输入
#[derive(Debug, Clone, Deserialize)]
pub struct SuccessPredictorInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_result: Option<Value>,
    pub strategy: String, // argue/amend/both/abandon/appeal
    #[serde(default)]
    pub round: u32,
    #[serde(default)]
    pub confidence_level: String, // 90%/95%/99%
    #[serde(skip_serializing_if = "Option::is_none")]
    pub historical_cases: Option<Vec<HistoricalCase>>,
}

/// 历史案例
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalCase {
    pub rejection_types: Vec<String>,
    pub strategy: String,
    pub claim_count: u32,
    pub round: u32,
    pub success: bool,
}

/// 成功率预测输出
#[derive(Debug, Clone, Serialize)]
pub struct SuccessPredictorOutput {
    pub predicted_success_rate: f64,
    pub confidence_interval: ConfidenceInterval,
    pub confidence_level: String,
    pub success_factors: Vec<String>,
    pub risk_factors: Vec<String>,
    pub strategy_recommendation: String,
    pub base_score: f64,
    pub adjustments: HashMap<String, f64>,
    pub historical_boost: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
}

pub fn execute_success_predictor(input: &SuccessPredictorInput) -> Result<Value, String> {
    // 基础分数（基于驳回类型）
    let base_score = calculate_base_score(input.parse_result.as_ref());

    // 调整因子
    let mut adjustments = HashMap::new();

    // 策略调整
    let strategy_adj = match input.strategy.as_str() {
        "argue" => 0.55,
        "amend" => 0.60,
        "both" => 0.65,
        "abandon" => 0.10,
        "appeal" => 0.35,
        _ => 0.50,
    };
    adjustments.insert("strategy".to_string(), strategy_adj);

    // 严重程度调整
    let severity_adj = calculate_severity_adjustment(input.parse_result.as_ref());
    adjustments.insert("severity".to_string(), severity_adj);

    // 轮次惩罚
    let round_penalty = if input.round > 1 {
        -0.08 * (f64::from(input.round) - 1.0)
    } else {
        0.0
    };
    adjustments.insert("round".to_string(), round_penalty);

    // 权利要求数量惩罚
    #[allow(clippy::cast_precision_loss)]
    let claim_penalty = if let Some(ref result) = input.parse_result {
        let claim_count = result["total_affected_claims"]
            .as_array()
            .map_or(0.0, |a| a.len() as f64);
        if claim_count > 3.0 {
            -0.03 * (claim_count - 3.0)
        } else {
            0.0
        }
    } else {
        0.0
    };
    adjustments.insert("claim_count".to_string(), claim_penalty);

    // 引用文献惩罚
    #[allow(clippy::cast_precision_loss)]
    let ref_penalty = if let Some(ref result) = input.parse_result {
        let ref_count = result["rejection_reasons"].as_array().map_or(0.0, |arr| {
            arr.iter()
                .filter_map(|r| r["cited_references"].as_array())
                .map(|refs| refs.len() as f64)
                .sum::<f64>()
        });
        if ref_count > 1.0 {
            -0.05 * (ref_count - 1.0)
        } else {
            0.0
        }
    } else {
        0.0
    };
    adjustments.insert("reference_count".to_string(), ref_penalty);

    // 历史案例加成
    let historical_boost = if let Some(ref cases) = input.historical_cases {
        if cases.is_empty() {
            None
        } else {
            Some(calculate_historical_boost(cases, input))
        }
    } else {
        None
    };

    // 计算最终成功率
    let mut predicted_rate = base_score;
    for adj in adjustments.values() {
        predicted_rate += adj;
    }
    if let Some(boost) = historical_boost {
        predicted_rate += boost;
    }
    predicted_rate = predicted_rate.clamp(0.0, 1.0);

    // 置信区间
    let ci = calculate_confidence_interval(predicted_rate, &input.confidence_level);

    // 成功和风险因素
    let (success_factors, risk_factors) = generate_factors(input.parse_result.as_ref(), input);

    // 策略建议
    let recommendation = generate_strategy_recommendation(predicted_rate, input);

    let output = SuccessPredictorOutput {
        predicted_success_rate: predicted_rate,
        confidence_interval: ci,
        confidence_level: input.confidence_level.clone(),
        success_factors,
        risk_factors,
        strategy_recommendation: recommendation,
        base_score,
        adjustments,
        historical_boost,
    };

    serde_json::to_value(output).map_err(|e| e.to_string())
}

/// 计算基础分数（基于驳回类型）
#[allow(clippy::cast_precision_loss)]
fn calculate_base_score(parse_result: Option<&Value>) -> f64 {
    let mut score = 0.5;

    if let Some(result) = parse_result {
        let rejection_types = result["rejection_reasons"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r["type"].as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if !rejection_types.is_empty() {
            // 基于驳回类型权重的平均分
            let weights: HashMap<&str, f64> = [
                ("Novelty", 0.25),
                ("Inventiveness", 0.30),
                ("Utility", 0.45),
                ("Support", 0.50),
                ("Clarity", 0.55),
                ("Scope", 0.40),
                ("AmendmentScope", 0.35),
                ("Unity", 0.60),
                ("Formality", 0.65),
                ("Other", 0.50),
            ]
            .iter()
            .copied()
            .collect();

            let total_weight: f64 = rejection_types.iter().filter_map(|t| weights.get(t)).sum();

            if total_weight > 0.0 {
                score = total_weight / rejection_types.len() as f64;
            }
        }
    }

    score
}

/// 计算严重程度调整
fn calculate_severity_adjustment(parse_result: Option<&Value>) -> f64 {
    if let Some(result) = parse_result {
        let overall_severity = result["overall_severity"].as_str().unwrap_or("medium");
        match overall_severity {
            "high" => -0.25,
            "low" => 0.15,
            _ => 0.0,
        }
    } else {
        0.0
    }
}

/// 计算历史案例加成
fn calculate_historical_boost(cases: &[HistoricalCase], input: &SuccessPredictorInput) -> f64 {
    let current_rejection_types = if let Some(ref result) = input.parse_result {
        result["rejection_reasons"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r["type"].as_str())
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    if current_rejection_types.is_empty() {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut total_similarity = 0.0;

    for case in cases {
        // 计算 Jaccard 相似度
        let similarity = jaccard_similarity(&current_rejection_types, &case.rejection_types);

        if similarity > 0.0 {
            let weight = if case.success { 1.0 } else { 0.0 };
            weighted_sum += weight * similarity;
            total_similarity += similarity;
        }
    }

    if total_similarity > 0.0 {
        (weighted_sum / total_similarity - 0.5) * 0.2 // 归一化并缩放
    } else {
        0.0
    }
}

/// Jaccard 相似度
#[allow(clippy::cast_precision_loss)]
fn jaccard_similarity<T: std::hash::Hash + Eq>(a: &[T], b: &[T]) -> f64 {
    let set_a: std::collections::HashSet<_> = a.iter().collect();
    let set_b: std::collections::HashSet<_> = b.iter().collect();

    let intersection = set_a.intersection(&set_b).count() as f64;
    let union = set_a.union(&set_b).count() as f64;

    if union > 0.0 {
        intersection / union
    } else {
        0.0
    }
}

/// 计算置信区间
fn calculate_confidence_interval(rate: f64, level: &str) -> ConfidenceInterval {
    // 标准误差估计（简化）
    let se = 0.08; // 假设标准误差为 8%

    let z_score = match level {
        "90%" => 1.645,
        "99%" => 2.576,
        _ => 1.96,
    };

    let margin = z_score * se;

    ConfidenceInterval {
        lower: (rate - margin).max(0.0),
        upper: (rate + margin).min(1.0),
    }
}

/// 生成成功和风险因素
fn generate_factors(
    parse_result: Option<&Value>,
    input: &SuccessPredictorInput,
) -> (Vec<String>, Vec<String>) {
    let mut success = Vec::new();
    let mut risk = Vec::new();

    if let Some(result) = parse_result {
        // 基于驳回类型的因素
        if let Some(reasons) = result["rejection_reasons"].as_array() {
            for reason in reasons {
                if let Some(rtype) = reason["type"].as_str() {
                    match rtype {
                        "Formality" => success.push("形式缺陷通常容易克服".to_string()),
                        "Clarity" => success.push("清楚性问题可通过澄清解决".to_string()),
                        "Novelty" => risk.push("新颖性驳回需要强有力的区别技术特征".to_string()),
                        "Inventiveness" => risk.push("创造性驳回较难克服".to_string()),
                        _ => {}
                    }
                }

                if let Some(severity) = reason["severity"].as_str() {
                    if severity == "high" {
                        risk.push("存在高严重程度驳回理由".to_string());
                    } else if severity == "low" {
                        success.push("驳回理由严重程度较低".to_string());
                    }
                }
            }
        }

        // 基于策略的因素
        match input.strategy.as_str() {
            "both" => success.push("结合争辩和修改的策略通常更有效".to_string()),
            "amend" => success.push("修改权利要求可直接解决审查员关切".to_string()),
            "argue" => risk.push("纯争辩策略需要现有技术支持".to_string()),
            _ => {}
        }

        // 基于轮次的因素
        if input.round > 1 {
            risk.push(format!("第{}轮答复难度较大", input.round));
        }
    }

    (success, risk)
}

/// 生成策略建议
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn generate_strategy_recommendation(rate: f64, _input: &SuccessPredictorInput) -> String {
    if rate >= 0.70 {
        format!(
            "预测成功率较高 ({}%)，建议继续执行当前策略",
            (rate * 100.0) as u32
        )
    } else if rate >= 0.50 {
        format!(
            "预测成功率中等 ({}%)，建议考虑加强答复论证或修改权利要求",
            (rate * 100.0) as u32
        )
    } else if rate >= 0.30 {
        format!(
            "预测成功率较低 ({}%)，建议重新评估策略或考虑修改权利要求",
            (rate * 100.0) as u32
        )
    } else {
        format!(
            "预测成功率很低 ({}%)，建议考虑放弃或请求复审",
            (rate * 100.0) as u32
        )
    }
}
