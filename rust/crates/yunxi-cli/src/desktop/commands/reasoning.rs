//! 推理引擎 IPC 命令。

use reasoning::{NoopReasoningExecutor, PipelineConfig, ReasoningPhase, ReasoningPipeline};

/// 执行结构化推理任务
#[tauri::command]
pub async fn run_reasoning(
    query: String,
    context: Option<String>,
    phases: Option<Vec<String>>,
    config: Option<serde_json::Value>,
) -> Result<String, String> {
    // 解析阶段配置
    let phase_list = if let Some(phases) = phases {
        phases
            .iter()
            .map(|p| match p.as_str() {
                "engagement" => Ok(ReasoningPhase::Engagement),
                "analysis" => Ok(ReasoningPhase::Analysis),
                "hypothesis" => Ok(ReasoningPhase::Hypothesis),
                "discovery" => Ok(ReasoningPhase::Discovery),
                "testing" => Ok(ReasoningPhase::Testing),
                "correction" => Ok(ReasoningPhase::Correction),
                _ => Err(format!("未知的推理阶段: {p}")),
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        // 默认使用所有阶段
        ReasoningPhase::all().to_vec()
    };

    // 解析配置
    let pipeline_config = if let Some(config) = config {
        serde_json::from_value::<PipelineConfig>(config)
            .map_err(|e| format!("配置解析失败: {e}"))?
    } else {
        PipelineConfig::default()
    };

    let pipeline = ReasoningPipeline::new(pipeline_config);
    let mut executor = NoopReasoningExecutor {
        model: "deepseek-v4-pro".to_string(),
    };

    let result = pipeline.execute(&query, &mut executor, None);

    serde_json::to_string(&result).map_err(|e| format!("结果序列化失败: {e}"))
}

/// 获取推理阶段列表
#[tauri::command]
pub fn list_reasoning_phases() -> Vec<String> {
    vec![
        "engagement".to_string(),
        "analysis".to_string(),
        "hypothesis".to_string(),
        "discovery".to_string(),
        "testing".to_string(),
        "correction".to_string(),
    ]
}

/// 获取推理管道配置
#[tauri::command]
pub fn get_pipeline_config() -> serde_json::Value {
    serde_json::to_value(PipelineConfig::default()).unwrap_or_else(|_| serde_json::Value::Null)
}
