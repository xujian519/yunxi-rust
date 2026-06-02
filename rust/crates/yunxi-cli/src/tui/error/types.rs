use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ErrorType {
    IO(String),
    Network(String),
    Parse(String),
    Validation(String),
    Permission(String),
    Runtime(String),
    User(String),
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::IO(msg) => write!(f, "IO错误: {}", msg),
            ErrorType::Network(msg) => write!(f, "网络错误: {}", msg),
            ErrorType::Parse(msg) => write!(f, "解析错误: {}", msg),
            ErrorType::Validation(msg) => write!(f, "验证错误: {}", msg),
            ErrorType::Permission(msg) => write!(f, "权限错误: {}", msg),
            ErrorType::Runtime(msg) => write!(f, "运行时错误: {}", msg),
            ErrorType::User(msg) => write!(f, "用户错误: {}", msg),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorLevel {
    Info,
    Warning,
    Error,
    Fatal,
}

impl fmt::Display for ErrorLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorLevel::Info => write!(f, "信息"),
            ErrorLevel::Warning => write!(f, "警告"),
            ErrorLevel::Error => write!(f, "错误"),
            ErrorLevel::Fatal => write!(f, "致命"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YunXiError {
    pub error_type: ErrorType,
    pub level: ErrorLevel,
    pub message: String,
    pub context: Vec<String>,
    #[serde(default = "YunXiError::default_timestamp")]
    pub timestamp: SystemTime,
    pub suggestions: Vec<String>,
}

impl YunXiError {
    pub fn new(error_type: ErrorType, level: ErrorLevel, message: impl Into<String>) -> Self {
        Self {
            error_type,
            level,
            message: message.into(),
            context: Vec::new(),
            timestamp: SystemTime::now(),
            suggestions: Vec::new(),
        }
    }

    fn default_timestamp() -> SystemTime {
        SystemTime::UNIX_EPOCH
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn io(message: impl Into<String>) -> Self {
        let msg_str = message.into();
        Self::new(ErrorType::IO(msg_str.clone()), ErrorLevel::Error, msg_str)
    }

    pub fn network(message: impl Into<String>) -> Self {
        let msg_str = message.into();
        Self::new(
            ErrorType::Network(msg_str.clone()),
            ErrorLevel::Error,
            msg_str,
        )
    }

    pub fn parse(message: impl Into<String>) -> Self {
        let msg_str = message.into();
        Self::new(
            ErrorType::Parse(msg_str.clone()),
            ErrorLevel::Error,
            msg_str,
        )
    }

    pub fn validation(message: impl Into<String>) -> Self {
        let msg_str = message.into();
        Self::new(
            ErrorType::Validation(msg_str.clone()),
            ErrorLevel::Warning,
            msg_str,
        )
    }

    pub fn permission(message: impl Into<String>) -> Self {
        let msg_str = message.into();
        Self::new(
            ErrorType::Permission(msg_str.clone()),
            ErrorLevel::Fatal,
            msg_str,
        )
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        let msg_str = message.into();
        Self::new(
            ErrorType::Runtime(msg_str.clone()),
            ErrorLevel::Error,
            msg_str,
        )
    }

    pub fn user(message: impl Into<String>) -> Self {
        let msg_str = message.into();
        Self::new(
            ErrorType::User(msg_str.clone()),
            ErrorLevel::Warning,
            msg_str,
        )
    }
}

impl fmt::Display for YunXiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.level, self.error_type, self.message)
    }
}

impl std::error::Error for YunXiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_display() {
        let io_error = ErrorType::IO("文件未找到".to_string());
        assert_eq!(io_error.to_string(), "IO错误: 文件未找到");

        let network_error = ErrorType::Network("连接超时".to_string());
        assert_eq!(network_error.to_string(), "网络错误: 连接超时");
    }

    #[test]
    fn test_error_level_display() {
        assert_eq!(ErrorLevel::Info.to_string(), "信息");
        assert_eq!(ErrorLevel::Warning.to_string(), "警告");
        assert_eq!(ErrorLevel::Error.to_string(), "错误");
        assert_eq!(ErrorLevel::Fatal.to_string(), "致命");
    }

    #[test]
    fn test_error_level_ordering() {
        assert!(ErrorLevel::Info < ErrorLevel::Warning);
        assert!(ErrorLevel::Warning < ErrorLevel::Error);
        assert!(ErrorLevel::Error < ErrorLevel::Fatal);
    }

    #[test]
    fn test_yunxi_error_creation() {
        let error = YunXiError::new(
            ErrorType::IO("测试错误".to_string()),
            ErrorLevel::Error,
            "测试消息",
        );
        assert_eq!(error.message, "测试消息");
        assert_eq!(error.level, ErrorLevel::Error);
        assert!(error.context.is_empty());
        assert!(error.suggestions.is_empty());
    }

    #[test]
    fn test_yunxi_error_with_context() {
        let error = YunXiError::io("文件未找到")
            .with_context("尝试读取配置文件")
            .with_context("路径: /path/to/config");
        assert_eq!(error.context.len(), 2);
        assert_eq!(error.context[0], "尝试读取配置文件");
        assert_eq!(error.context[1], "路径: /path/to/config");
    }

    #[test]
    fn test_yunxi_error_with_suggestion() {
        let error = YunXiError::validation("输入无效")
            .with_suggestion("请检查输入格式")
            .with_suggestion("参考文档中的示例");
        assert_eq!(error.suggestions.len(), 2);
        assert_eq!(error.suggestions[0], "请检查输入格式");
        assert_eq!(error.suggestions[1], "参考文档中的示例");
    }

    #[test]
    fn test_yunxi_error_factory_methods() {
        let io_error = YunXiError::io("IO错误");
        assert!(matches!(io_error.error_type, ErrorType::IO(_)));

        let network_error = YunXiError::network("网络错误");
        assert!(matches!(network_error.error_type, ErrorType::Network(_)));

        let parse_error = YunXiError::parse("解析错误");
        assert!(matches!(parse_error.error_type, ErrorType::Parse(_)));

        let validation_error = YunXiError::validation("验证错误");
        assert!(matches!(
            validation_error.error_type,
            ErrorType::Validation(_)
        ));
        assert_eq!(validation_error.level, ErrorLevel::Warning);

        let permission_error = YunXiError::permission("权限错误");
        assert!(matches!(
            permission_error.error_type,
            ErrorType::Permission(_)
        ));
        assert_eq!(permission_error.level, ErrorLevel::Fatal);

        let runtime_error = YunXiError::runtime("运行时错误");
        assert!(matches!(runtime_error.error_type, ErrorType::Runtime(_)));

        let user_error = YunXiError::user("用户错误");
        assert!(matches!(user_error.error_type, ErrorType::User(_)));
        assert_eq!(user_error.level, ErrorLevel::Warning);
    }

    #[test]
    fn test_yunxi_error_display() {
        let error = YunXiError::io("文件未找到");
        let display = error.to_string();
        assert!(display.contains("错误"));
        assert!(display.contains("IO错误"));
        assert!(display.contains("文件未找到"));
    }

    #[test]
    fn test_yunxi_error_timestamp() {
        let before = SystemTime::now();
        let error = YunXiError::io("测试");
        let after = SystemTime::now();
        assert!(error.timestamp >= before);
        assert!(error.timestamp <= after);
    }

    #[test]
    fn test_yunxi_error_clone() {
        let error = YunXiError::io("测试")
            .with_context("上下文")
            .with_suggestion("建议");
        let cloned = error.clone();
        assert_eq!(error.message, cloned.message);
        assert_eq!(error.context, cloned.context);
        assert_eq!(error.suggestions, cloned.suggestions);
    }

    #[test]
    fn test_error_type_equality() {
        let error1 = ErrorType::IO("错误1".to_string());
        let error2 = ErrorType::IO("错误1".to_string());
        let error3 = ErrorType::IO("错误2".to_string());

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_error_level_equality() {
        assert_eq!(ErrorLevel::Info, ErrorLevel::Info);
        assert_ne!(ErrorLevel::Info, ErrorLevel::Warning);
    }

    #[test]
    fn test_yunxi_error_debug() {
        let error = YunXiError::io("测试错误");
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("IO"));
        assert!(debug_output.contains("测试错误"));
    }
}
