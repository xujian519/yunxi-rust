//! 统一工具执行 trait
//!
//! 为 dispatch.rs 的 80+ 工具提供标准化的执行抽象接口。
//! 与 `spec::ToolSpec`（编译期元数据）互补，添加运行期执行和校验能力。

use serde_json::Value;

/// 工具操作类别
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

/// 统一工具执行接口 — 所有工具应实现此 trait
///
/// 示例实现：
/// ```ignore
/// struct ClaimParseTool;
/// impl ToolExecutable for ClaimParseTool {
///     fn execute(&self, input: &Value) -> Result<String, String> {
///         // 解析并执行
///     }
/// }
/// ```
pub trait ToolExecutable: Send + Sync {
    /// 执行工具并返回字符串结果
    fn execute(&self, input: &Value) -> Result<String, String>;

    /// 工具名称（用于日志和调度）
    fn name(&self) -> &'static str;

    /// 工具类别
    fn category(&self) -> ToolCategory {
        ToolCategory::Other
    }

    /// 校验输入（默认为 true，子类可覆盖）
    fn validate(&self, _input: &Value) -> bool {
        true
    }

    /// 获取工具的 MCP 兼容 inputSchema
    fn input_schema(&self) -> Option<Value> {
        None
    }
}

/// 可执行工具注册表
pub struct ExecutableToolRegistry {
    tools: std::collections::HashMap<&'static str, Box<dyn ToolExecutable>>,
}

impl Default for ExecutableToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutableToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
        }
    }

    /// 注册一个可执行工具
    pub fn register(&mut self, tool: Box<dyn ToolExecutable>) {
        let name = tool.name();
        self.tools.insert(name, tool);
    }

    /// 按名称执行工具
    pub fn execute(&self, name: &str, input: &Value) -> Result<String, String> {
        match self.tools.get(name) {
            Some(tool) => tool.execute(input),
            None => Err(format!("Tool not found: {name}")),
        }
    }

    /// 按类别获取工具名称列表
    pub fn list_by_category(&self, category: ToolCategory) -> Vec<&'static str> {
        self.tools
            .values()
            .filter(|t| t.category() == category)
            .map(|t| t.name())
            .collect()
    }

    /// 工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// 列出所有工具名称
    pub fn names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEchoTool;

    impl ToolExecutable for TestEchoTool {
        fn execute(&self, input: &Value) -> Result<String, String> {
            let msg = input.get("msg").and_then(|v| v.as_str()).unwrap_or("");
            Ok(format!("echo: {msg}"))
        }

        fn name(&self) -> &'static str {
            "test_echo"
        }

        fn category(&self) -> ToolCategory {
            ToolCategory::Other
        }
    }

    #[test]
    fn test_registry_register_and_execute() {
        let mut registry = ExecutableToolRegistry::new();
        registry.register(Box::new(TestEchoTool));

        assert_eq!(registry.len(), 1);

        let result = registry
            .execute("test_echo", &serde_json::json!({"msg": "hello"}))
            .unwrap();
        assert_eq!(result, "echo: hello");
    }

    #[test]
    fn test_registry_unknown_tool() {
        let registry = ExecutableToolRegistry::new();
        let result = registry.execute("unknown", &serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_list_by_category() {
        let mut registry = ExecutableToolRegistry::new();
        registry.register(Box::new(TestEchoTool));

        let others = registry.list_by_category(ToolCategory::Other);
        assert_eq!(others, vec!["test_echo"]);

        let patents = registry.list_by_category(ToolCategory::PatentCore);
        assert!(patents.is_empty());
    }
}
