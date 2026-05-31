use crate::quality_gate::{QualityGateConfig, QualityGateResult};
use serde::Serialize;

/// 验证器集合。
///
/// 用于检查评估结果是否符合质量要求。
pub struct QualityValidators {
    gates: Vec<QualityGateConfig>,
}

impl QualityValidators {
    pub fn new() -> Self {
        Self { gates: Vec::new() }
    }

    /// 添加质量门禁。
    pub fn add_gate(&mut self, gate: QualityGateConfig) -> &mut Self {
        self.gates.push(gate);
        self
    }

    /// 执行所有验证。
    pub fn validate_all(&self, evaluation_data: &serde_json::Value) -> ValidationResult {
        let mut results = Vec::new();
        let mut all_passed = true;

        for gate in &self.gates {
            match gate.execute(evaluation_data) {
                Ok(result) => {
                    all_passed = all_passed && result.passed;
                    results.push(result.clone());
                }
                Err(error) => {
                    all_passed = false;
                    results.push(QualityGateResult {
                        gate_name: gate.gate_name.clone(),
                        passed: false,
                        score: 0.0,
                        threshold_met: Vec::new(),
                        threshold_failed: vec![error.clone()],
                        evaluation_details: Vec::new(),
                        action: gate.on_failure,
                        message: format!("门禁「{}」执行失败：{}", gate.gate_name, error),
                    });
                }
            }
        }

        ValidationResult {
            all_passed,
            gate_results: results.clone(),
            overall_message: if all_passed {
                "所有质量门禁检查通过".to_string()
            } else {
                let failed_gates: Vec<_> = results
                    .iter()
                    .filter(|r| !r.passed)
                    .map(|r| r.gate_name.clone())
                    .collect();
                format!("以下质量门禁未通过：{}", failed_gates.join(", "))
            },
        }
    }
}

/// 验证结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub all_passed: bool,
    pub gate_results: Vec<QualityGateResult>,
    pub overall_message: String,
}

impl Default for QualityValidators {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validators_all_pass() {
        let mut validators = QualityValidators::new();
        validators.add_gate(QualityGateConfig {
            gate_name: "gate1".to_string(),
            enabled: true,
            required_evaluations: vec![],
            thresholds: vec![],
            on_failure: crate::quality_gate::FailureAction::Block,
            allow_warnings: false,
        });

        let data = serde_json::json!({});
        let result = validators.validate_all(&data);

        assert!(result.all_passed);
        assert_eq!(result.gate_results.len(), 1);
    }

    #[test]
    fn test_validators_mixed() {
        let mut validators = QualityValidators::new();

        validators.add_gate(QualityGateConfig {
            gate_name: "pass_gate".to_string(),
            enabled: true,
            required_evaluations: vec![],
            thresholds: vec![],
            on_failure: crate::quality_gate::FailureAction::Block,
            allow_warnings: false,
        });

        validators.add_gate(QualityGateConfig {
            gate_name: "disabled_gate".to_string(),
            enabled: false,
            required_evaluations: vec![],
            thresholds: vec![],
            on_failure: crate::quality_gate::FailureAction::Block,
            allow_warnings: false,
        });

        let data = serde_json::json!({});
        let result = validators.validate_all(&data);

        assert!(result.all_passed);
        assert_eq!(result.gate_results.len(), 2);
    }
}
