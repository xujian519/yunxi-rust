//! Patent-domain tool specifications: claim analysis, novelty, OA, drafting,
//! knowledge graph, document parsing, search, infringement, and lifecycle management.

use serde_json::json;

use super::types::{PermissionMode, ToolSpec};

#[allow(clippy::too_many_lines)]
pub(crate) fn patent_tool_specs() -> Vec<ToolSpec> {
    vec![
        // --- 专利专用工具 ---
        ToolSpec {
            name: "ClaimParse",
            description: "Parse a patent claim text into structured AST (preamble, transition, features, component extraction).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claim_text": { "type": "string", "description": "Full text of the patent claim" },
                    "claim_number": { "type": "integer", "description": "Claim number (optional, defaults to 1)" }
                },
                "required": ["claim_text"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "ClaimCompare",
            description: "Compare two patent claims and output similarity score and correspondence type.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claim_a": { "type": "string", "description": "First claim text" },
                    "claim_b": { "type": "string", "description": "Second claim text" }
                },
                "required": ["claim_a", "claim_b"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "PatentCompare",
            description: "专利非 LLM 对比工具。提供特征矩阵构建、结构化 diff、IPC class 级分类。模式：diff（默认，完整对比）、matrix（仅特征矩阵）、ipc（IPC 分类）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["diff", "matrix", "ipc"],
                        "description": "操作模式：diff（完整对比）、matrix（仅特征矩阵）、ipc（IPC 分类）",
                        "default": "diff"
                    },
                    "target": {
                        "type": "object",
                        "description": "目标发明文档",
                        "properties": {
                            "title": { "type": "string" },
                            "abstractText": { "type": "string" },
                            "claims": { "type": "array", "items": { "type": "string" } },
                            "ipcCodes": { "type": "array", "items": { "type": "string" } },
                            "features": { "type": "array", "items": { "type": "object" } }
                        }
                    },
                    "priorArt": {
                        "type": "object",
                        "description": "现有技术文档",
                        "properties": {
                            "title": { "type": "string" },
                            "abstractText": { "type": "string" },
                            "claims": { "type": "array", "items": { "type": "string" } },
                            "ipcCodes": { "type": "array", "items": { "type": "string" } },
                            "features": { "type": "array", "items": { "type": "object" } }
                        }
                    },
                    "text": {
                        "type": "string",
                        "description": "待分类文本（ipc 模式）"
                    }
                }
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "NoveltyAnalysis",
            description: "Analyze whether an invention possesses novelty based on prior art and distinguishing features.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "invention_description": { "type": "string" },
                    "prior_art_descriptions": { "type": "array", "items": { "type": "string" } },
                    "differences": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["invention_description"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "InventivenessAnalysis",
            description: "Analyze whether an invention possesses inventive step (non-obviousness).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "invention_description": { "type": "string" },
                    "technical_effect": { "type": "string" },
                    "performance_improvement": { "type": "number" },
                    "obviousness": { "type": "boolean" }
                },
                "required": ["invention_description", "technical_effect"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "OaStrategy",
            description: "Suggest office-action response strategy based on rejection type and distinguishing features.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "rejection_type": { "type": "string", "enum": ["novelty", "inventiveness", "utility", "clarity", "support", "unity"] },
                    "differences": { "type": "array", "items": { "type": "string" } },
                    "technical_effects": { "type": "array", "items": { "type": "string" } },
                    "prior_art_different_field": { "type": "boolean" }
                },
                "required": ["rejection_type"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "FormalCheck",
            description: "Perform formal examination on patent claims and specification sections.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claims": { "type": "array", "items": { "type": "string" } },
                    "specification_sections": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["claims"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "QualityAssess",
            description: "Assess overall patent quality score across 7 dimensions.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claims": { "type": "array", "items": { "type": "string" } },
                    "specification_word_count": { "type": "integer" }
                },
                "required": ["claims"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "KnowledgeGraphQuery",
            description: "Query the patent knowledge graph. Uses SQLite DB (~40K nodes) if available, otherwise falls back to JSON files. Supports full-text search via FTS5.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search keyword or phrase" },
                    "source": { "type": "string", "enum": ["guideline", "legal", "all"], "description": "Which graph to query (fallback JSON only)" },
                    "limit": { "type": "integer", "default": 10, "description": "Max results to return" },
                    "node_type": { "type": "string", "description": "Filter by node type (e.g. GuidelineRule, Case, SupremeCourtJudgment)" },
                    "graph_dir": { "type": "string", "description": "Optional path to knowledge_graph directory (fallback JSON)" },
                    "sqlite_path": { "type": "string", "description": "Optional path to patent_kg.db SQLite file. Defaults to ~/.openclaw/workspace/memory/patent-knowledge-graph/patent_kg.db" }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "QualityScorer",
            description: "多维专利质量评分工具。对权利要求、说明书、附图进行12条规则检查，输出完整性、清晰性、一致性、可执行性四维评分及百分位排名。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claims": { "type": "array", "items": { "type": "object" }, "description": "权利要求列表，每项含 type/number/content/depends_on" },
                    "specification": { "type": "object", "description": "说明书对象：technical_field/background_art/invention_content/embodiment" },
                    "patent_type": { "type": "string", "default": "invention" },
                    "invention_title": { "type": "string" },
                    "drawings": { "type": "array", "items": { "type": "object" } },
                    "check_level": { "type": "integer", "default": 2 }
                },
                "required": ["claims"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "QualityChecker",
            description: "专利质量检查工具。基于规则引擎检查权利要求和说明书的质量问题，输出问题列表和修改建议。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claims": { "type": "array", "items": { "type": "object" }, "description": "权利要求列表" },
                    "specification": { "type": "object", "description": "说明书对象" },
                    "patent_type": { "type": "string" },
                    "invention_title": { "type": "string" },
                    "check_level": { "type": "integer", "default": 2 }
                },
                "required": ["claims", "specification", "patent_type"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "ClaimFormalityCheck",
            description: "权利要求形式检查器。检查权利要求的清楚性、简要性、非必要技术特征。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claims": { "type": "array", "items": { "type": "object" }, "description": "权利要求列表，每项含 claim_number/full_text" }
                },
                "required": ["claims"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SpecFormalityCheck",
            description: "说明书形式检查工具。检查说明书是否符合专利法第26条、实施细则第17-19条。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "specification": { "type": "object", "description": "说明书对象：technical_field/background_art/invention_content/embodiment/drawings" },
                    "claims": { "type": "array", "items": { "type": "object" } },
                    "patent_type": { "type": "string" }
                },
                "required": ["specification", "patent_type"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SubjectMatterCheck",
            description: "保护客体检查工具。检查发明是否属于专利法保护的客体，排除智力活动规则、疾病诊断方法等。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "invention_title": { "type": "string" },
                    "claims": { "type": "array", "items": { "type": "object" } },
                    "specification": { "type": "object" },
                    "patent_type": { "type": "string" }
                },
                "required": ["invention_title", "claims", "patent_type"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "UnityCheck",
            description: "单一性检查工具。检查多项权利要求是否符合专利法实施细则第43条单一性要求。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "claims": { "type": "array", "items": { "type": "object" }, "description": "权利要求列表，每项含 type/number/content" },
                    "patent_type": { "type": "string" },
                    "invention_title": { "type": "string" }
                },
                "required": ["claims", "patent_type"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "StrategyScore",
            description: "OA答复策略评分器。基于驳回类型、历史案例、风险偏好评估最优答复策略（argue/amend/both/abandon/appeal）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "rejection_reasons": { "type": "array", "items": { "type": "object" } },
                    "rejection_types": { "type": "array", "items": { "type": "string" } },
                    "affected_claims": { "type": "array", "items": { "type": "integer" } },
                    "patent_title": { "type": "string" },
                    "risk_tolerance": { "type": "number" }
                },
                "required": ["rejection_reasons"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "StrategyArguments",
            description: "OA答复论点生成器。根据驳回类型生成论点模板、修改建议、风险识别、补充证据建议。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "parse_result": { "type": "object", "description": "OA解析结果" },
                    "strategy": { "type": "string", "enum": ["argue", "amend", "both", "abandon", "appeal"] },
                    "scores": { "type": "array", "items": { "type": "object" } }
                },
                "required": ["parse_result", "strategy"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "ClaimGenerator",
            description: "权利要求书生成器。基于技术方案描述，利用LLM生成符合中国专利法要求的权利要求书（独立权利要求+从属权利要求）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "technicalSolution": { "type": "string", "description": "技术方案描述（详细的技术交底书内容）" },
                    "patentType": { "type": "string", "enum": ["invention", "utilityModel"], "description": "专利类型" },
                    "field": { "type": "string", "description": "技术领域（可选）" },
                    "existingClaims": { "type": "array", "items": { "type": "string" }, "description": "现有权利要求（用于改写/扩展，可选）" },
                    "language": { "type": "string", "enum": ["chinese", "english"], "description": "输出语言" },
                    "independentClaimCount": { "type": "integer", "description": "期望的独立权利要求数量" },
                    "dependentClaimMax": { "type": "integer", "description": "每项独立权利要求对应的从属权利要求数量上限" }
                },
                "required": ["technicalSolution"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "AbstractDrafter",
            description: "专利摘要起草器。基于技术方案描述，利用LLM生成符合专利法要求的专利摘要。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "technicalSolution": { "type": "string", "description": "技术方案描述" },
                    "patentType": { "type": "string", "enum": ["invention", "utilityModel", "design"], "description": "专利类型" },
                    "keyFeatures": { "type": "array", "items": { "type": "string" }, "description": "关键技术特征（可选）" },
                    "language": { "type": "string", "enum": ["chinese", "english"], "description": "输出语言" },
                    "maxWords": { "type": "integer", "description": "最大字数限制" }
                },
                "required": ["technicalSolution"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SpecificationDrafter",
            description: "说明书起草器。基于技术方案描述，利用LLM生成符合专利法要求的专利说明书各部分内容。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "technicalSolution": { "type": "string", "description": "技术方案描述" },
                    "patentType": { "type": "string", "enum": ["invention", "utilityModel", "design"], "description": "专利类型" },
                    "mode": { "type": "string", "enum": ["full", "background", "summary", "detailedDescription", "embodiments"], "description": "起草模式" },
                    "field": { "type": "string", "description": "技术领域（可选）" },
                    "priorArt": { "type": "string", "description": "现有技术背景（可选）" },
                    "technicalEffects": { "type": "array", "items": { "type": "string" }, "description": "期望的技术效果（可选）" },
                    "language": { "type": "string", "enum": ["chinese", "english"], "description": "输出语言" },
                    "detailLevel": { "type": "string", "enum": ["concise", "standard", "detailed"], "description": "详细程度" }
                },
                "required": ["technicalSolution"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "InnovationEvaluator",
            description: "创新度评估器。基于技术方案和现有技术，利用LLM评估新颖性、创造性、技术效果和市场潜力。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "technicalSolution": { "type": "string", "description": "技术方案描述" },
                    "priorArt": { "type": "array", "items": { "type": "string" }, "description": "现有技术描述（可选）" },
                    "field": { "type": "string", "description": "技术领域（可选）" },
                    "mode": { "type": "string", "enum": ["full", "novelty", "inventiveness", "technicalEffect", "marketValue"], "description": "评估维度" },
                    "language": { "type": "string", "enum": ["chinese", "english"], "description": "输出语言" }
                },
                "required": ["technicalSolution"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "DocumentRead",
            description: "Extract text from a structured document (PDF, Excel, DOCX).",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Absolute or relative path to the document" },
                    "max_pages": { "type": "integer", "description": "For PDF: max pages to extract (null = all)" },
                    "sheet": { "type": "string", "description": "For Excel: sheet name to read (null = first sheet)" }
                },
                "required": ["file_path"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "PdfParse",
            description: "Enhanced PDF parser with multiple operation modes (extract_text/parse/to_markdown). Returns structured output with page-level statistics and word counts.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the PDF file" },
                    "operation": { "type": "string", "enum": ["extract_text", "parse", "to_markdown"], "default": "extract_text", "description": "Operation mode" },
                    "start_page": { "type": "integer", "description": "Start page (optional)" },
                    "end_page": { "type": "integer", "description": "End page (optional)" }
                },
                "required": ["file_path"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "DocxParse",
            description: "Enhanced DOCX parser with multiple output formats (extract_text/to_html/to_markdown/parse). Detects patent sections and provides paragraph-level statistics.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the DOCX file" },
                    "operation": { "type": "string", "enum": ["extract_text", "to_html", "to_markdown", "parse"], "default": "extract_text", "description": "Operation mode" }
                },
                "required": ["file_path"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "ExcelParse",
            description: "Enhanced Excel parser supporting both XLS and XLSX formats. Operations: read/to_json/to_markdown/parse. Returns structured data with headers and row counts.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Path to the Excel file" },
                    "operation": { "type": "string", "enum": ["read", "to_json", "to_markdown", "parse"], "default": "read", "description": "Operation mode" },
                    "sheet_name": { "type": "string", "description": "Sheet name (optional, defaults to first sheet)" },
                    "max_rows": { "type": "integer", "default": 1000, "description": "Maximum rows to read" }
                },
                "required": ["file_path"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "MarkdownParse",
            description: "Patent document section parser. Operations: parse_markdown (heading-based), parse_plain_text (keyword-based), parse_claims (claim structure), parse_opinion_statement (OA dialog). Returns structured sections with aliases and content.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "Text content to parse" },
                    "operation": { "type": "string", "enum": ["parse_markdown", "parse_plain_text", "parse_claims", "parse_opinion_statement"], "default": "parse_markdown", "description": "Parsing mode" },
                    "metadata": { "type": "object", "additionalProperties": { "type": "string" }, "description": "Optional metadata" }
                },
                "required": ["text"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "PowerShell",
            description: "Execute a PowerShell command with optional timeout.",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "timeout": { "type": "integer", "minimum": 1 },
                    "description": { "type": "string" },
                    "run_in_background": { "type": "boolean" }
                },
                "required": ["command"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::DangerFullAccess,
        },
        // --- 新增专利工具 ---
        ToolSpec {
            name: "SynonymSearch",
            description: "专利同义词词典。70+术语覆盖7个技术领域，支持lookup/expand/detectDomain/buildQuery/buildProgressive/stats操作。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["lookup", "expand", "detect_domain", "build_query", "build_progressive", "stats"], "description": "操作类型" },
                    "term": { "type": "string", "description": "查询单词（lookup/build_query使用）" },
                    "terms": { "type": "array", "items": { "type": "string" }, "description": "批量查询术语（expand使用）" },
                    "domain": { "type": "string", "description": "技术领域过滤" },
                    "accuracy": { "type": "string", "enum": ["High", "Medium", "Low"], "default": "Medium" },
                    "field": { "type": "string", "enum": ["intitle", "inabstract", "inclaims", "all"], "default": "all" },
                    "exclusions": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["operation"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SearchQueryBuilder",
            description: "3阶段渐进式专利检索式构建器。自动同义词扩展，生成高精度→精化→补充三阶段检索策略。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "keywords": { "type": "array", "items": { "type": "string" }, "description": "检索关键词" },
                    "ipc_codes": { "type": "array", "items": { "type": "string" }, "description": "IPC分类号" },
                    "technical_field": { "type": "string" },
                    "technical_problem": { "type": "string" },
                    "domain_strategy": { "type": "string", "enum": ["broad", "focused", "precise"], "default": "focused" }
                },
                "required": ["keywords"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "PatentSearch",
            description: "本地专利检索。从本地 patent_db（PostgreSQL，7520万中国专利）检索，毫秒级响应。支持关键词/申请人/发明人/IPC/全文检索。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "检索内容：关键词、申请人名、发明人名、IPC分类号、或公开号（detail模式）" },
                    "search_type": { "type": "string", "enum": ["keyword", "applicant", "inventor", "ipc", "fulltext", "detail"], "default": "keyword", "description": "检索类型：keyword=关键词(默认), applicant=申请人, inventor=发明人, ipc=IPC分类, fulltext=全文, detail=公开号详情" },
                    "limit": { "type": "integer", "default": 10, "maximum": 100 },
                    "offset": { "type": "integer", "default": 0 }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "GooglePatentsFetch",
            description: "Google Patents专利检索。支持中英文检索和分页（需配置API凭证）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "page": { "type": "integer", "default": 1 },
                    "language": { "type": "string", "default": "zh" }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "HighCitationPatents",
            description: "高被引专利发现工具。按技术领域和IPC分类查找高引用量专利（需配置引用数据源）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "technology": { "type": "string" },
                    "ipc_code": { "type": "string" },
                    "min_citations": { "type": "integer", "default": 50 },
                    "limit": { "type": "integer", "default": 20 }
                },
                "required": ["technology"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "IterativeSearch",
            description: "迭代式深度检索工具。基于同义词扩展生成多轮查询变体，支持专利/文献/法律检索。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "max_iterations": { "type": "integer", "default": 3 },
                    "search_type": { "type": "string", "enum": ["patent", "literature", "legal"], "default": "patent" },
                    "width": { "type": "integer", "default": 3 }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "OaParse",
            description: "审查意见通知书结构化解析。纯规则引擎，支持CN/PCT/US/EP文档，提取驳回类型、权利要求、引用文件。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "OA文档文本内容" },
                    "application_number": { "type": "string" },
                    "patent_title": { "type": "string" },
                    "document_type": { "type": "string", "enum": ["cn", "pct", "us", "ep"], "default": "cn" },
                    "examiner": { "type": "string" },
                    "notification_date": { "type": "string" },
                    "deadline": { "type": "string" }
                },
                "required": ["content"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "ResponseTemplate",
            description: "OA答复模板库。6个内置模板覆盖CN/PCT新颖性争辩、创造性争辩、修改等策略，支持变量渲染。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["list", "filter", "render"], "description": "操作类型" },
                    "rejection_type": { "type": "string" },
                    "strategy": { "type": "string", "enum": ["argue", "amend", "both"] },
                    "template_id": { "type": "string" },
                    "variables": { "type": "object", "additionalProperties": { "type": "string" } }
                },
                "required": ["operation"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SuccessPredictor",
            description: "OA答复成功率预测器。基于驳回类型权重、策略评分、历史案例相似度计算成功率和置信区间。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "parse_result": { "type": "object", "description": "OaParse输出结果" },
                    "strategy": { "type": "string", "enum": ["argue", "amend", "both", "abandon", "appeal"] },
                    "round": { "type": "integer", "default": 1 },
                    "confidence_level": { "type": "string", "enum": ["90%", "95%", "99%"], "default": "90%" },
                    "historical_cases": { "type": "array", "items": { "type": "object" } }
                },
                "required": ["strategy"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SemanticCompare",
            description: "多维度语义对比工具。支持词法/嵌入/MARG-lite模式，计算标题、摘要、权利要求、特征四维相似度。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target": { "type": "object", "properties": { "title": { "type": "string" }, "abstract_text": { "type": "string" }, "claims": { "type": "array", "items": { "type": "string" } }, "features": { "type": "array", "items": { "type": "string" } } } },
                    "prior_art": { "type": "object", "properties": { "title": { "type": "string" }, "abstract_text": { "type": "string" }, "claims": { "type": "array", "items": { "type": "string" } }, "features": { "type": "array", "items": { "type": "string" } } } },
                    "compare_mode": { "type": "string", "enum": ["lexical", "embedding", "marg_lite", "auto"], "default": "lexical" },
                    "weights": { "type": "object" }
                },
                "required": ["target", "prior_art"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "InfringementAnalysis",
            description: "专利侵权分析工具。逐权利要求要素比对，判断字面侵权和等同侵权，评估风险等级（基于专利法第59条）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "patent_claims": { "type": "array", "items": { "type": "string" }, "description": "专利权利要求列表" },
                    "accused_product": { "type": "string", "description": "被控侵权产品描述" },
                    "analysis_type": { "type": "string", "enum": ["literal_equivalence", "doctrine_of_equivalents", "full"], "default": "full" }
                },
                "required": ["patent_claims", "accused_product"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "SynergyAnalysis",
            description: "技术特征协同效应检验。三条件测试：相同技术问题、协同效果、相互依赖，支持合并协同单元。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "units": { "type": "array", "items": { "type": "object", "properties": { "id": { "type": "string" }, "name": { "type": "string" }, "source_text": { "type": "string" }, "technical_function": { "type": "string" }, "technical_effect": { "type": "string" } }, "required": ["id", "name", "source_text"] } },
                    "apply_merge": { "type": "boolean", "default": false }
                },
                "required": ["units"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "LegalQA",
            description: "知识产权法律问答。覆盖专利法、商标法、著作权法，基于关键词匹配返回法条依据和建议。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "question": { "type": "string" },
                    "domain": { "type": "string", "enum": ["patent", "trademark", "copyright", "all"], "default": "all" },
                    "context": { "type": "string" }
                },
                "required": ["question"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "ProcessChart",
            description: "Mermaid流程图生成器。将步骤和流转数据转换为Mermaid语法流程图，嵌入Markdown。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "steps": { "type": "array", "items": { "type": "object", "properties": { "id": { "type": "string" }, "label": { "type": "string" }, "step_type": { "type": "string", "enum": ["start", "end", "decision", "process"], "default": "process" } }, "required": ["id", "label"] } },
                    "flows": { "type": "array", "items": { "type": "object", "properties": { "from": { "type": "string" }, "to": { "type": "string" }, "label": { "type": "string" } }, "required": ["from", "to"] } },
                    "title": { "type": "string" }
                },
                "required": ["steps", "flows"]
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "DrawingUnderstanding",
            description: "专利附图理解工具。基于文本描述提取组件、连接关系和技术特征，支持多种图纸类型。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "figure_number": { "type": "string" },
                    "image_description": { "type": "string", "description": "附图的文字描述" },
                    "technical_field": { "type": "string" },
                    "drawing_type": { "type": "string", "enum": ["block_diagram", "circuit", "flowchart", "mechanical", "chemical", "general"] }
                },
                "required": ["figure_number", "image_description"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "TechnicalDrawing",
            description: "技术图纸识别工具。自动检测图纸类型（化学/数学/电气），提取元器件符号和结构。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "image_description": { "type": "string" },
                    "drawing_type": { "type": "string", "enum": ["chemical", "math", "electrical", "general"] },
                    "auto_detect": { "type": "boolean", "default": true }
                },
                "required": ["image_description"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "PatentManager",
            description: "专利生命周期管理工具。状态机驱动的CRUD操作，支持截止日、费用追踪和组合报告。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["add", "update", "remove", "get", "list", "change_status", "add_deadline", "get_upcoming_deadlines", "add_fee", "get_pending_fees", "get_portfolio", "generate_report"] },
                    "patent_id": { "type": "string" },
                    "patent": { "type": "object" },
                    "new_status": { "type": "string" },
                    "deadline": { "type": "object" },
                    "fee": { "type": "object" }
                },
                "required": ["operation"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::WorkspaceWrite,
        },
        ToolSpec {
            name: "TemplateLibrary",
            description: "专利文档模板库。5个内置模板：OA答复、专利申请、无效宣告、侵权分析、检索报告，支持变量渲染。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "operation": { "type": "string", "enum": ["list", "load", "render"] },
                    "template_type": { "type": "string", "enum": ["oa-response", "patent-application", "invalidity-request", "infringement-analysis", "patent-search-report"] },
                    "template_id": { "type": "string" },
                    "variables": { "type": "object", "additionalProperties": { "type": "string" } }
                },
                "required": ["operation"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "TrademarkAnalysis",
            description: "商标可注册性分析工具。评估显著性、描述性、冲突风险，基于商标法第9-12条。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "trademark_name": { "type": "string" },
                    "goods_services": { "type": "string" },
                    "trademark_type": { "type": "string", "enum": ["word", "design", "composite", "three_dimensional"], "default": "word" }
                },
                "required": ["trademark_name"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        },
        ToolSpec {
            name: "PatentDownload",
            description: "专利文档下载工具。支持PDF/XML/全文格式下载（需配置下载服务）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "patent_id": { "type": "string", "description": "专利号" },
                    "output_dir": { "type": "string" },
                    "format": { "type": "string", "enum": ["pdf", "xml", "fulltext"], "default": "pdf" }
                },
                "required": ["patent_id"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::WorkspaceWrite,
        },
        ToolSpec {
            name: "BatchPatentDownload",
            description: "批量专利文档下载工具。支持多专利号批量下载和统计（需配置下载服务）。",
            input_schema: json!({
                "type": "object",
                "properties": {
                    "patent_ids": { "type": "array", "items": { "type": "string" }, "description": "专利号列表" },
                    "output_dir": { "type": "string" },
                    "format": { "type": "string", "enum": ["pdf", "xml", "fulltext"], "default": "pdf" }
                },
                "required": ["patent_ids"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::WorkspaceWrite,
        },
    ]
}
