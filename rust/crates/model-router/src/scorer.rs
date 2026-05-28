use crate::{ComplexityScore, TaskFeatures, TaskType};

const COMPLEX_TOOLS: &[&str] = &["python", "bash", "search", "web_fetch", "agent"];

pub struct ComplexityScorer {
    threshold: u8,
}

impl ComplexityScorer {
    pub fn new() -> Self {
        Self { threshold: 65 }
    }

    pub fn score(&self, features: &TaskFeatures) -> ComplexityScore {
        let task_type_score = self.score_task_type(&features.task_type);
        let input_score = self.score_input(features);
        let context_score = self.score_context(features);
        let tools_score = self.score_tools(features);

        let total = task_type_score + input_score + context_score + tools_score;

        ComplexityScore {
            total,
            task_type_score,
            input_score,
            context_score,
            tools_score,
        }
    }

    fn score_task_type(&self, task_type: &TaskType) -> u8 {
        match task_type {
            TaskType::Planning => 35,
            TaskType::Analysis => 30,
            TaskType::Generation => 25,
            TaskType::Execution => 15,
            TaskType::Chat => 10,
            TaskType::Unknown => 5,
        }
    }

    fn score_input(&self, features: &TaskFeatures) -> u8 {
        let length_score = (features.input_length / 100).min(10) as u8;
        let code_bonus = if features.has_code { 5 } else { 0 };
        let data_bonus = if features.has_structured_data { 5 } else { 0 };
        (length_score + code_bonus + data_bonus).min(20)
    }

    fn score_context(&self, features: &TaskFeatures) -> u8 {
        let history_score = features.history_rounds.min(10) as u8;
        let files_score = (features.files_involved * 2).min(10) as u8;
        (history_score + files_score).min(20)
    }

    fn score_tools(&self, features: &TaskFeatures) -> u8 {
        let base_score = features.estimated_tool_calls.min(15) as u8;
        let complex_bonus = features
            .complex_tools_used
            .iter()
            .filter(|tool| COMPLEX_TOOLS.contains(&tool.as_str()))
            .count() as u8
            * 5;
        (base_score + complex_bonus).min(20)
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }
}

impl Default for ComplexityScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planning_task_high_score() {
        let scorer = ComplexityScorer::new();
        let features = TaskFeatures {
            task_type: TaskType::Planning,
            input_length: 1000,
            has_code: true,
            has_structured_data: true,
            history_rounds: 10,
            files_involved: 5,
            estimated_tool_calls: 10,
            complex_tools_used: vec!["python".to_string(), "bash".to_string()],
        };
        let score = scorer.score(&features);
        assert!(score.total >= 65);
    }

    #[test]
    fn test_chat_task_low_score() {
        let scorer = ComplexityScorer::new();
        let features = TaskFeatures {
            task_type: TaskType::Chat,
            input_length: 50,
            has_code: false,
            has_structured_data: false,
            history_rounds: 0,
            files_involved: 0,
            estimated_tool_calls: 0,
            complex_tools_used: vec![],
        };
        let score = scorer.score(&features);
        assert!(score.total < 65);
    }
}
