use crate::eval_framework::Evaluator;
use serde::Serialize;
use serde_json::Value;

/// 评估流水线。
///
/// 支持多个评估器的串行/并行执行。
pub struct EvalPipeline {
    stages: Vec<PipelineStage>,
}

impl EvalPipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// 添加评估阶段。
    pub fn add_stage(&mut self, stage: PipelineStage) -> &mut Self {
        self.stages.push(stage);
        self
    }

    /// 执行流水线。
    pub fn execute(&self, input: &Value, context: &Value) -> Result<PipelineResult, String> {
        let mut results = Vec::new();
        let mut current_input = input.clone();

        for stage in &self.stages {
            match &stage.execution_mode {
                ExecutionMode::Sequential => {
                    let result = self.execute_stage(stage, &current_input, context)?;
                    results.push(result.clone());
                    current_input = result.data;
                }
                ExecutionMode::Parallel => {
                    let result = self.execute_stage(stage, input, context)?;
                    results.push(result);
                }
            }
        }

        Ok(PipelineResult {
            stage_results: results.clone(),
            aggregated_data: self.aggregate_results(&results),
            overall_success: results.iter().all(|r| r.success),
        })
    }

    fn execute_stage(
        &self,
        stage: &PipelineStage,
        input: &Value,
        context: &Value,
    ) -> Result<StageResult, String> {
        let start_time = std::time::Instant::now();
        let evaluator = &stage.evaluator;

        let result = match evaluator.evaluate(input, context) {
            Ok(data) => StageResult {
                stage_name: stage.name.clone(),
                success: true,
                data,
                error: None,
                duration_ms: start_time.elapsed().as_millis() as u64,
            },
            Err(error) => StageResult {
                stage_name: stage.name.clone(),
                success: false,
                data: Value::Null,
                error: Some(error),
                duration_ms: start_time.elapsed().as_millis() as u64,
            },
        };

        Ok(result)
    }

    fn aggregate_results(&self, results: &[StageResult]) -> Value {
        let mut aggregated = serde_json::json!({
            "stageCount": results.len(),
            "successCount": results.iter().filter(|r| r.success).count(),
            "failCount": results.iter().filter(|r| !r.success).count(),
            "totalDurationMs": results.iter().map(|r| r.duration_ms).sum::<u64>(),
        });

        // 收集所有数据
        let mut all_data = Vec::new();
        for result in results {
            if result.success {
                all_data.push(result.data.clone());
            }
        }
        if let Some(obj) = aggregated.as_object_mut() {
            obj.insert("stageData".to_string(), Value::Array(all_data));
        }

        aggregated
    }
}

/// 流水线阶段。
pub struct PipelineStage {
    pub name: String,
    pub evaluator: Box<dyn Evaluator>,
    pub execution_mode: ExecutionMode,
}

/// 执行模式。
#[derive(Debug, Clone, Copy)]
pub enum ExecutionMode {
    /// 串行执行（后一阶段依赖前一阶段输出）
    Sequential,
    /// 并行执行（所有阶段使用相同输入）
    Parallel,
}

/// 阶段结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageResult {
    pub stage_name: String,
    pub success: bool,
    pub data: Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// 流水线结果。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineResult {
    pub stage_results: Vec<StageResult>,
    pub aggregated_data: Value,
    pub overall_success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEvaluator {
        name: String,
        should_succeed: bool,
    }

    impl Evaluator for MockEvaluator {
        fn evaluate(&self, _input: &Value, _context: &Value) -> Result<Value, String> {
            if self.should_succeed {
                Ok(serde_json::json!({
                    "result": "mock_output",
                    "evaluator": self.name
                }))
            } else {
                Err("mock_error".to_string())
            }
        }

        fn evaluator_name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_pipeline_sequential() {
        let mut pipeline = EvalPipeline::new();
        pipeline.add_stage(PipelineStage {
            name: "stage1".to_string(),
            evaluator: Box::new(MockEvaluator {
                name: "eval1".to_string(),
                should_succeed: true,
            }),
            execution_mode: ExecutionMode::Sequential,
        });

        let input = serde_json::json!({"test": "data"});
        let result = pipeline.execute(&input, &serde_json::json!({}));

        assert!(result.is_ok());
        let pipeline_result = result.unwrap();
        assert!(pipeline_result.overall_success);
        assert_eq!(pipeline_result.stage_results.len(), 1);
    }

    #[test]
    fn test_pipeline_parallel() {
        let mut pipeline = EvalPipeline::new();
        pipeline.add_stage(PipelineStage {
            name: "stage1".to_string(),
            evaluator: Box::new(MockEvaluator {
                name: "eval1".to_string(),
                should_succeed: true,
            }),
            execution_mode: ExecutionMode::Parallel,
        });

        let input = serde_json::json!({"test": "data"});
        let result = pipeline.execute(&input, &serde_json::json!({}));

        assert!(result.is_ok());
        let pipeline_result = result.unwrap();
        assert!(pipeline_result.overall_success);
    }
}
