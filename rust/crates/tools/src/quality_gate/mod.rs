use serde::{Deserialize, Serialize};
use serde_json::Value;

mod thresholds;
mod validators;

/// 质量门禁配置。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityGateConfig {
    pub gate_name: String,
    pub enabled: bool,
    pub required_evaluations: Vec<String>,
    pub thresholds: Vec<Threshold>,
    pub on_failure: FailureAction,
    pub allow_warnings: bool,
}

impl QualityGateConfig {
    pub fn new(gate_name: String) -> Self {
        Self {
            gate_name,
            enabled: true,
            required_evaluations: Vec::new(),
            thresholds: Vec::new(),
            on_failure: FailureAction::Block,
            allow_warnings: false,
        }
    }
}

/// 阈值规则。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Threshold {
    pub metric_name: String,
    pub operator: ThresholdOperator,
    pub value: f64,
}

/// 阈值操作符。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThresholdOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// 失败处理方式。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureAction {
    Block,
    Warn,
    Continue,
}

/// 质量门禁结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityGateResult {
    pub gate_name: String,
    pub passed: bool,
    pub score: f64,
    pub threshold_met: Vec<String>,
    pub threshold_failed: Vec<String>,
    pub evaluation_details: Vec<EvaluationDetail>,
    pub action: FailureAction,
    pub message: String,
}

/// 评估详情。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationDetail {
    pub evaluator_name: String,
    pub score: f64,
    pub passed: bool,
    pub threshold_value: f64,
    pub operator: ThresholdOperator,
}

impl QualityGateConfig {
    /// 执行质量门禁检查。
    pub fn execute(&self, evaluation_data: &Value) -> Result<QualityGateResult, String> {
        if !self.enabled {
            return Ok(QualityGateResult {
                gate_name: self.gate_name.clone(),
                passed: true,
                score: 100.0,
                threshold_met: Vec::new(),
                threshold_failed: Vec::new(),
                evaluation_details: Vec::new(),
                action: FailureAction::Continue,
                message: "质量门禁已禁用，跳过检查".to_string(),
            });
        }

        let mut threshold_met = Vec::new();
        let mut threshold_failed = Vec::new();
        let mut evaluation_details = Vec::new();
        let mut total_score = 0.0;
        let mut count = 0;

        for threshold in &self.thresholds {
            let metric_value =
                self.extract_metric_value(evaluation_data, &threshold.metric_name)?;
            let passed = self.check_threshold(&metric_value, threshold);

            evaluation_details.push(EvaluationDetail {
                evaluator_name: threshold.metric_name.clone(),
                score: metric_value,
                passed,
                threshold_value: threshold.value,
                operator: threshold.operator,
            });

            total_score += metric_value;
            count += 1;

            if passed {
                threshold_met.push(format!(
                    "{} {} {}",
                    threshold.metric_name,
                    Self::operator_to_string(&threshold.operator),
                    threshold.value
                ));
            } else {
                threshold_failed.push(format!(
                    "{} {} {} (实际值: {})",
                    threshold.metric_name,
                    Self::operator_to_string(&threshold.operator),
                    threshold.value,
                    metric_value
                ));
            }
        }

        let avg_score = if count > 0 {
            total_score / count as f64
        } else {
            100.0
        };

        let all_passed = threshold_failed.is_empty();
        let action = if all_passed {
            FailureAction::Continue
        } else {
            self.on_failure
        };

        let message = if all_passed {
            format!(
                "质量门禁「{}」检查通过，平均分: {:.1}",
                self.gate_name, avg_score
            )
        } else {
            format!(
                "质量门禁「{}」检查失败，未满足 {} 个阈值，平均分: {:.1}",
                self.gate_name,
                threshold_failed.len(),
                avg_score
            )
        };

        Ok(QualityGateResult {
            gate_name: self.gate_name.clone(),
            passed: all_passed,
            score: avg_score,
            threshold_met,
            threshold_failed,
            evaluation_details,
            action,
            message,
        })
    }

    fn extract_metric_value(&self, data: &Value, metric_name: &str) -> Result<f64, String> {
        let parts: Vec<&str> = metric_name.split('.').collect();

        let mut current = data;
        for (i, part) in parts.iter().enumerate() {
            if let Some(obj) = current.as_object() {
                current = obj
                    .get(*part)
                    .ok_or_else(|| format!("无法找到字段「{}」（第 {} 部分）", part, i + 1))?;
            } else if let Some(arr) = current.as_array() {
                let index: usize = part
                    .parse()
                    .map_err(|_| format!("无法解析索引「{}」为数字", part))?;
                current = arr
                    .get(index)
                    .ok_or_else(|| format!("数组索引 {} 超出范围", index))?;
            } else {
                return Err(format!("字段「{}」不是对象或数组", part));
            }
        }

        current
            .as_f64()
            .ok_or_else(|| format!("字段「{}」的值不是数字", metric_name))
    }

    fn check_threshold(&self, metric_value: &f64, threshold: &Threshold) -> bool {
        match threshold.operator {
            ThresholdOperator::GreaterThan => metric_value > &threshold.value,
            ThresholdOperator::GreaterThanOrEqual => metric_value >= &threshold.value,
            ThresholdOperator::LessThan => metric_value < &threshold.value,
            ThresholdOperator::LessThanOrEqual => metric_value <= &threshold.value,
            ThresholdOperator::Equal => (metric_value - threshold.value).abs() < 0.001,
            ThresholdOperator::NotEqual => (metric_value - threshold.value).abs() >= 0.001,
        }
    }

    fn operator_to_string(op: &ThresholdOperator) -> &'static str {
        match op {
            ThresholdOperator::GreaterThan => ">",
            ThresholdOperator::GreaterThanOrEqual => ">=",
            ThresholdOperator::LessThan => "<",
            ThresholdOperator::LessThanOrEqual => "<=",
            ThresholdOperator::Equal => "=",
            ThresholdOperator::NotEqual => "!=",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_gate_pass() {
        let config = QualityGateConfig {
            gate_name: "test_gate".to_string(),
            enabled: true,
            required_evaluations: vec![],
            thresholds: vec![Threshold {
                metric_name: "overall_quality".to_string(),
                operator: ThresholdOperator::GreaterThanOrEqual,
                value: 75.0,
            }],
            on_failure: FailureAction::Block,
            allow_warnings: false,
        };

        let data = serde_json::json!({
            "overall_quality": 85.0
        });

        let result = config.execute(&data).unwrap();

        assert!(result.passed);
        assert_eq!(result.score, 85.0);
        assert_eq!(result.threshold_met.len(), 1);
        assert_eq!(result.threshold_failed.len(), 0);
    }

    #[test]
    fn test_quality_gate_fail() {
        let config = QualityGateConfig {
            gate_name: "test_gate".to_string(),
            enabled: true,
            required_evaluations: vec![],
            thresholds: vec![Threshold {
                metric_name: "overall_quality".to_string(),
                operator: ThresholdOperator::GreaterThanOrEqual,
                value: 90.0,
            }],
            on_failure: FailureAction::Block,
            allow_warnings: false,
        };

        let data = serde_json::json!({
            "overall_quality": 75.0
        });

        let result = config.execute(&data).unwrap();

        assert!(!result.passed);
        assert_eq!(result.action, FailureAction::Block);
        assert_eq!(result.threshold_failed.len(), 1);
    }

    #[test]
    fn test_quality_gate_disabled() {
        let config = QualityGateConfig {
            gate_name: "test_gate".to_string(),
            enabled: false,
            required_evaluations: vec![],
            thresholds: vec![],
            on_failure: FailureAction::Block,
            allow_warnings: false,
        };

        let data = serde_json::json!({});

        let result = config.execute(&data).unwrap();

        assert!(result.passed);
        assert_eq!(result.action, FailureAction::Continue);
        assert!(result.message.contains("已禁用"));
    }
}
