//! 统一工具接口 trait
//!
//! 为 80+ 工具提供标准化的抽象接口，支持工具发现、输入校验和结果序列化。
//! 当前为 trait 定义 + 宏辅助，逐步将 `dispatch.rs` 的工具迁移到此模式。

use serde_json::Value;

/// 工具执行结果 — 替代裸 `Result<String, String>`
#[derive(Debug, Clone)]
pub struct ToolOutput {
    /// 主要输出内容（文本/JSON）
    pub content: String,
    /// 结构化数据（可选，供下游消费）
    pub data: Option<serde_json::Value>,
    /// 执行时长（毫秒）
    pub duration_ms: Option<u64>,
    /// 是否成功
    pub success: bool,
    /// 附加元数据
    pub metadata: std::collections::HashMap<String, String>,
}

impl ToolOutput {
    /// 成功的文本输出
    pub fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            data: None,
            duration_ms: None,
            success: true,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 成功的结构化输出
    pub fn ok_with_data(content: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            content: content.into(),
            data: Some(data),
            duration_ms: None,
            success: true,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 错误输出
    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            content: msg.into(),
            data: None,
            duration_ms: None,
            success: false,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 记录执行时长
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), val.into());
        self
    }

    /// 转换为 `Result<String, String>` 以兼容现有 API
    pub fn to_result(self) -> Result<String, String> {
        if self.success {
            Ok(self.content)
        } else {
            Err(self.content)
        }
    }
}

impl From<Result<String, String>> for ToolOutput {
    fn from(r: Result<String, String>) -> Self {
        match r {
            Ok(s) => Self::ok(s),
            Err(e) => Self::err(e),
        }
    }
}

/// 工具元数据 — 用于 LLM function calling 和工具发现
#[derive(Debug, Clone)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub category: ToolCategory,
}

/// 工具分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolCategory {
    System,
    File,
    Search,
    PatentCore,
    PatentAnalysis,
    PatentDrafting,
    PatentSearch,
    PatentQuality,
    Knowledge,
    Memory,
    Agent,
    Workflow,
    Other,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "系统"),
            Self::File => write!(f, "文件"),
            Self::Search => write!(f, "搜索"),
            Self::PatentCore => write!(f, "专利核心"),
            Self::PatentAnalysis => write!(f, "专利分析"),
            Self::PatentDrafting => write!(f, "专利撰写"),
            Self::PatentSearch => write!(f, "专利检索"),
            Self::PatentQuality => write!(f, "专利质量"),
            Self::Knowledge => write!(f, "知识库"),
            Self::Memory => write!(f, "记忆"),
            Self::Agent => write!(f, "智能体"),
            Self::Workflow => write!(f, "工作流"),
            Self::Other => write!(f, "其他"),
        }
    }
}

/// 统一工具接口 — 每个工具都应实现此 trait
pub trait Tool {
    /// 获取工具元数据
    fn spec(&self) -> ToolSpec;

    /// 执行工具
    fn execute(&self, input: &Value) -> Result<String, String>;

    /// 执行工具并返回结构化结果（可选覆盖）
    fn execute_typed(&self, input: &Value) -> ToolOutput {
        match self.execute(input) {
            Ok(s) => ToolOutput::ok(s),
            Err(e) => ToolOutput::err(e),
        }
    }

    /// 检查输入是否合法（可选覆盖，默认不做校验）
    fn validate(&self, _input: &Value) -> bool {
        true
    }
}

/// 工具注册表 — 管理所有已注册的工具
pub struct ToolRegistry {
    tools: std::collections::HashMap<String, Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }

    /// 注册一个工具
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let spec = tool.spec();
        self.tools.insert(spec.name, tool);
    }

    /// 获取所有工具元数据（用于 LLM function calling）
    pub fn list_specs(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|t| t.spec()).collect()
    }

    /// 按名称获取工具
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// 执行工具
    pub fn execute(&self, name: &str, input: &Value) -> Result<String, String> {
        match self.tools.get(name) {
            Some(tool) => tool.execute(input),
            None => Err(format!("Tool not found: {name}")),
        }
    }

    /// 工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

/// 快捷生成 ToolSpec 的助手
#[macro_export]
macro_rules! tool_spec {
    ($name:expr, $desc:expr, $category:expr, { $($json_key:expr => $json_val:expr),* $(,)? }) => {
        $crate::tool_trait::ToolSpec {
            name: $name.to_string(),
            description: $desc.to_string(),
            input_schema: serde_json::json!({
                $($json_key: $json_val),*
            }),
            category: $category,
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool;

    impl Tool for TestTool {
        fn spec(&self) -> ToolSpec {
            ToolSpec {
                name: "TestTool".to_string(),
                description: "A test tool for unit testing".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    },
                    "required": ["message"]
                }),
                category: ToolCategory::Other,
            }
        }

        fn execute(&self, input: &Value) -> Result<String, String> {
            let msg = input.get("message").and_then(|v| v.as_str()).unwrap_or("");
            Ok(format!("test: {msg}"))
        }
    }

    #[test]
    fn test_registry_register_and_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(TestTool));

        assert_eq!(registry.len(), 1);

        let specs = registry.list_specs();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].name, "TestTool");
        assert!(matches!(specs[0].category, ToolCategory::Other));

        let result = registry
            .execute("TestTool", &serde_json::json!({"message": "hello"}))
            .unwrap();
        assert_eq!(result, "test: hello");
    }

    #[test]
    fn test_registry_unknown_tool() {
        let registry = ToolRegistry::new();
        let result = registry.execute("UnknownTool", &serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_tool_spec_macro() {
        let spec = tool_spec!(
            "MyTool",
            "Description",
            ToolCategory::PatentCore,
            {
                "type" => "object",
                "properties" => "x"
            }
        );
        assert_eq!(spec.name, "MyTool");
        assert_eq!(spec.category, ToolCategory::PatentCore);
        assert!(spec.description.contains("Description"));
    }
}
