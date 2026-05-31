use serde::Serialize;
use serde_json::Value;
use std::time::SystemTime;

/// 评估追踪记录。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalTrace {
    pub trace_id: String,
    pub evaluator_type: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_ms: Option<u64>,
    pub input_summary: String,
    pub output_summary: Option<String>,
    pub error_summary: Option<String>,
}

impl EvalTrace {
    pub fn start(evaluator_type: &str, input: &Value) -> Self {
        let trace_id = format!(
            "{}-{}",
            evaluator_type,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let input_summary = Self::summarize_value(input);

        Self {
            trace_id,
            evaluator_type: evaluator_type.to_string(),
            start_time: Self::current_timestamp(),
            end_time: None,
            duration_ms: None,
            input_summary,
            output_summary: None,
            error_summary: None,
        }
    }

    pub fn complete(mut self, output: &Value) -> Self {
        self.end_time = Some(Self::current_timestamp());
        self.duration_ms = Some(self.calculate_duration());
        self.output_summary = Some(Self::summarize_value(output));
        self
    }

    pub fn fail(mut self, error: &str) -> Self {
        self.end_time = Some(Self::current_timestamp());
        self.duration_ms = Some(self.calculate_duration());
        self.error_summary = Some(error.to_string());
        self
    }

    fn current_timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        format!("{:.3}", now.as_secs_f64())
    }

    fn calculate_duration(&self) -> u64 {
        let start: f64 = self.start_time.parse().unwrap_or(0.0);
        let end: f64 = self.end_time.as_ref().unwrap().parse().unwrap_or(0.0);
        ((end - start) * 1000.0) as u64
    }

    fn summarize_value(value: &Value) -> String {
        let json_str = serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string());
        if json_str.len() > 200 {
            format!("{}...", &json_str[..200])
        } else {
            json_str
        }
    }
}

/// 追踪存储（用于持久化）。
#[derive(Debug)]
pub struct TraceStore {
    traces: Vec<EvalTrace>,
}

impl TraceStore {
    pub fn new() -> Self {
        Self { traces: Vec::new() }
    }

    pub fn add_trace(&mut self, trace: EvalTrace) {
        self.traces.push(trace);
    }

    pub fn get_traces(&self, evaluator_type: Option<&str>) -> Vec<EvalTrace> {
        if let Some(eval_type) = evaluator_type {
            self.traces
                .iter()
                .filter(|t| t.evaluator_type == eval_type)
                .cloned()
                .collect()
        } else {
            self.traces.clone()
        }
    }

    pub fn get_statistics(&self) -> TraceStatistics {
        let total = self.traces.len();
        let successful = self
            .traces
            .iter()
            .filter(|t| t.error_summary.is_none())
            .count();
        let failed = total - successful;

        let avg_duration: f64 = self
            .traces
            .iter()
            .filter_map(|t| t.duration_ms)
            .map(|d| d as f64)
            .sum::<f64>()
            .max(1.0)
            / self
                .traces
                .iter()
                .filter(|t| t.duration_ms.is_some())
                .count()
                .max(1) as f64;

        TraceStatistics {
            total_evaluations: total,
            successful_evaluations: successful,
            failed_evaluations: failed,
            success_rate: if total > 0 {
                successful as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            average_duration_ms: avg_duration,
        }
    }
}

impl Default for TraceStore {
    fn default() -> Self {
        Self::new()
    }
}

/// 追踪统计。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceStatistics {
    pub total_evaluations: usize,
    pub successful_evaluations: usize,
    pub failed_evaluations: usize,
    pub success_rate: f64,
    pub average_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_start_complete() {
        let input = serde_json::json!({"test": "data"});
        let output = serde_json::json!({"result": "success"});

        let trace = EvalTrace::start("test_evaluator", &input);
        let completed = trace.complete(&output);

        assert!(completed.end_time.is_some());
        assert!(completed.output_summary.is_some());
        assert!(completed.error_summary.is_none());
        assert!(completed.duration_ms.is_some());
    }

    #[test]
    fn test_trace_fail() {
        let input = serde_json::json!({"test": "data"});
        let trace = EvalTrace::start("test_evaluator", &input);
        let failed = trace.fail("test error");

        assert!(failed.end_time.is_some());
        assert!(failed.error_summary.is_some());
        assert_eq!(failed.error_summary.as_deref(), Some("test error"));
        assert!(failed.duration_ms.is_some());
    }

    #[test]
    fn test_trace_store() {
        let mut store = TraceStore::new();

        let mut trace1 = EvalTrace::start("eval1", &serde_json::json!({}));
        let mut trace2 = EvalTrace::start("eval2", &serde_json::json!({}));

        // 标记为成功完成
        trace1 = trace1.complete(&serde_json::json!({}));
        trace2 = trace2.complete(&serde_json::json!({}));

        store.add_trace(trace1);
        store.add_trace(trace2);

        let all_traces = store.get_traces(None);
        assert_eq!(all_traces.len(), 2);

        let eval1_traces = store.get_traces(Some("eval1"));
        assert_eq!(eval1_traces.len(), 1);

        let stats = store.get_statistics();
        assert_eq!(stats.total_evaluations, 2);
        assert_eq!(stats.successful_evaluations, 2); // traces are completed successfully
        assert_eq!(stats.failed_evaluations, 0);
    }
}
