use super::matching::IntentRouter;
use super::parser::identify_scenario_from_input;
use super::types::*;
use crate::SlashCommand;

#[test]
fn explicit_slash_command_passthrough() {
    assert_eq!(
        IntentRouter::recognize("/help"),
        Intent::Command(SlashCommand::Help)
    );
    assert_eq!(
        IntentRouter::recognize("  /status  "),
        Intent::Command(SlashCommand::Status)
    );
}

#[test]
fn negation_returns_chat() {
    assert_eq!(IntentRouter::recognize("don't help me"), Intent::Chat);
    assert_eq!(IntentRouter::recognize("不要清空"), Intent::Chat);
    assert_eq!(IntentRouter::recognize("no cost please"), Intent::Chat);
    assert_eq!(IntentRouter::recognize("这不是帮助"), Intent::Chat);
}

#[test]
fn recognizes_help() {
    assert_eq!(
        IntentRouter::recognize("help"),
        Intent::Command(SlashCommand::Help)
    );
    assert_eq!(
        IntentRouter::recognize("show me the commands"),
        Intent::Command(SlashCommand::Help)
    );
    assert_eq!(
        IntentRouter::recognize("显示帮助"),
        Intent::Command(SlashCommand::Help)
    );
}

#[test]
fn recognizes_compact() {
    assert_eq!(
        IntentRouter::recognize("compact my session"),
        Intent::Command(SlashCommand::Compact)
    );
    assert_eq!(
        IntentRouter::recognize("压缩历史"),
        Intent::Command(SlashCommand::Compact)
    );
}

#[test]
fn recognizes_model() {
    assert_eq!(
        IntentRouter::recognize("switch model to claude-opus"),
        Intent::Command(SlashCommand::Model {
            model: Some("claude-opus".to_string())
        })
    );
    assert_eq!(
        IntentRouter::recognize("换模型"),
        Intent::Command(SlashCommand::Model { model: None })
    );
}

#[test]
fn recognizes_search() {
    assert_eq!(
        IntentRouter::recognize("search for debug"),
        Intent::Command(SlashCommand::Search {
            query: Some("debug".to_string())
        })
    );
}

#[test]
fn recognizes_teleport() {
    assert_eq!(
        IntentRouter::recognize("teleport to conversation.rs"),
        Intent::Command(SlashCommand::Teleport {
            target: Some("conversation.rs".to_string())
        })
    );
}

#[test]
fn plain_chat_returns_chat() {
    assert_eq!(IntentRouter::recognize("hello"), Intent::Chat);
    assert_eq!(IntentRouter::recognize("请帮我写代码"), Intent::Chat);
    assert_eq!(
        IntentRouter::recognize("what is the meaning of life"),
        Intent::Chat
    );
}

#[test]
fn recognizes_chinese_commands() {
    assert_eq!(
        IntentRouter::recognize("当前状态"),
        Intent::Command(SlashCommand::Status)
    );
    assert_eq!(
        IntentRouter::recognize("查看版本"),
        Intent::Command(SlashCommand::Version)
    );
    assert_eq!(
        IntentRouter::recognize("导出对话"),
        Intent::Command(SlashCommand::Export { path: None })
    );
}

#[test]
fn recognizes_cost_and_clear() {
    assert_eq!(
        IntentRouter::recognize("how much did this cost"),
        Intent::Command(SlashCommand::Cost)
    );
    assert_eq!(
        IntentRouter::recognize("清空会话"),
        Intent::Command(SlashCommand::Clear { confirm: false })
    );
}

#[test]
fn recognizes_ultraplan() {
    assert_eq!(
        IntentRouter::recognize("ultraplan to refactor the engine"),
        Intent::Command(SlashCommand::Ultraplan {
            task: Some("refactor the engine".to_string())
        })
    );
}

// ------------------------------------------------------------------------
// Patent intent tests
// ------------------------------------------------------------------------

#[test]
fn recognizes_patent_draft() {
    assert_eq!(
        IntentRouter::recognize("帮我撰写一份专利"),
        Intent::Patent(PatentIntent::Draft)
    );
    assert_eq!(
        IntentRouter::recognize("draft a patent application"),
        Intent::Patent(PatentIntent::Draft)
    );
}

#[test]
fn recognizes_patent_oa_response() {
    assert_eq!(
        IntentRouter::recognize("答复审查意见通知书"),
        Intent::Patent(PatentIntent::OAResponse)
    );
    assert_eq!(
        IntentRouter::recognize("respond to office action"),
        Intent::Patent(PatentIntent::OAResponse)
    );
}

#[test]
fn recognizes_patent_search() {
    assert_eq!(
        IntentRouter::recognize("帮我做一下专利检索"),
        Intent::Patent(PatentIntent::Search)
    );
    assert_eq!(
        IntentRouter::recognize("patent search for ai"),
        Intent::Patent(PatentIntent::Search)
    );
}

#[test]
fn recognizes_patent_quality_check() {
    assert_eq!(
        IntentRouter::recognize("评估这项发明的创造性"),
        Intent::Patent(PatentIntent::QualityCheck)
    );
    assert_eq!(
        IntentRouter::recognize("evaluate innovation"),
        Intent::Patent(PatentIntent::QualityCheck)
    );
}

#[test]
fn recognizes_patent_formality_check() {
    assert_eq!(
        IntentRouter::recognize("检查专利格式"),
        Intent::Patent(PatentIntent::FormalityCheck)
    );
    assert_eq!(
        IntentRouter::recognize("formality check"),
        Intent::Patent(PatentIntent::FormalityCheck)
    );
}

#[test]
fn recognizes_patent_strategy() {
    assert_eq!(
        IntentRouter::recognize("制定专利布局策略"),
        Intent::Patent(PatentIntent::Strategy)
    );
    assert_eq!(
        IntentRouter::recognize("patent strategy"),
        Intent::Patent(PatentIntent::Strategy)
    );
}

#[test]
fn recognizes_patent_compare() {
    assert_eq!(
        IntentRouter::recognize("帮我做一份专利对比分析"),
        Intent::Patent(PatentIntent::Compare)
    );
    assert_eq!(
        IntentRouter::recognize("compare patents"),
        Intent::Patent(PatentIntent::Compare)
    );
}

#[test]
fn patent_negation_returns_chat() {
    assert_eq!(IntentRouter::recognize("不要写专利"), Intent::Chat);
    assert_eq!(IntentRouter::recognize("不要答复审查意见"), Intent::Chat);
}

// ------------------------------------------------------------------------
// Scenario identification tests
// ------------------------------------------------------------------------

#[test]
fn identifies_patent_search_scenario() {
    let ctx = identify_scenario_from_input("请帮我检索人工智能相关的专利");
    assert_eq!(ctx.domain, Domain::Patent);
    assert_eq!(ctx.task_type, TaskType::Search);
    assert_eq!(ctx.suggested_agent_id(), Some("search"));
}

#[test]
fn identifies_oa_examination_phase() {
    let ctx = identify_scenario_from_input("答复审查意见通知书，关于创造性的驳回");
    assert_eq!(ctx.domain, Domain::Patent);
    assert_eq!(ctx.phase, Phase::Examination);
    assert!(ctx.confidence > 0.0);
}

#[test]
fn legal_search_domain_patent() {
    let ctx = identify_scenario_from_input("专利法第二十二条创造性");
    assert_eq!(ctx.legal_search_domain(), Some("patent"));
}

#[test]
fn scenario_identifier_extracts_variables() {
    let ctx = identify_scenario_from_input("技术领域：人工智能，技术问题：如何提高模型精度");
    assert_eq!(ctx.domain, Domain::Patent);
    assert_eq!(
        ctx.extracted_variables.get("technology_field"),
        Some(&"人工智能".to_string())
    );
    assert_eq!(
        ctx.extracted_variables.get("technical_problem"),
        Some(&"如何提高模型精度".to_string())
    );
}

#[test]
fn scenario_identifier_trademark() {
    let ctx = identify_scenario_from_input("商标：华为 第9类");
    assert_eq!(ctx.domain, Domain::Trademark);
    assert_eq!(
        ctx.extracted_variables.get("trademark_name"),
        Some(&"华为 第9类".to_string())
    );
    assert_eq!(
        ctx.extracted_variables.get("trademark_category"),
        Some(&"9".to_string())
    );
}
