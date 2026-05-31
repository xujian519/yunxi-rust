//! 专业代理角色定义
//!
//! 基于 Athena 的 9 个子智能体重写。
//! 每个角色指定：允许工具、系统提示模板、输出格式。

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// 专利领域专业代理角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Retriever,
    Analyzer,
    Writer,
    NoveltyChecker,
    CreativityChecker,
    InfringementChecker,
    InvalidityChecker,
    Reviewer,
    QualityChecker,
}

impl AgentRole {
    /// 获取角色的系统提示词
    pub fn system_prompt(&self) -> String {
        let role_id = self.role_id();
        let cargo_manifest = env!("CARGO_MANIFEST_DIR");
        let xml_path = std::path::PathBuf::from(cargo_manifest)
            .join("../../../assets/agents")
            .join(format!("{role_id}.xml"));

        if let Ok(content) = std::fs::read_to_string(&xml_path) {
            return content;
        }
        // 回退到硬编码（兼容旧版无 XML 文件的情况）
        match self {
            Self::Retriever => "你是一个专利检索专家。你的任务是检索相关专利、现有技术和文献。使用可用的搜索工具查找最相关的结果。",
            Self::Analyzer => "你是一个专利分析专家。你的任务是深入分析专利文件，提取关键技术特征、技术方案和发明要点。",
            Self::Writer => "你是一个专利撰写专家。你的任务是撰写专利申请文件，包括说明书、权利要求书和摘要。",
            Self::NoveltyChecker => "你是一个专利新颖性评估专家。使用新颖性三步法（确定最接近现有技术→识别区别特征→评估技术效果）分析专利的新颖性。",
            Self::CreativityChecker => "你是一个专利创造性评估专家。使用问题-解决方案法分析专利的创造性。",
            Self::InfringementChecker => "你是一个专利侵权分析专家。使用全面覆盖原则和等同原则进行侵权分析。",
            Self::InvalidityChecker => "你是一个专利无效分析专家。分析专利的无效理由和证据。",
            Self::Reviewer => "你是一个专利文件审查专家。审查专利申请文件的格式规范和内容质量。",
            Self::QualityChecker => "你是一个专利质量评估专家。从多个维度评估专利文件的撰写质量。",
        }.to_string()
    }

    /// 获取角色的 XML 文件标识符（snake_case）
    fn role_id(&self) -> &str {
        match self {
            Self::Retriever => "retriever",
            Self::Analyzer => "analyzer",
            Self::Writer => "writer",
            Self::NoveltyChecker => "novelty_checker",
            Self::CreativityChecker => "creativity_checker",
            Self::InfringementChecker => "infringement_checker",
            Self::InvalidityChecker => "invalidity_checker",
            Self::Reviewer => "reviewer",
            Self::QualityChecker => "quality_checker",
        }
    }

    /// 获取角色允许使用的工具
    pub fn allowed_tools(&self) -> BTreeSet<String> {
        let base = vec![
            "read_file",
            "glob_search",
            "grep_search",
            "TodoWrite",
            "StructuredOutput",
            "SendUserMessage",
        ];

        let domain_tools = match self {
            Self::Retriever => vec![
                "PatentSearch",
                "GooglePatentsFetch",
                "IterativeSearch",
                "SynonymSearch",
                "KnowledgeSearch",
                "WebSearch",
                "WebFetch",
            ],
            Self::Analyzer => vec![
                "SemanticCompare",
                "ClaimParse",
                "KnowledgeSearch",
                "WebFetch",
            ],
            Self::Writer => vec![
                "ClaimParse",
                "OaStrategy",
                "read_file",
                "write_file",
                "edit_file",
            ],
            Self::NoveltyChecker => vec![
                "ClaimCompare",
                "NoveltyAnalysis",
                "SemanticCompare",
                "KnowledgeSearch",
            ],
            Self::CreativityChecker => vec![
                "ClaimCompare",
                "InventivenessAnalysis",
                "SemanticCompare",
                "KnowledgeSearch",
            ],
            Self::InfringementChecker => vec!["ClaimParse", "SemanticCompare", "KnowledgeSearch"],
            Self::InvalidityChecker => vec!["ClaimCompare", "SemanticCompare", "KnowledgeSearch"],
            Self::Reviewer => vec!["FormalCheck", "ClaimParse", "KnowledgeSearch"],
            Self::QualityChecker => vec!["QualityAssess", "FormalCheck", "ClaimParse"],
        };

        let mut tools: BTreeSet<String> = base.into_iter().map(String::from).collect();
        for t in domain_tools {
            tools.insert(t.to_string());
        }
        tools
    }

    /// 获取角色名称
    pub fn name(&self) -> &str {
        match self {
            Self::Retriever => "检索专家",
            Self::Analyzer => "分析专家",
            Self::Writer => "撰写专家",
            Self::NoveltyChecker => "新颖性评估专家",
            Self::CreativityChecker => "创造性评估专家",
            Self::InfringementChecker => "侵权分析专家",
            Self::InvalidityChecker => "无效分析专家",
            Self::Reviewer => "文件审查专家",
            Self::QualityChecker => "质量评估专家",
        }
    }

    /// 获取角色推荐模型
    pub fn preferred_model(&self) -> &'static str {
        match self {
            Self::NoveltyChecker
            | Self::CreativityChecker
            | Self::InfringementChecker
            | Self::InvalidityChecker
            | Self::Analyzer => "deepseek-v4-pro",
            Self::Writer | Self::Reviewer | Self::QualityChecker => "deepseek-v4-pro",
            Self::Retriever => "deepseek-v4-pro",
        }
    }

    /// 路由上下文中的角色简短描述（单一真相源）
    pub fn routing_hint(&self) -> &'static str {
        match self {
            Self::Retriever => "专利检索(使用PatentSearch、GooglePatentsFetch等)",
            Self::Analyzer => "专利分析(使用SemanticCompare、ClaimParse等)",
            Self::Writer => "专利撰写(使用ClaimParse、OaStrategy等)",
            Self::NoveltyChecker => "新颖性评估(使用NoveltyAnalysis、ClaimCompare等)",
            Self::CreativityChecker => "创造性评估(使用InventivenessAnalysis等)",
            Self::InfringementChecker => "侵权分析(使用ClaimParse、SemanticCompare等)",
            Self::InvalidityChecker => "无效分析(使用ClaimCompare、SemanticCompare等)",
            Self::Reviewer => "文件审查(使用FormalCheck、ClaimParse等)",
            Self::QualityChecker => "质量评估(使用QualityAssess、FormalCheck等)",
        }
    }

    /// 从路由推荐的 agent 名称解析角色（如 "patent-analysis-agent" → Analyzer）
    pub fn from_agent_name(name: &str) -> Option<Self> {
        match name {
            "patent-analysis-agent" => Some(Self::Analyzer),
            "patent-drafting-agent" => Some(Self::Writer),
            "patent-retrieval-agent" | "patent-search-agent" => Some(Self::Retriever),
            "patent-novelty-agent" => Some(Self::NoveltyChecker),
            "patent-creativity-agent" => Some(Self::CreativityChecker),
            "patent-infringement-agent" => Some(Self::InfringementChecker),
            "patent-invalidity-agent" => Some(Self::InvalidityChecker),
            "patent-review-agent" => Some(Self::Reviewer),
            "patent-quality-agent" => Some(Self::QualityChecker),
            "trademark-agent" => Some(Self::Analyzer),
            "legal-agent" => Some(Self::Analyzer),
            _ => None,
        }
    }

    /// 从字符串解析角色
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "retriever" => Some(Self::Retriever),
            "analyzer" => Some(Self::Analyzer),
            "writer" => Some(Self::Writer),
            "novelty" | "novelty_checker" | "noveltychecker" => Some(Self::NoveltyChecker),
            "creativity" | "creativity_checker" | "creativitychecker" => {
                Some(Self::CreativityChecker)
            }
            "infringement" | "infringement_checker" | "infringementchecker" => {
                Some(Self::InfringementChecker)
            }
            "invalidity" | "invalidity_checker" | "invaliditychecker" => {
                Some(Self::InvalidityChecker)
            }
            "reviewer" => Some(Self::Reviewer),
            "quality" | "quality_checker" | "qualitychecker" => Some(Self::QualityChecker),
            _ => None,
        }
    }

    /// 获取所有角色
    pub fn all() -> &'static [AgentRole] {
        &[
            Self::Retriever,
            Self::Analyzer,
            Self::Writer,
            Self::NoveltyChecker,
            Self::CreativityChecker,
            Self::InfringementChecker,
            Self::InvalidityChecker,
            Self::Reviewer,
            Self::QualityChecker,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_roles_have_prompts() {
        for role in AgentRole::all() {
            assert!(!role.system_prompt().is_empty());
            assert!(!role.name().is_empty());
            assert!(!role.allowed_tools().is_empty());
        }
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(
            AgentRole::from_str_opt("Retriever"),
            Some(AgentRole::Retriever)
        );
        assert_eq!(
            AgentRole::from_str_opt("NoveltyChecker"),
            Some(AgentRole::NoveltyChecker)
        );
        assert_eq!(AgentRole::from_str_opt("unknown"), None);
    }

    #[test]
    fn test_all_roles_have_preferred_model() {
        for role in AgentRole::all() {
            assert!(
                !role.preferred_model().is_empty(),
                "{} should have a preferred_model",
                role.name()
            );
        }
    }

    #[test]
    fn test_retriever_has_search_tools() {
        let tools = AgentRole::Retriever.allowed_tools();
        assert!(tools.contains("PatentSearch"));
        assert!(tools.contains("KnowledgeSearch"));
    }

    #[test]
    fn test_novelty_checker_has_analysis_tools() {
        let tools = AgentRole::NoveltyChecker.allowed_tools();
        assert!(tools.contains("NoveltyAnalysis"));
        assert!(tools.contains("SemanticCompare"));
    }
}
