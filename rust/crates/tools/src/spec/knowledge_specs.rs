//! 知识库工具规格

use serde_json::json;

use super::types::{PermissionMode, ToolSpec};

pub(crate) fn knowledge_tool_specs() -> Vec<ToolSpec> {
    vec![
        ToolSpec {
            name: "KnowledgeSearch",
            description: "跨知识图谱、法律法规与知识卡片的统一检索。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "检索关键词" },
                    "limit": { "type": "integer", "minimum": 1, "default": 20 },
                    "search_kg": { "type": "boolean", "default": true },
                    "search_law": { "type": "boolean", "default": true },
                    "search_cards": { "type": "boolean", "default": true },
                    "min_card_quality": { "type": "number", "minimum": 0, "maximum": 1, "default": 0.5 },
                    "search_mode": {
                        "type": "string",
                        "enum": ["text", "semantic", "hybrid"],
                        "description": "检索模式；semantic/hybrid 需 semantic.enabled=true"
                    }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "LegalReasoning",
            description:
                "法律知识图谱结构化推理（新颖性三步法、创造性问题-解决方案法、侵权要素分析）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "发明描述或法律问题" },
                    "method": {
                        "type": "string",
                        "enum": ["novelty_three_step", "inventiveness_problem_solution", "infringement_elements"],
                        "default": "novelty_three_step"
                    },
                    "path_limit": { "type": "integer", "minimum": 1, "default": 5 }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "LawQuery",
            description: "查询法律法规数据库（按名称、正文、层级或列出全部层级）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "keyword": { "type": "string" },
                    "mode": {
                        "type": "string",
                        "enum": ["name", "content", "level", "levels"],
                        "default": "content"
                    },
                    "level": { "type": "string", "description": "mode=level 时必填" },
                    "limit": { "type": "integer", "minimum": 1, "default": 20 }
                },
                "required": ["keyword"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "KnowledgeCard",
            description: "检索专利知识卡片（按关键词与质量分筛选）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "keyword": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1, "default": 10 },
                    "min_quality": { "type": "number", "minimum": 0, "maximum": 1 },
                    "load_content": { "type": "boolean", "default": false }
                },
                "required": ["keyword"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SuperReasoningPlan",
            description:
                "生成 6 阶段专利法律结构化推理计划（Engagement→Correction），供后续工具逐步取证。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "problem": { "type": "string", "description": "待分析的法律/专利问题描述" },
                    "max_hypotheses": { "type": "integer", "minimum": 1, "default": 5 },
                    "max_iterations": { "type": "integer", "minimum": 1, "default": 3 }
                },
                "required": ["problem"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
    ]
}
