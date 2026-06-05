//! Tool registry: maps tool names to their runner closures.
//!
//! Replaces the former 40+-arm `match` in `dispatch.rs` with a `HashMap` lookup
//! against a lazily-initialized global registry.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use serde::Deserialize;
use serde_json::Value;

use crate::agent::AgentInput;
use crate::runners::{
    self, EditFileInput, GlobSearchInputValue, ReadFileInput, ToolSearchInput, WriteFileInput,
};
use crate::{
    brief, config_tool, document, knowledge_tools, notebook, patent, patent_analysis,
    patent_compare, patent_document, patent_drafting, patent_formality, patent_management,
    patent_oa, patent_quality, patent_search, patent_strategy, patent_visualization, repl, shell,
    skill, todo, web,
};

type ToolRunner = Arc<dyn Fn(&Value) -> Result<String, String> + Send + Sync>;

/// Global tool registry — lazily populated on first access.
pub static GLOBAL_REGISTRY: LazyLock<ToolRegistry> = LazyLock::new(init_global_registry);

/// Tool registry backed by a `HashMap<&str, ToolRunner>`.
pub struct ToolRegistry {
    tools: HashMap<&'static str, ToolRunner>,
}

impl ToolRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool runner under the given name.
    pub fn register(&mut self, name: &'static str, runner: ToolRunner) {
        self.tools.insert(name, runner);
    }

    /// Look up a tool by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ToolRunner> {
        self.tools.get(name)
    }

    /// Return all registered tool names.
    #[must_use]
    pub fn all_names(&self) -> Vec<&'static str> {
        self.tools.keys().copied().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn from_value<T: for<'de> Deserialize<'de>>(input: &Value) -> Result<T, String> {
    serde_json::from_value(input.clone()).map_err(|error| error.to_string())
}

/// Register a tool whose runner takes an owned input: `fn(T) -> Result<String, String>`.
macro_rules! reg {
    ($reg:expr, $name:expr, $ty:ty, $runner:expr) => {
        $reg.register(
            $name,
            Arc::new(|input: &Value| from_value::<$ty>(input).and_then($runner)),
        );
    };
}

/// Register a tool whose runner takes a borrowed input: `fn(&T) -> Result<String, String>`.
macro_rules! reg_ref {
    ($reg:expr, $name:expr, $ty:ty, $runner:expr) => {
        $reg.register(
            $name,
            Arc::new(|input: &Value| from_value::<$ty>(input).and_then(|v| $runner(&v))),
        );
    };
}

// ── Registry initialization ─────────────────────────────────────────────────

fn init_global_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();

    // ── Core tools (owned input) ─────────────────────────────────────────────

    reg!(reg, "bash", runtime::BashCommandInput, runners::run_bash);
    reg!(reg, "read_file", ReadFileInput, runners::run_read_file);
    reg!(reg, "write_file", WriteFileInput, runners::run_write_file);
    reg!(reg, "edit_file", EditFileInput, runners::run_edit_file);
    reg!(
        reg,
        "glob_search",
        GlobSearchInputValue,
        runners::run_glob_search
    );
    reg!(
        reg,
        "grep_search",
        runtime::GrepSearchInput,
        runners::run_grep_search
    );
    reg!(reg, "WebFetch", web::WebFetchInput, runners::run_web_fetch);
    reg!(
        reg,
        "WebSearch",
        web::WebSearchInput,
        runners::run_web_search
    );
    reg!(
        reg,
        "TodoWrite",
        todo::TodoWriteInput,
        runners::run_todo_write
    );
    reg!(reg, "Skill", skill::SkillInput, runners::run_skill);
    reg!(reg, "Agent", AgentInput, runners::run_agent);
    reg!(reg, "ToolSearch", ToolSearchInput, runners::run_tool_search);
    reg!(
        reg,
        "NotebookEdit",
        notebook::NotebookEditInput,
        runners::run_notebook_edit
    );
    reg!(reg, "Sleep", brief::SleepInput, runners::run_sleep);
    reg!(
        reg,
        "SendUserMessage",
        brief::BriefInput,
        runners::run_brief
    );
    reg!(reg, "Brief", brief::BriefInput, runners::run_brief);
    reg!(reg, "Config", config_tool::ConfigInput, runners::run_config);
    reg!(
        reg,
        "StructuredOutput",
        config_tool::StructuredOutputInput,
        runners::run_structured_output
    );
    reg!(reg, "REPL", repl::ReplInput, runners::run_repl);
    reg!(
        reg,
        "PowerShell",
        shell::PowerShellInput,
        runners::run_powershell
    );

    // ── Document tools (ref-parameter, direct module calls) ──────────────────

    reg_ref!(
        reg,
        "DocumentRead",
        document::DocumentReadInput,
        document::run_document_read
    );
    reg_ref!(
        reg,
        "PdfParse",
        patent_document::PdfParseInput,
        patent_document::run_pdf_parse
    );
    reg_ref!(
        reg,
        "DocxParse",
        patent_document::DocxParseInput,
        patent_document::run_docx_parse
    );
    reg_ref!(
        reg,
        "ExcelParse",
        patent_document::ExcelParseInput,
        patent_document::run_excel_parse
    );
    reg_ref!(
        reg,
        "MarkdownParse",
        patent_document::MarkdownParseInput,
        patent_document::run_markdown_parse
    );

    // ── Patent core tools (ref-parameter via runners) ────────────────────────

    reg_ref!(
        reg,
        "ClaimParse",
        patent::ClaimParseInput,
        runners::run_claim_parse
    );
    reg_ref!(
        reg,
        "ClaimCompare",
        patent::ClaimCompareInput,
        runners::run_claim_compare
    );
    reg_ref!(
        reg,
        "PatentCompare",
        patent_compare::PatentCompareInput,
        runners::run_patent_compare
    );
    reg_ref!(
        reg,
        "QualityScorer",
        patent_quality::QualityScorerInput,
        runners::run_quality_scorer
    );
    reg_ref!(
        reg,
        "QualityChecker",
        patent_quality::QualityCheckerInput,
        runners::run_quality_checker
    );

    // ── Patent formality tools (ref-parameter) ──────────────────────────────

    reg_ref!(
        reg,
        "ClaimFormalityCheck",
        patent_formality::ClaimFormalityInput,
        runners::run_claim_formality_check
    );
    reg_ref!(
        reg,
        "SpecFormalityCheck",
        patent_formality::SpecFormalityInput,
        runners::run_spec_formality_check
    );
    reg_ref!(
        reg,
        "SubjectMatterCheck",
        patent_formality::SubjectMatterInput,
        runners::run_subject_matter_check
    );
    reg_ref!(
        reg,
        "UnityCheck",
        patent_formality::UnityCheckInput,
        runners::run_unity_check
    );

    // ── Patent strategy tools (ref-parameter) ───────────────────────────────

    reg_ref!(
        reg,
        "StrategyScore",
        patent_strategy::StrategyScoreInput,
        runners::run_strategy_score
    );
    reg_ref!(
        reg,
        "StrategyArguments",
        patent_strategy::StrategyArgumentInput,
        runners::run_strategy_arguments
    );

    // ── Patent drafting tools (ref-parameter) ───────────────────────────────

    reg_ref!(
        reg,
        "ClaimGenerator",
        patent_drafting::ClaimGeneratorInput,
        runners::run_claim_generator
    );
    reg_ref!(
        reg,
        "AbstractDrafter",
        patent_drafting::AbstractDrafterInput,
        runners::run_abstract_drafter
    );
    reg_ref!(
        reg,
        "SpecificationDrafter",
        patent_drafting::SpecificationDrafterInput,
        runners::run_specification_drafter
    );
    reg_ref!(
        reg,
        "InnovationEvaluator",
        patent_drafting::InnovationEvaluatorInput,
        runners::run_innovation_evaluator
    );

    // ── Patent analysis tools (ref-parameter) ───────────────────────────────

    reg_ref!(
        reg,
        "NoveltyAnalysis",
        patent::NoveltyAnalysisInput,
        runners::run_novelty_analysis
    );
    reg_ref!(
        reg,
        "InventivenessAnalysis",
        patent::InventivenessAnalysisInput,
        runners::run_inventiveness_analysis
    );
    reg_ref!(
        reg,
        "OaStrategy",
        patent::OaStrategyInput,
        runners::run_oa_strategy
    );
    reg_ref!(
        reg,
        "FormalCheck",
        patent::FormalCheckInput,
        runners::run_formal_check
    );
    reg_ref!(
        reg,
        "QualityAssess",
        patent::QualityAssessInput,
        runners::run_quality_assess
    );
    reg_ref!(
        reg,
        "KnowledgeGraphQuery",
        patent::KnowledgeGraphQueryInput,
        runners::run_knowledge_graph_query
    );

    // ── Patent search tools (owned input) ───────────────────────────────────

    reg!(
        reg,
        "SynonymSearch",
        patent_search::SynonymSearchInput,
        runners::run_synonym_search
    );
    reg!(
        reg,
        "SearchQueryBuilder",
        patent_search::SearchQueryBuilderInput,
        runners::run_search_query_builder
    );
    reg!(
        reg,
        "PatentSearch",
        patent_search::PatentSearchInput,
        runners::run_patent_search
    );
    reg!(
        reg,
        "GooglePatentsFetch",
        patent_search::GooglePatentsFetchInput,
        runners::run_google_patents_fetch
    );
    reg_ref!(
        reg,
        "HighCitationPatents",
        patent_search::HighCitationPatentsInput,
        runners::run_high_citation_patents
    );
    reg!(
        reg,
        "IterativeSearch",
        patent_search::IterativeSearchInput,
        runners::run_iterative_search
    );

    // ── Patent OA tools (ref-parameter) ─────────────────────────────────────

    reg_ref!(
        reg,
        "OaParse",
        patent_oa::OaParseInput,
        runners::run_oa_parse
    );
    reg_ref!(
        reg,
        "ResponseTemplate",
        patent_oa::ResponseTemplateInput,
        runners::run_response_template
    );
    reg_ref!(
        reg,
        "SuccessPredictor",
        patent_oa::SuccessPredictorInput,
        runners::run_success_predictor
    );

    // ── Patent analysis (extended) tools (owned input) ──────────────────────

    reg!(
        reg,
        "SemanticCompare",
        patent_analysis::SemanticCompareInput,
        runners::run_semantic_compare
    );
    reg!(
        reg,
        "InfringementAnalysis",
        patent_analysis::InfringementAnalysisInput,
        runners::run_infringement_analysis
    );
    reg!(
        reg,
        "SynergyAnalysis",
        patent_analysis::SynergyAnalysisInput,
        runners::run_synergy_analysis
    );
    reg!(
        reg,
        "LegalQA",
        patent_analysis::LegalQAInput,
        runners::run_legal_qa
    );

    // ── Patent visualization tools (mixed) ──────────────────────────────────

    reg!(
        reg,
        "ProcessChart",
        patent_visualization::ProcessChartInput,
        runners::run_process_chart
    );
    reg_ref!(
        reg,
        "DrawingUnderstanding",
        patent_visualization::DrawingUnderstandingInput,
        runners::run_drawing_understanding
    );
    reg_ref!(
        reg,
        "TechnicalDrawing",
        patent_visualization::TechnicalDrawingInput,
        runners::run_technical_drawing
    );

    // ── Patent management tools (owned input) ───────────────────────────────

    reg!(
        reg,
        "PatentManager",
        patent_management::PatentManagerInput,
        runners::run_patent_manager
    );
    reg!(
        reg,
        "TemplateLibrary",
        patent_management::TemplateLibraryInput,
        runners::run_template_library
    );
    reg!(
        reg,
        "TrademarkAnalysis",
        patent_management::TrademarkAnalysisInput,
        runners::run_trademark_analysis
    );
    reg!(
        reg,
        "PatentDownload",
        patent_management::PatentDownloadInput,
        runners::run_patent_download
    );
    reg!(
        reg,
        "BatchPatentDownload",
        patent_management::BatchPatentDownloadInput,
        runners::run_batch_patent_download
    );

    // ── Knowledge tools (ref-parameter) ─────────────────────────────────────

    reg_ref!(
        reg,
        "KnowledgeSearch",
        knowledge_tools::KnowledgeSearchInput,
        runners::run_knowledge_search
    );
    reg_ref!(
        reg,
        "LegalReasoning",
        knowledge_tools::LegalReasoningInput,
        runners::run_legal_reasoning
    );
    reg_ref!(
        reg,
        "LawQuery",
        knowledge_tools::LawQueryInput,
        runners::run_law_query
    );
    reg_ref!(
        reg,
        "KnowledgeCard",
        knowledge_tools::KnowledgeCardInput,
        runners::run_knowledge_card
    );
    reg_ref!(
        reg,
        "SuperReasoningPlan",
        knowledge_tools::SuperReasoningPlanInput,
        runners::run_super_reasoning_plan
    );

    reg
}
