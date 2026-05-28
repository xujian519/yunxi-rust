use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    Planning,
    Analysis,
    Generation,
    Execution,
    Chat,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFeatures {
    pub task_type: TaskType,
    pub input_length: usize,
    pub has_code: bool,
    pub has_structured_data: bool,
    pub history_rounds: usize,
    pub files_involved: usize,
    pub estimated_tool_calls: usize,
    pub complex_tools_used: Vec<String>,
}

impl Default for TaskFeatures {
    fn default() -> Self {
        Self {
            task_type: TaskType::Unknown,
            input_length: 0,
            has_code: false,
            has_structured_data: false,
            history_rounds: 0,
            files_involved: 0,
            estimated_tool_calls: 0,
            complex_tools_used: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityScore {
    pub total: u8,
    pub task_type_score: u8,
    pub input_score: u8,
    pub context_score: u8,
    pub tools_score: u8,
}

impl ComplexityScore {
    pub fn zero() -> Self {
        Self {
            total: 0,
            task_type_score: 0,
            input_score: 0,
            context_score: 0,
            tools_score: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSelection {
    pub model: String,
    pub score: ComplexityScore,
    pub reason: String,
    pub forced: bool,
}

#[derive(Debug, Clone)]
pub struct UserInput {
    pub text: String,
}

impl UserInput {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone)]
pub struct TaskContext {
    pub user_input: UserInput,
    pub history_rounds: usize,
    pub files_involved: usize,
}

impl TaskContext {
    pub fn new(user_input: UserInput) -> Self {
        Self {
            user_input,
            history_rounds: 0,
            files_involved: 0,
        }
    }

    pub fn with_history(mut self, rounds: usize) -> Self {
        self.history_rounds = rounds;
        self
    }

    pub fn with_files(mut self, count: usize) -> Self {
        self.files_involved = count;
        self
    }
}

#[derive(Debug, Clone)]
pub enum RouterError {
    ConfigLoadError(String),
    ParseError(String),
    ScoreError(String),
    InvalidModel(String),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::ConfigLoadError(msg) => write!(f, "配置加载失败: {}", msg),
            RouterError::ParseError(msg) => write!(f, "解析错误: {}", msg),
            RouterError::ScoreError(msg) => write!(f, "评分错误: {}", msg),
            RouterError::InvalidModel(msg) => write!(f, "无效模型: {}", msg),
        }
    }
}

impl std::error::Error for RouterError {}
