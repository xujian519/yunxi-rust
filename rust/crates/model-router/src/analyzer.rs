use crate::{TaskFeatures, TaskType, UserInput};
use std::collections::HashSet;

pub struct TaskAnalyzer {
    planning_keywords: HashSet<&'static str>,
    analysis_keywords: HashSet<&'static str>,
    generation_keywords: HashSet<&'static str>,
    execution_keywords: HashSet<&'static str>,
    chat_keywords: HashSet<&'static str>,
}

impl TaskAnalyzer {
    pub fn new() -> Self {
        Self {
            planning_keywords: ["规划", "计划", "设计", "评估", "反思", "策略", "方案"]
                .iter()
                .cloned()
                .collect(),
            analysis_keywords: ["分析", "检查", "验证", "审查", "对比", "诊断"]
                .iter()
                .cloned()
                .collect(),
            generation_keywords: ["生成", "撰写", "创建", "起草", "编写", "构建"]
                .iter()
                .cloned()
                .collect(),
            execution_keywords: ["执行", "修改", "操作", "处理", "运行", "应用"]
                .iter()
                .cloned()
                .collect(),
            chat_keywords: ["聊天", "对话", "解释", "说明", "帮助", "咨询"]
                .iter()
                .cloned()
                .collect(),
        }
    }

    pub fn analyze(&self, input: &UserInput) -> TaskFeatures {
        let task_type = self.detect_task_type(&input.text);
        let input_length = input.text.len();
        let has_code = self.detect_code(&input.text);
        let has_structured_data = self.detect_structured_data(&input.text);

        TaskFeatures {
            task_type,
            input_length,
            has_code,
            has_structured_data,
            history_rounds: 0,
            files_involved: 0,
            estimated_tool_calls: 0,
            complex_tools_used: Vec::new(),
        }
    }

    fn detect_task_type(&self, text: &str) -> TaskType {
        if self.planning_keywords.iter().any(|kw| text.contains(kw)) {
            return TaskType::Planning;
        }
        if self.analysis_keywords.iter().any(|kw| text.contains(kw)) {
            return TaskType::Analysis;
        }
        if self.generation_keywords.iter().any(|kw| text.contains(kw)) {
            return TaskType::Generation;
        }
        if self.execution_keywords.iter().any(|kw| text.contains(kw)) {
            return TaskType::Execution;
        }
        if self.chat_keywords.iter().any(|kw| text.contains(kw)) {
            return TaskType::Chat;
        }
        TaskType::Unknown
    }

    fn detect_code(&self, text: &str) -> bool {
        text.contains("function")
            || text.contains("def ")
            || text.contains("class ")
            || text.contains("```")
            || text.contains("import ")
            || text.contains("from ")
    }

    fn detect_structured_data(&self, text: &str) -> bool {
        (text.contains('{') && text.contains('}'))
            || (text.contains('<') && text.contains('>'))
    }
}

impl Default for TaskAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_matching_planning() {
        let analyzer = TaskAnalyzer::new();
        let features = analyzer.analyze(&UserInput::new("帮我规划一下这个项目的架构"));
        assert_eq!(features.task_type, TaskType::Planning);
    }

    #[test]
    fn test_keyword_matching_analysis() {
        let analyzer = TaskAnalyzer::new();
        let features = analyzer.analyze(&UserInput::new("分析这个文件的逻辑"));
        assert_eq!(features.task_type, TaskType::Analysis);
    }

    #[test]
    fn test_input_length() {
        let analyzer = TaskAnalyzer::new();
        let long_input = "a".repeat(2000);
        let features = analyzer.analyze(&UserInput::new(&long_input));
        assert_eq!(features.input_length, 2000);
    }

    #[test]
    fn test_code_detection() {
        let analyzer = TaskAnalyzer::new();
        let input = "帮我修改这段代码: function test() { return 1; }";
        let features = analyzer.analyze(&UserInput::new(input));
        assert!(features.has_code);
    }

    #[test]
    fn test_structured_data_detection() {
        let analyzer = TaskAnalyzer::new();
        let input = "处理这个 JSON: {\"key\": \"value\"}";
        let features = analyzer.analyze(&UserInput::new(input));
        assert!(features.has_structured_data);
    }
}
