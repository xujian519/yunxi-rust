//! 审查员模拟器工具包装层

use serde::Deserialize;
use serde_json::Value;

use patent_domain::examiner_simulator::ExaminerSimulator;

/// 审查员模拟输入
#[derive(Debug, Deserialize)]
pub struct ExaminerSimulateInput {
    /// 审查意见文本
    pub oa_text: String,
    /// 权利要求列表
    pub claims: Vec<String>,
    /// 现有技术分析 (JSON)
    pub prior_art_analysis: Value,
    /// 操作模式: simulate_initial | respond | evaluate
    pub mode: String,
    /// 申请人答复文本（respond/evaluate 模式使用）
    pub applicant_response: Option<String>,
    /// 答复轮次（respond 模式使用）
    pub round_number: Option<u32>,
}

/// 执行审查员模拟
pub fn examiner_simulate(input: ExaminerSimulateInput) -> Result<Value, String> {
    match input.mode.as_str() {
        "simulate_initial" => {
            let mut sim = ExaminerSimulator::new();
            let result = sim.simulate_initial_review(
                &input.oa_text,
                &input.claims,
                &input.prior_art_analysis,
            );
            Ok(result)
        }
        "respond" => {
            let sim = ExaminerSimulator::new();
            let argument = input
                .applicant_response
                .ok_or("applicant_response required")?;
            let round = input.round_number.unwrap_or(1);
            let result =
                sim.respond_to_applicant_argument(&argument, &input.prior_art_analysis, round);
            Ok(result)
        }
        "evaluate" => {
            let response = input
                .applicant_response
                .ok_or("applicant_response required")?;
            let result = ExaminerSimulator::evaluate_final_response(&response);
            Ok(result)
        }
        _ => Err(format!("Unknown mode: {}", input.mode)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_examiner_simulate_initial() {
        let input = ExaminerSimulateInput {
            oa_text: "根据专利法第22条第3款，权利要求1不具备创造性。".into(),
            claims: vec!["1. 一种方法，包括步骤A和步骤B。".into()],
            prior_art_analysis: json!({
                "d1": {
                    "undisclosed_features": [],
                    "implementation": "已知方法"
                }
            }),
            mode: "simulate_initial".into(),
            applicant_response: None,
            round_number: None,
        };

        let result = examiner_simulate(input).unwrap();
        assert_eq!(result["rejectionType"], "inventiveness");
        assert!(!result["objections"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_examiner_respond() {
        let input = ExaminerSimulateInput {
            oa_text: "".into(),
            claims: vec![],
            prior_art_analysis: json!({}),
            mode: "respond".into(),
            applicant_response: Some("四要素产生了协同效果。".into()),
            round_number: Some(1),
        };

        let result = examiner_simulate(input).unwrap();
        assert_eq!(result["responseStrategy"], "strict");
    }

    #[test]
    fn test_examiner_evaluate() {
        let input = ExaminerSimulateInput {
            oa_text: "".into(),
            claims: vec![],
            prior_art_analysis: json!({}),
            mode: "evaluate".into(),
            applicant_response: Some(
                "因此权利要求具备创造性。参见对比文件D1。实验数据显示效果显著。".into(),
            ),
            round_number: None,
        };

        let result = examiner_simulate(input).unwrap();
        assert!(result["overallScore"].as_f64().unwrap() > 0.0);
    }
}
