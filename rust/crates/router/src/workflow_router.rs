//! 工作流路由
//!
//! 根据领域和复杂度推荐工作流、工具和智能体。

use std::sync::Arc;

use crate::complexity::ComplexityAssessor;
use crate::config::RoutingConfig;
use crate::detector::DomainDetector;
use crate::hebbian_hint::HebbianPathHint;
use crate::types::{Complexity, Domain, RoutingDecision, WorkflowType};
use intent::IntentClassifier;

/// 工作流路由器
pub struct WorkflowRouter {
    config: RoutingConfig,
    detector: DomainDetector,
    assessor: ComplexityAssessor,
    /// Hebbian 路径提示（可选）
    hebbian: Option<Arc<dyn HebbianPathHint>>,
}

impl WorkflowRouter {
    pub fn new(config: RoutingConfig) -> Self {
        Self {
            config,
            detector: DomainDetector::new(),
            assessor: ComplexityAssessor::new(),
            hebbian: None,
        }
    }

    /// 注入 Hebbian 路径提示（builder 风格）。
    pub fn with_hebbian(mut self, hint: Arc<dyn HebbianPathHint>) -> Self {
        self.hebbian = Some(hint);
        self
    }

    /// 对用户输入进行完整路由决策
    pub fn route(&self, input: &str) -> RoutingDecision {
        if !self.config.enabled {
            return RoutingDecision {
                domain: Domain::General,
                complexity: Complexity::Simple,
                workflow: WorkflowType::Direct,
                suggested_tools: vec![],
                suggested_agents: vec![],
                confidence: 0.0,
                reasoning: "路由已禁用".into(),
                intent_name: "GENERAL".into(),
                intent_confidence: 0.0,
            };
        }

        let (domain, domain_confidence) = self.detector.detect(input);
        let complexity = self.assessor.assess(input);
        let intent_hint = IntentClassifier::from_user_settings().classify(input);

        let (mut suggested_tools, suggested_agents) = self.suggest_resources(domain, complexity);
        Self::augment_tools_for_intent(&mut suggested_tools, intent_hint.intent.to_athena_name());
        if embedding::semantic_enabled() {
            Self::augment_semantic_tools(&mut suggested_tools);
        }

        // Hebbian 增强：基于已有工具列表的协同调用模式
        if let Some(ref hebbian) = self.hebbian {
            for tool in suggested_tools.clone() {
                if let Some(extra) = hebbian.optimal_path(&tool) {
                    for (suggested, strength) in extra {
                        if strength >= 0.3 && !suggested_tools.iter().any(|t| t == &suggested) {
                            suggested_tools.push(suggested);
                        }
                    }
                }
            }
        }

        let workflow = match complexity {
            Complexity::Simple => WorkflowType::Direct,
            Complexity::Medium => WorkflowType::Hitl,
            Complexity::Complex => WorkflowType::PlanPlusHitl,
        };

        let reasoning = format!(
            "检测到{}领域任务，复杂度为{}，意图≈{}（{:.0}%），建议{}工作流",
            domain,
            complexity,
            intent_hint.intent.to_athena_name(),
            intent_hint.confidence * 100.0,
            match workflow {
                WorkflowType::Direct => "直接执行",
                WorkflowType::Hitl => "人机协同",
                WorkflowType::PlanPlusHitl => "规划+人机协同",
            }
        );

        RoutingDecision {
            domain,
            complexity,
            workflow,
            suggested_tools,
            suggested_agents,
            confidence: domain_confidence,
            reasoning,
            intent_name: intent_hint.intent.to_athena_name().to_string(),
            intent_confidence: intent_hint.confidence,
        }
    }

    fn augment_semantic_tools(tools: &mut Vec<String>) {
        for name in [
            "KnowledgeSearch",
            "SemanticCompare",
            "LegalReasoning",
            "SuperReasoningPlan",
        ] {
            if !tools.iter().any(|t| t == name) {
                tools.push(name.into());
            }
        }
    }

    fn augment_tools_for_intent(tools: &mut Vec<String>, intent_name: &str) {
        let extra: &[&str] = match intent_name {
            // 检索 — 排除含 SEARCH 但非检索意图的（如 CASE_SEARCH_INVALIDATION）
            n if n.contains("SEARCH") && !n.contains("INVALIDATION") => {
                &["IterativeSearch", "SynonymSearch"]
            }
            // 撰写
            n if n.contains("DRAFT") => &["FormalCheck", "QualityAssess"],
            // 新颖性 / 创造性
            n if n.contains("NOVELTY") | n.contains("CREATIVITY") => {
                &["SemanticCompare", "LegalReasoning", "NoveltyAnalysis"]
            }
            // 侵权
            n if n.contains("INFRINGEMENT")
                | n.contains("EQUIVALENT")
                | n.contains("ALL_ELEMENTS")
                | n.contains("DOCTRINE")
                | n.contains("ESTOPPEL")
                | n.contains("LITERAL_INTERPRETATION") =>
            {
                &["SemanticCompare", "LegalReasoning", "InfringementAnalysis"]
            }
            // 无效宣告（含 CASE_SEARCH_INVALIDATION）
            n if n.contains("INVALID") | n.contains("INVALIDATION") => &[
                "SemanticCompare",
                "LegalReasoning",
                "InvalidDecision",
                "RulesEngine",
            ],
            // 审查意见 / 答辩
            n if n.contains("OPINION") | n.contains("ARGUMENT") => {
                &["OaParse", "OaStrategy", "ResponseTemplate"]
            }
            // 权利要求解释 / 形式
            n if n.contains("SCOPE")
                | n.contains("DEFINITION")
                | n.contains("SPECIFICATION_SUPPORT")
                | n.contains("SUPPORT_DISCLOSURE")
                | n.contains("CLAIM_AMENDMENT") =>
            {
                &["ClaimParse", "FormalCheck", "KnowledgeSearch"]
            }
            // 审查标准 / 审查指南
            n if n.contains("EXAMINATION")
                | n.contains("GUIDELINE")
                | n.contains("RULE_INTERPRETATION")
                | n.contains("SECTION_LOOKUP") =>
            {
                &["KnowledgeSearch", "LegalReasoning", "LawQuery"]
            }
            // 法律分析 / 证据
            n if n.contains("LEGAL")
                | n.contains("JUDGMENT")
                | n.contains("EVIDENCE")
                | n.contains("DEFENSE")
                | n.contains("ADDED_SUBJECT") =>
            {
                &["LegalReasoning", "LawQuery", "KnowledgeSearch"]
            }
            _ => &["KnowledgeSearch", "LegalReasoning"],
        };
        for name in extra {
            if !tools.iter().any(|t| t == name) {
                tools.push((*name).into());
            }
        }
    }

    fn suggest_resources(
        &self,
        domain: Domain,
        complexity: Complexity,
    ) -> (Vec<String>, Vec<String>) {
        // 优先使用用户自定义配置
        if let Some((tools, agents)) = self.config.resolve_resources(domain, complexity) {
            return (tools.clone(), agents.clone());
        }

        // 回退到内置规则
        Self::default_resources(domain, complexity)
    }

    fn default_resources(domain: Domain, complexity: Complexity) -> (Vec<String>, Vec<String>) {
        let mut tools = Vec::new();
        let mut agents = Vec::new();

        match domain {
            Domain::Patent => {
                tools.extend_from_slice(&[
                    "ClaimParse".into(),
                    "KnowledgeGraphQuery".into(),
                    "KnowledgeSearch".into(),
                    "LegalReasoning".into(),
                ]);
                if complexity == Complexity::Complex || complexity == Complexity::Medium {
                    tools.extend_from_slice(&[
                        "ClaimCompare".into(),
                        "NoveltyAnalysis".into(),
                        "InventivenessAnalysis".into(),
                        "SemanticCompare".into(),
                    ]);
                    agents.push("patent-analysis-agent".into());
                }
                if complexity == Complexity::Complex {
                    tools.extend_from_slice(&[
                        "QualityAssess".into(),
                        "FormalCheck".into(),
                        "SuperReasoningPlan".into(),
                    ]);
                    agents.push("patent-drafting-agent".into());
                }
            }
            Domain::Trademark => {
                tools.extend_from_slice(&[
                    "KnowledgeSearch".into(),
                    "TrademarkAnalysis".into(),
                    "TrademarkSimilarity".into(),
                    "TrademarkConflictAnalysis".into(),
                ]);
                agents.push("trademark-agent".into());
            }
            Domain::Copyright => {
                tools.push("KnowledgeSearch".into());
            }
            Domain::Legal => {
                tools.push("LawQuery".into());
                tools.push("KnowledgeSearch".into());
                agents.push("legal-agent".into());
            }
            Domain::General => {}
        }

        (tools, agents)
    }
}

impl Default for WorkflowRouter {
    fn default() -> Self {
        Self::new(RoutingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_patent_analysis() {
        let router = WorkflowRouter::default();
        let decision = router.route("帮我分析这个专利的新颖性");
        assert_eq!(decision.domain, Domain::Patent);
        assert_eq!(decision.complexity, Complexity::Medium);
        assert!(!decision.suggested_tools.is_empty());
    }

    #[test]
    fn test_route_patent_writing() {
        let router = WorkflowRouter::default();
        let decision = router.route("撰写专利申请文件");
        assert_eq!(decision.domain, Domain::Patent);
        assert_eq!(decision.complexity, Complexity::Complex);
        assert!(decision
            .suggested_agents
            .contains(&"patent-drafting-agent".to_string()));
    }

    #[test]
    fn test_route_general() {
        let router = WorkflowRouter::default();
        let decision = router.route("今天天气怎么样");
        assert_eq!(decision.domain, Domain::General);
    }

    #[test]
    fn test_route_disabled() {
        let router = WorkflowRouter::new(RoutingConfig {
            enabled: false,
            ..Default::default()
        });
        let decision = router.route("分析这个专利");
        assert_eq!(decision.domain, Domain::General);
        assert_eq!(decision.confidence, 0.0);
    }
}
