//! Tool dispatch: maps tool names to their runner functions.

use serde::Deserialize;
use serde_json::Value;

use crate::agent::AgentInput;
use crate::runners::{
    self, EditFileInput, GlobSearchInputValue, ReadFileInput, ToolSearchInput, WriteFileInput,
};
use crate::{
    brief, config_tool, document, notebook, patent, patent_analysis, patent_compare,
    patent_document, patent_drafting, patent_formality, patent_management, patent_oa,
    patent_quality, patent_search, patent_strategy, patent_visualization, repl, shell, skill, todo,
    web,
};

/// Dispatches tool execution by name, deserializing the input JSON and returning the result.
///
/// Returns an error string if the tool name is unknown or execution fails.
#[allow(clippy::too_many_lines)]
pub fn execute_tool(name: &str, input: &Value) -> Result<String, String> {
    match name {
        "bash" => from_value::<runtime::BashCommandInput>(input).and_then(runners::run_bash),
        "read_file" => from_value::<ReadFileInput>(input).and_then(runners::run_read_file),
        "write_file" => from_value::<WriteFileInput>(input).and_then(runners::run_write_file),
        "edit_file" => from_value::<EditFileInput>(input).and_then(runners::run_edit_file),
        "glob_search" => {
            from_value::<GlobSearchInputValue>(input).and_then(runners::run_glob_search)
        }
        "grep_search" => {
            from_value::<runtime::GrepSearchInput>(input).and_then(runners::run_grep_search)
        }
        "WebFetch" => from_value::<web::WebFetchInput>(input).and_then(runners::run_web_fetch),
        "WebSearch" => from_value::<web::WebSearchInput>(input).and_then(runners::run_web_search),
        "TodoWrite" => from_value::<todo::TodoWriteInput>(input).and_then(runners::run_todo_write),
        "Skill" => from_value::<skill::SkillInput>(input).and_then(runners::run_skill),
        "Agent" => from_value::<AgentInput>(input).and_then(runners::run_agent),
        "ToolSearch" => from_value::<ToolSearchInput>(input).and_then(runners::run_tool_search),
        "NotebookEdit" => {
            from_value::<notebook::NotebookEditInput>(input).and_then(runners::run_notebook_edit)
        }
        "Sleep" => from_value::<brief::SleepInput>(input).and_then(runners::run_sleep),
        "SendUserMessage" | "Brief" => {
            from_value::<brief::BriefInput>(input).and_then(runners::run_brief)
        }
        "Config" => from_value::<config_tool::ConfigInput>(input).and_then(runners::run_config),
        "StructuredOutput" => from_value::<config_tool::StructuredOutputInput>(input)
            .and_then(runners::run_structured_output),
        "REPL" => from_value::<repl::ReplInput>(input).and_then(runners::run_repl),
        "PowerShell" => {
            from_value::<shell::PowerShellInput>(input).and_then(runners::run_powershell)
        }
        "DocumentRead" => from_value::<document::DocumentReadInput>(input)
            .and_then(|input| document::run_document_read(&input)),
        "PdfParse" => from_value::<patent_document::PdfParseInput>(input)
            .and_then(|input| patent_document::run_pdf_parse(&input)),
        "DocxParse" => from_value::<patent_document::DocxParseInput>(input)
            .and_then(|input| patent_document::run_docx_parse(&input)),
        "ExcelParse" => from_value::<patent_document::ExcelParseInput>(input)
            .and_then(|input| patent_document::run_excel_parse(&input)),
        "MarkdownParse" => from_value::<patent_document::MarkdownParseInput>(input)
            .and_then(|input| patent_document::run_markdown_parse(&input)),
        // 专利专用工具
        "ClaimParse" => from_value::<patent::ClaimParseInput>(input)
            .and_then(|input| runners::run_claim_parse(&input)),
        "ClaimCompare" => from_value::<patent::ClaimCompareInput>(input)
            .and_then(|input| runners::run_claim_compare(&input)),
        "PatentCompare" => from_value::<patent_compare::PatentCompareInput>(input)
            .and_then(|input| runners::run_patent_compare(&input)),
        "QualityScorer" => from_value::<patent_quality::QualityScorerInput>(input)
            .and_then(|input| runners::run_quality_scorer(&input)),
        "QualityChecker" => from_value::<patent_quality::QualityCheckerInput>(input)
            .and_then(|input| runners::run_quality_checker(&input)),
        "ClaimFormalityCheck" => from_value::<patent_formality::ClaimFormalityInput>(input)
            .and_then(|input| runners::run_claim_formality_check(&input)),
        "SpecFormalityCheck" => from_value::<patent_formality::SpecFormalityInput>(input)
            .and_then(|input| runners::run_spec_formality_check(&input)),
        "SubjectMatterCheck" => from_value::<patent_formality::SubjectMatterInput>(input)
            .and_then(|input| runners::run_subject_matter_check(&input)),
        "UnityCheck" => from_value::<patent_formality::UnityCheckInput>(input)
            .and_then(|input| runners::run_unity_check(&input)),
        "StrategyScore" => from_value::<patent_strategy::StrategyScoreInput>(input)
            .and_then(|input| runners::run_strategy_score(&input)),
        "StrategyArguments" => from_value::<patent_strategy::StrategyArgumentInput>(input)
            .and_then(|input| runners::run_strategy_arguments(&input)),
        "ClaimGenerator" => from_value::<patent_drafting::ClaimGeneratorInput>(input)
            .and_then(|input| runners::run_claim_generator(&input)),
        "AbstractDrafter" => from_value::<patent_drafting::AbstractDrafterInput>(input)
            .and_then(|input| runners::run_abstract_drafter(&input)),
        "SpecificationDrafter" => from_value::<patent_drafting::SpecificationDrafterInput>(input)
            .and_then(|input| runners::run_specification_drafter(&input)),
        "InnovationEvaluator" => from_value::<patent_drafting::InnovationEvaluatorInput>(input)
            .and_then(|input| runners::run_innovation_evaluator(&input)),
        "NoveltyAnalysis" => from_value::<patent::NoveltyAnalysisInput>(input)
            .and_then(|input| runners::run_novelty_analysis(&input)),
        "InventivenessAnalysis" => from_value::<patent::InventivenessAnalysisInput>(input)
            .and_then(|input| runners::run_inventiveness_analysis(&input)),
        "OaStrategy" => from_value::<patent::OaStrategyInput>(input)
            .and_then(|input| runners::run_oa_strategy(&input)),
        "FormalCheck" => from_value::<patent::FormalCheckInput>(input)
            .and_then(|input| runners::run_formal_check(&input)),
        "QualityAssess" => from_value::<patent::QualityAssessInput>(input)
            .and_then(|input| runners::run_quality_assess(&input)),
        "KnowledgeGraphQuery" => from_value::<patent::KnowledgeGraphQueryInput>(input)
            .and_then(|input| runners::run_knowledge_graph_query(&input)),
        // --- 新增专利工具 match arms ---
        "SynonymSearch" => from_value::<patent_search::SynonymSearchInput>(input)
            .and_then(runners::run_synonym_search),
        "SearchQueryBuilder" => from_value::<patent_search::SearchQueryBuilderInput>(input)
            .and_then(runners::run_search_query_builder),
        "PatentSearch" => from_value::<patent_search::PatentSearchInput>(input)
            .and_then(runners::run_patent_search),
        "GooglePatentsFetch" => from_value::<patent_search::GooglePatentsFetchInput>(input)
            .and_then(runners::run_google_patents_fetch),
        "HighCitationPatents" => from_value::<patent_search::HighCitationPatentsInput>(input)
            .and_then(|input| runners::run_high_citation_patents(&input)),
        "IterativeSearch" => from_value::<patent_search::IterativeSearchInput>(input)
            .and_then(runners::run_iterative_search),
        "OaParse" => from_value::<patent_oa::OaParseInput>(input)
            .and_then(|input| runners::run_oa_parse(&input)),
        "ResponseTemplate" => from_value::<patent_oa::ResponseTemplateInput>(input)
            .and_then(|input| runners::run_response_template(&input)),
        "SuccessPredictor" => from_value::<patent_oa::SuccessPredictorInput>(input)
            .and_then(|input| runners::run_success_predictor(&input)),
        "SemanticCompare" => from_value::<patent_analysis::SemanticCompareInput>(input)
            .and_then(runners::run_semantic_compare),
        "InfringementAnalysis" => from_value::<patent_analysis::InfringementAnalysisInput>(input)
            .and_then(runners::run_infringement_analysis),
        "SynergyAnalysis" => from_value::<patent_analysis::SynergyAnalysisInput>(input)
            .and_then(runners::run_synergy_analysis),
        "LegalQA" => {
            from_value::<patent_analysis::LegalQAInput>(input).and_then(runners::run_legal_qa)
        }
        "ProcessChart" => from_value::<patent_visualization::ProcessChartInput>(input)
            .and_then(runners::run_process_chart),
        "DrawingUnderstanding" => {
            from_value::<patent_visualization::DrawingUnderstandingInput>(input)
                .and_then(|input| runners::run_drawing_understanding(&input))
        }
        "TechnicalDrawing" => from_value::<patent_visualization::TechnicalDrawingInput>(input)
            .and_then(|input| runners::run_technical_drawing(&input)),
        "PatentManager" => from_value::<patent_management::PatentManagerInput>(input)
            .and_then(runners::run_patent_manager),
        "TemplateLibrary" => from_value::<patent_management::TemplateLibraryInput>(input)
            .and_then(runners::run_template_library),
        "TrademarkAnalysis" => from_value::<patent_management::TrademarkAnalysisInput>(input)
            .and_then(runners::run_trademark_analysis),
        "PatentDownload" => from_value::<patent_management::PatentDownloadInput>(input)
            .and_then(runners::run_patent_download),
        "BatchPatentDownload" => from_value::<patent_management::BatchPatentDownloadInput>(input)
            .and_then(runners::run_batch_patent_download),
        _ => Err(format!("unsupported tool: {name}")),
    }
}

fn from_value<T: for<'de> Deserialize<'de>>(input: &Value) -> Result<T, String> {
    serde_json::from_value(input.clone()).map_err(|error| error.to_string())
}
