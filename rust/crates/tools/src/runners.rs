//! Thin runner wrappers that bridge the dispatcher to the actual tool implementations.

use runtime::{
    edit_file, execute_bash, glob_search, grep_search, read_file, write_file, BashCommandInput,
    GrepSearchInput,
};
use serde::Deserialize;

use crate::agent::AgentInput;
use crate::{
    brief, config_tool, notebook, patent, patent_analysis, patent_compare, patent_drafting,
    patent_formality, patent_management, patent_oa, patent_quality, patent_search, patent_strategy,
    patent_visualization, repl, shell, skill, todo, tool_search, web,
};

fn to_pretty_json<T: serde::Serialize>(value: T) -> Result<String, String> {
    serde_json::to_string_pretty(&value).map_err(|error| error.to_string())
}

#[allow(clippy::needless_pass_by_value)]
fn io_to_string(error: std::io::Error) -> String {
    error.to_string()
}

pub(super) fn run_bash(input: BashCommandInput) -> Result<String, String> {
    serde_json::to_string_pretty(&execute_bash(input).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn run_read_file(input: ReadFileInput) -> Result<String, String> {
    to_pretty_json(read_file(&input.path, input.offset, input.limit).map_err(io_to_string)?)
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn run_write_file(input: WriteFileInput) -> Result<String, String> {
    to_pretty_json(write_file(&input.path, &input.content).map_err(io_to_string)?)
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn run_edit_file(input: EditFileInput) -> Result<String, String> {
    to_pretty_json(
        edit_file(
            &input.path,
            &input.old_string,
            &input.new_string,
            input.replace_all.unwrap_or(false),
        )
        .map_err(io_to_string)?,
    )
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn run_glob_search(input: GlobSearchInputValue) -> Result<String, String> {
    to_pretty_json(glob_search(&input.pattern, input.path.as_deref()).map_err(io_to_string)?)
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn run_grep_search(input: GrepSearchInput) -> Result<String, String> {
    to_pretty_json(grep_search(&input).map_err(io_to_string)?)
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn run_web_fetch(input: web::WebFetchInput) -> Result<String, String> {
    to_pretty_json(web::execute_web_fetch(&input)?)
}

#[allow(clippy::needless_pass_by_value)]
pub(super) fn run_web_search(input: web::WebSearchInput) -> Result<String, String> {
    to_pretty_json(web::execute_web_search(&input)?)
}

pub(super) fn run_todo_write(input: todo::TodoWriteInput) -> Result<String, String> {
    to_pretty_json(todo::execute_todo_write(input)?)
}

pub(super) fn run_skill(input: skill::SkillInput) -> Result<String, String> {
    to_pretty_json(skill::execute_skill(input)?)
}

pub(super) fn run_agent(input: AgentInput) -> Result<String, String> {
    to_pretty_json(crate::agent::execute_agent(input)?)
}

pub(super) fn run_tool_search(input: ToolSearchInput) -> Result<String, String> {
    to_pretty_json(tool_search::execute_tool_search(
        input.query,
        input.max_results,
    ))
}

pub(super) fn run_notebook_edit(input: notebook::NotebookEditInput) -> Result<String, String> {
    to_pretty_json(notebook::execute_notebook_edit(input)?)
}

pub(super) fn run_sleep(input: brief::SleepInput) -> Result<String, String> {
    to_pretty_json(brief::execute_sleep(input))
}

pub(super) fn run_brief(input: brief::BriefInput) -> Result<String, String> {
    to_pretty_json(brief::execute_brief(input)?)
}

pub(super) fn run_config(input: config_tool::ConfigInput) -> Result<String, String> {
    to_pretty_json(config_tool::execute_config(input)?)
}

pub(super) fn run_structured_output(
    input: config_tool::StructuredOutputInput,
) -> Result<String, String> {
    to_pretty_json(config_tool::execute_structured_output(input))
}

pub(super) fn run_repl(input: repl::ReplInput) -> Result<String, String> {
    to_pretty_json(repl::execute_repl(input)?)
}

pub(super) fn run_powershell(input: shell::PowerShellInput) -> Result<String, String> {
    to_pretty_json(shell::execute_powershell(input).map_err(|error| error.to_string())?)
}

// --- 专利专用工具运行函数 ---

pub(super) fn run_claim_parse(input: &patent::ClaimParseInput) -> Result<String, String> {
    to_pretty_json(patent::execute_claim_parse(input)?)
}

pub(super) fn run_claim_compare(input: &patent::ClaimCompareInput) -> Result<String, String> {
    to_pretty_json(patent::execute_claim_compare(input)?)
}

pub(super) fn run_novelty_analysis(input: &patent::NoveltyAnalysisInput) -> Result<String, String> {
    to_pretty_json(patent::execute_novelty_analysis(input)?)
}

pub(super) fn run_inventiveness_analysis(
    input: &patent::InventivenessAnalysisInput,
) -> Result<String, String> {
    to_pretty_json(patent::execute_inventiveness_analysis(input)?)
}

pub(super) fn run_oa_strategy(input: &patent::OaStrategyInput) -> Result<String, String> {
    to_pretty_json(patent::execute_oa_strategy(input)?)
}

pub(super) fn run_formal_check(input: &patent::FormalCheckInput) -> Result<String, String> {
    to_pretty_json(patent::execute_formal_check(input)?)
}

pub(super) fn run_quality_assess(input: &patent::QualityAssessInput) -> Result<String, String> {
    to_pretty_json(patent::execute_quality_assess(input))
}

pub(super) fn run_knowledge_graph_query(
    input: &patent::KnowledgeGraphQueryInput,
) -> Result<String, String> {
    to_pretty_json(patent::execute_knowledge_graph_query(input)?)
}

pub(super) fn run_patent_compare(
    input: &patent_compare::PatentCompareInput,
) -> Result<String, String> {
    to_pretty_json(patent_compare::execute_patent_compare(input)?)
}

pub(super) fn run_quality_scorer(
    input: &patent_quality::QualityScorerInput,
) -> Result<String, String> {
    to_pretty_json(patent_quality::execute_quality_scorer(input)?)
}

pub(super) fn run_quality_checker(
    input: &patent_quality::QualityCheckerInput,
) -> Result<String, String> {
    to_pretty_json(patent_quality::execute_quality_checker(input)?)
}

pub(super) fn run_claim_formality_check(
    input: &patent_formality::ClaimFormalityInput,
) -> Result<String, String> {
    to_pretty_json(patent_formality::execute_claim_formality_check(input)?)
}

pub(super) fn run_spec_formality_check(
    input: &patent_formality::SpecFormalityInput,
) -> Result<String, String> {
    to_pretty_json(patent_formality::execute_spec_formality_check(input)?)
}

pub(super) fn run_subject_matter_check(
    input: &patent_formality::SubjectMatterInput,
) -> Result<String, String> {
    to_pretty_json(patent_formality::execute_subject_matter_check(input)?)
}

pub(super) fn run_unity_check(input: &patent_formality::UnityCheckInput) -> Result<String, String> {
    to_pretty_json(patent_formality::execute_unity_check(input)?)
}

pub(super) fn run_strategy_score(
    input: &patent_strategy::StrategyScoreInput,
) -> Result<String, String> {
    to_pretty_json(patent_strategy::execute_strategy_score(input)?)
}

pub(super) fn run_strategy_arguments(
    input: &patent_strategy::StrategyArgumentInput,
) -> Result<String, String> {
    to_pretty_json(patent_strategy::execute_strategy_arguments(input)?)
}

pub(super) fn run_claim_generator(
    input: &patent_drafting::ClaimGeneratorInput,
) -> Result<String, String> {
    to_pretty_json(patent_drafting::execute_claim_generator(input)?)
}

pub(super) fn run_abstract_drafter(
    input: &patent_drafting::AbstractDrafterInput,
) -> Result<String, String> {
    to_pretty_json(patent_drafting::execute_abstract_drafter(input)?)
}

pub(super) fn run_specification_drafter(
    input: &patent_drafting::SpecificationDrafterInput,
) -> Result<String, String> {
    to_pretty_json(patent_drafting::execute_specification_drafter(input)?)
}

pub(super) fn run_innovation_evaluator(
    input: &patent_drafting::InnovationEvaluatorInput,
) -> Result<String, String> {
    to_pretty_json(patent_drafting::execute_innovation_evaluator(input)?)
}

// --- 新增专利工具 wrapper 函数 ---

pub(super) fn run_synonym_search(
    input: patent_search::SynonymSearchInput,
) -> Result<String, String> {
    to_pretty_json(patent_search::synonym_search(input)?)
}

pub(super) fn run_search_query_builder(
    input: patent_search::SearchQueryBuilderInput,
) -> Result<String, String> {
    to_pretty_json(patent_search::search_query_builder(input)?)
}

pub(super) fn run_patent_search(input: patent_search::PatentSearchInput) -> Result<String, String> {
    to_pretty_json(patent_search::patent_search(input)?)
}

pub(super) fn run_google_patents_fetch(
    input: patent_search::GooglePatentsFetchInput,
) -> Result<String, String> {
    to_pretty_json(patent_search::google_patents_fetch(input)?)
}

pub(super) fn run_high_citation_patents(
    input: &patent_search::HighCitationPatentsInput,
) -> Result<String, String> {
    to_pretty_json(patent_search::high_citation_patents(input)?)
}

pub(super) fn run_iterative_search(
    input: patent_search::IterativeSearchInput,
) -> Result<String, String> {
    to_pretty_json(patent_search::iterative_search(input)?)
}

pub(super) fn run_oa_parse(input: &patent_oa::OaParseInput) -> Result<String, String> {
    to_pretty_json(patent_oa::execute_oa_parse(input)?)
}

pub(super) fn run_response_template(
    input: &patent_oa::ResponseTemplateInput,
) -> Result<String, String> {
    to_pretty_json(patent_oa::execute_response_template(input)?)
}

pub(super) fn run_success_predictor(
    input: &patent_oa::SuccessPredictorInput,
) -> Result<String, String> {
    to_pretty_json(patent_oa::execute_success_predictor(input)?)
}

pub(super) fn run_semantic_compare(
    input: patent_analysis::SemanticCompareInput,
) -> Result<String, String> {
    to_pretty_json(patent_analysis::semantic_compare(input)?)
}

pub(super) fn run_infringement_analysis(
    input: patent_analysis::InfringementAnalysisInput,
) -> Result<String, String> {
    to_pretty_json(patent_analysis::infringement_analysis(input)?)
}

pub(super) fn run_synergy_analysis(
    input: patent_analysis::SynergyAnalysisInput,
) -> Result<String, String> {
    to_pretty_json(patent_analysis::synergy_analysis(input)?)
}

pub(super) fn run_legal_qa(input: patent_analysis::LegalQAInput) -> Result<String, String> {
    to_pretty_json(patent_analysis::legal_qa(input)?)
}

pub(super) fn run_process_chart(
    input: patent_visualization::ProcessChartInput,
) -> Result<String, String> {
    to_pretty_json(patent_visualization::process_chart(input)?)
}

pub(super) fn run_drawing_understanding(
    input: &patent_visualization::DrawingUnderstandingInput,
) -> Result<String, String> {
    to_pretty_json(patent_visualization::drawing_understanding(input))
}

pub(super) fn run_technical_drawing(
    input: &patent_visualization::TechnicalDrawingInput,
) -> Result<String, String> {
    to_pretty_json(patent_visualization::technical_drawing(input))
}

pub(super) fn run_patent_manager(
    input: patent_management::PatentManagerInput,
) -> Result<String, String> {
    to_pretty_json(patent_management::execute_patent_manager(input)?)
}

pub(super) fn run_template_library(
    input: patent_management::TemplateLibraryInput,
) -> Result<String, String> {
    to_pretty_json(patent_management::execute_template_library(input)?)
}

pub(super) fn run_trademark_analysis(
    input: patent_management::TrademarkAnalysisInput,
) -> Result<String, String> {
    to_pretty_json(patent_management::execute_trademark_analysis(input)?)
}

pub(super) fn run_patent_download(
    input: patent_management::PatentDownloadInput,
) -> Result<String, String> {
    to_pretty_json(patent_management::execute_patent_download(input)?)
}

pub(super) fn run_batch_patent_download(
    input: patent_management::BatchPatentDownloadInput,
) -> Result<String, String> {
    to_pretty_json(patent_management::execute_batch_patent_download(input)?)
}

// --- Input structs kept for dispatcher use ---

#[derive(Debug, Deserialize)]
pub(super) struct ReadFileInput {
    pub path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub(super) struct WriteFileInput {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct EditFileInput {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
    pub replace_all: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub(super) struct GlobSearchInputValue {
    pub pattern: String,
    pub path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ToolSearchInput {
    pub query: String,
    pub max_results: Option<usize>,
}
