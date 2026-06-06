use super::types::YunXiError;
use serde::{Deserialize, Serialize};
use std::backtrace::Backtrace;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub error: YunXiError,
    pub backtrace: Option<String>,
    pub report_id: String,
}

impl ErrorReport {
    pub fn new(error: YunXiError) -> Self {
        let report_id = uuid::Uuid::new_v4().to_string();
        Self {
            error,
            backtrace: Some(Backtrace::force_capture().to_string()),
            report_id,
        }
    }

    pub fn without_backtrace(error: YunXiError) -> Self {
        let report_id = uuid::Uuid::new_v4().to_string();
        Self {
            error,
            backtrace: None,
            report_id,
        }
    }

    pub fn generate_text_report(&self) -> String {
        let mut report = String::new();

        report.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        report.push_str("║                       错误报告                               ║\n");
        report.push_str("╚══════════════════════════════════════════════════════════════╝\n");
        report.push('\n');

        report.push_str(&format!("报告ID: {}\n", self.report_id));
        report.push_str(&format!("时间: {:?}\n", self.error.timestamp));
        report.push_str(&format!("级别: {}\n", self.error.level));
        report.push_str(&format!("类型: {}\n", self.error.error_type));
        report.push_str(&format!("消息: {}\n", self.error.message));
        report.push('\n');

        if !self.error.context.is_empty() {
            report.push_str("上下文:\n");
            for (i, ctx) in self.error.context.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, ctx));
            }
            report.push('\n');
        }

        if !self.error.suggestions.is_empty() {
            report.push_str("建议解决方案:\n");
            for suggestion in &self.error.suggestions {
                report.push_str(&format!("  • {}\n", suggestion));
            }
            report.push('\n');
        }

        if let Some(ref bt) = self.backtrace {
            report.push_str("堆栈跟踪:\n");
            report.push_str(bt);
            report.push('\n');
        }

        report.push_str("────────────────────────────────────────────────────────────\n");

        report
    }

    pub fn generate_json_report(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self)
    }

    pub fn generate_markdown_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# 错误报告\n\n");
        report.push_str(&format!("**报告ID**: `{}`\n\n", self.report_id));
        report.push_str(&format!("**时间**: `{:?}`\n\n", self.error.timestamp));
        report.push_str(&format!("**级别**: `{}`\n\n", self.error.level));
        report.push_str(&format!("**类型**: `{}`\n\n", self.error.error_type));
        report.push_str(&format!("**消息**: {}\n\n", self.error.message));

        if !self.error.context.is_empty() {
            report.push_str("## 上下文\n\n");
            for (i, ctx) in self.error.context.iter().enumerate() {
                report.push_str(&format!("{}. {}\n", i + 1, ctx));
            }
            report.push('\n');
        }

        if !self.error.suggestions.is_empty() {
            report.push_str("## 建议解决方案\n\n");
            for suggestion in &self.error.suggestions {
                report.push_str(&format!("* {}\n", suggestion));
            }
            report.push('\n');
        }

        if let Some(ref bt) = self.backtrace {
            report.push_str("## 堆栈跟踪\n\n");
            report.push_str("```\n");
            report.push_str(bt);
            report.push_str("```\n\n");
        }

        report
    }
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.generate_text_report())
    }
}

pub struct ErrorReporter {
    enabled: bool,
    collect_backtrace: bool,
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            collect_backtrace: true,
        }
    }

    pub fn with_backtrace(mut self, enabled: bool) -> Self {
        self.collect_backtrace = enabled;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn report(&self, error: YunXiError) -> Option<ErrorReport> {
        if !self.enabled {
            return None;
        }

        let report = if self.collect_backtrace {
            ErrorReport::new(error)
        } else {
            ErrorReport::without_backtrace(error)
        };

        Some(report)
    }

    pub fn report_and_print(&self, error: YunXiError) -> Option<ErrorReport> {
        if let Some(report) = self.report(error) {
            eprintln!("{}", report.generate_text_report());
            Some(report)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::error::types::{ErrorLevel, ErrorType};

    #[test]
    fn test_error_report_creation() {
        let error = YunXiError::io("测试错误");
        let report = ErrorReport::new(error);
        assert!(!report.report_id.is_empty());
        assert!(report.backtrace.is_some());
    }

    #[test]
    fn test_error_report_without_backtrace() {
        let error = YunXiError::io("测试错误");
        let report = ErrorReport::without_backtrace(error);
        assert!(!report.report_id.is_empty());
        assert!(report.backtrace.is_none());
    }

    #[test]
    fn test_error_report_text_generation() {
        let error = YunXiError::io("文件未找到")
            .with_context("读取配置")
            .with_suggestion("检查文件路径");
        let report = ErrorReport::new(error);
        let text = report.generate_text_report();
        assert!(text.contains("错误报告"));
        assert!(text.contains("报告ID"));
        assert!(text.contains("文件未找到"));
        assert!(text.contains("读取配置"));
        assert!(text.contains("建议解决方案"));
    }

    #[test]
    fn test_error_report_json_generation() {
        let error = YunXiError::io("测试错误");
        let report = ErrorReport::new(error);
        let json = report.generate_json_report();
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("report_id"));
        assert!(json_str.contains("error"));
    }

    #[test]
    fn test_error_report_markdown_generation() {
        let error = YunXiError::io("文件未找到")
            .with_context("读取配置")
            .with_suggestion("检查文件路径");
        let report = ErrorReport::new(error);
        let markdown = report.generate_markdown_report();
        assert!(markdown.contains("# 错误报告"));
        assert!(markdown.contains("## 上下文"));
        assert!(markdown.contains("## 建议解决方案"));
        assert!(markdown.contains("文件未找到"));
    }

    #[test]
    fn test_error_report_display() {
        let error = YunXiError::io("测试错误");
        let report = ErrorReport::new(error);
        let display = format!("{}", report);
        assert!(display.contains("错误报告"));
        assert!(display.contains("测试错误"));
    }

    #[test]
    fn test_error_reporter_default() {
        let reporter = ErrorReporter::default();
        assert!(reporter.enabled);
        assert!(reporter.collect_backtrace);
    }

    #[test]
    fn test_error_reporter_custom() {
        let reporter = ErrorReporter::new()
            .with_backtrace(false)
            .with_enabled(false);

        assert!(!reporter.enabled);
        assert!(!reporter.collect_backtrace);
    }

    #[test]
    fn test_error_reporter_report() {
        let reporter = ErrorReporter::new();
        let error = YunXiError::io("测试错误");
        let report = reporter.report(error);
        assert!(report.is_some());
        assert!(report.unwrap().backtrace.is_some());
    }

    #[test]
    fn test_error_reporter_report_disabled() {
        let reporter = ErrorReporter::new().with_enabled(false);
        let error = YunXiError::io("测试错误");
        let report = reporter.report(error);
        assert!(report.is_none());
    }

    #[test]
    fn test_error_reporter_without_backtrace() {
        let reporter = ErrorReporter::new().with_backtrace(false);
        let error = YunXiError::io("测试错误");
        let report = reporter.report(error);
        assert!(report.is_some());
        assert!(report.unwrap().backtrace.is_none());
    }

    #[test]
    fn test_error_reporter_report_and_print() {
        let reporter = ErrorReporter::new();
        let error = YunXiError::io("测试错误");
        let report = reporter.report_and_print(error);
        assert!(report.is_some());
    }

    #[test]
    fn test_error_report_with_multiple_contexts() {
        let error = YunXiError::io("测试错误")
            .with_context("上下文1")
            .with_context("上下文2")
            .with_context("上下文3");
        let report = ErrorReport::new(error);
        let text = report.generate_text_report();
        assert!(text.contains("上下文1"));
        assert!(text.contains("上下文2"));
        assert!(text.contains("上下文3"));
    }

    #[test]
    fn test_error_report_with_multiple_suggestions() {
        let error = YunXiError::validation("输入错误")
            .with_suggestion("建议1")
            .with_suggestion("建议2")
            .with_suggestion("建议3");
        let report = ErrorReport::new(error);
        let text = report.generate_text_report();
        assert!(text.contains("建议1"));
        assert!(text.contains("建议2"));
        assert!(text.contains("建议3"));
    }

    #[test]
    fn test_error_report_different_levels() {
        for level in [
            ErrorLevel::Info,
            ErrorLevel::Warning,
            ErrorLevel::Error,
            ErrorLevel::Fatal,
        ] {
            let error = YunXiError::new(ErrorType::Runtime("测试".to_string()), level, "测试消息");
            let report = ErrorReport::new(error);
            let text = report.generate_text_report();
            assert!(text.contains(&level.to_string()));
        }
    }

    #[test]
    fn test_error_report_clone() {
        let error = YunXiError::io("测试错误")
            .with_context("上下文")
            .with_suggestion("建议");
        let report = ErrorReport::new(error);
        let cloned = report.clone();
        assert_eq!(report.report_id, cloned.report_id);
        assert_eq!(report.error.message, cloned.error.message);
    }
}
