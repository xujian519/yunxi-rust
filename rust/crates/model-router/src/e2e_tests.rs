#[cfg(test)]
mod integration_tests {
    use crate::{ModelSelector, TaskContext, UserInput};

    #[test]
    fn test_e2e_simple_task() {
        let selector = ModelSelector::new();
        let ctx = TaskContext::new(UserInput::new("帮我修改这个文件"));
        let selection = selector.select_model(&ctx).unwrap();
        assert_eq!(selection.model, "deepseek-v4-flash");
    }

    #[test]
    fn test_e2e_complex_task() {
        let selector = ModelSelector::new();
        let long_text = "设计一个完整的系统架构。".repeat(30);
        let ctx = TaskContext::new(UserInput::new(format!(
            "{long_text}\n\n关键代码: function init() {{ return {{\"mode\": \"production\"}}; }}"
        )))
        .with_history(10)
        .with_files(5);
        let selection = selector.select_model(&ctx).unwrap();
        assert_eq!(selection.model, "deepseek-v4-pro");
    }

    #[test]
    fn test_e2e_code_generation() {
        let selector = ModelSelector::new();
        let long_text = "规划这个项目的架构设计并评估技术风险。".repeat(30);
        let ctx = TaskContext::new(UserInput::new(format!(
            "{long_text}\n\nclass AuthService {{ init() {{}} }}"
        )))
        .with_history(10)
        .with_files(5);
        let selection = selector.select_model(&ctx).unwrap();
        assert_eq!(selection.model, "deepseek-v4-pro");
    }

    #[test]
    fn test_e2e_with_history() {
        let selector = ModelSelector::new();
        let ctx = TaskContext::new(UserInput::new("继续上面的对话")).with_history(10);
        let selection = selector.select_model(&ctx).unwrap();
        assert!(selection.score.context_score > 0);
    }
}

#[cfg(test)]
mod bench_tests {
    use crate::{ModelSelector, TaskContext, UserInput};
    use std::time::Instant;

    #[test]
    fn benchmark_scoring() {
        let selector = ModelSelector::new();
        let ctx = TaskContext::new(UserInput::new("这是一个测试输入"));

        let start = Instant::now();
        for _ in 0..1000 {
            selector.select_model(&ctx).unwrap();
        }
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 10_000,
            "评分耗时应在10ms以内，实际耗时: {}ms",
            duration.as_millis()
        );
    }
}
