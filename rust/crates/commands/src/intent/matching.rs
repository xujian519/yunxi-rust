use crate::SlashCommand;

use super::parser::identify_scenario_from_input;
use super::types::{Intent, PatentIntent, ScenarioContext};

// ============================================================================
// Intent Router
// ============================================================================

/// Rule-based intent router that maps natural language to slash commands
/// or patent intents.
///
/// The router operates in four stages:
/// 1. **Explicit command** — if input starts with `/`, parse as `SlashCommand` directly.
/// 2. **Negation guard** — if the input contains negation words (e.g. "don't", "不要"),
///    skip all keyword matching and return `Chat`.
/// 3. **Keyword matching** — scan for command-specific trigger words and synonyms.
/// 4. **Patent keyword matching** — scan for patent-specific trigger words.
///
/// No LLM calls are made; recognition is deterministic and local.
pub struct IntentRouter;

impl IntentRouter {
    /// Recognize intent from a user message.
    ///
    /// Returns `Intent::Command` when the message clearly expresses a command intent,
    /// `Intent::Patent` when a patent workflow is detected, or `Intent::Chat` for
    /// ordinary conversation (or negated requests).
    #[must_use]
    pub fn recognize(input: &str) -> Intent {
        // Stage 1: explicit slash command
        if let Some(cmd) = SlashCommand::parse(input) {
            return Intent::Command(cmd);
        }

        let lower = input.to_lowercase();

        // Stage 2: negation guard
        if is_negated(&lower) {
            return Intent::Chat;
        }

        // Stage 3 & 4: keyword matching
        Self::match_keywords(&lower)
    }

    /// Recognize intent together with legal scenario context.
    #[must_use]
    pub fn recognize_with_scenario(input: &str) -> (Intent, ScenarioContext) {
        let intent = Self::recognize(input);
        let scenario = identify_scenario_from_input(input);
        (intent, scenario)
    }

    fn match_keywords(lower: &str) -> Intent {
        if let Some(intent) = match_basic_command_keywords(lower) {
            return intent;
        }
        if let Some(intent) = match_management_command_keywords(lower) {
            return intent;
        }
        if let Some(intent) = match_search_keywords(lower) {
            return intent;
        }
        if let Some(intent) = match_git_session_keywords(lower) {
            return intent;
        }
        if let Some(intent) = match_planning_debug_keywords(lower) {
            return intent;
        }
        if let Some(intent) = match_patent_keywords(lower) {
            return intent;
        }
        Intent::Chat
    }
}

fn match_basic_command_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "help",
            "帮助",
            "怎么用",
            "用法",
            "commands",
            "list commands",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Help));
    }
    if has_any(lower, &["status", "状态", "当前状态", "session status"]) {
        return Some(Intent::Command(SlashCommand::Status));
    }
    if has_any(
        lower,
        &[
            "compact",
            "压缩",
            "summary",
            "summarize",
            "总结",
            "精简",
            "压缩历史",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Compact));
    }
    None
}

fn match_management_command_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &["model", "模型", "switch model", "换模型", "切换模型"],
    ) {
        let model = extract_after(lower, &["to ", "为 ", "成 ", "换成 ", "switch to ", "use "]);
        return Some(Intent::Command(SlashCommand::Model { model }));
    }
    if has_any(
        lower,
        &[
            "permissions",
            "权限",
            "permission",
            "安全模式",
            "permission mode",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Permissions { mode: None }));
    }
    if has_any(
        lower,
        &[
            "clear",
            "清空",
            "新会话",
            "重新开始",
            "reset session",
            "clear session",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Clear { confirm: false }));
    }
    if has_any(
        lower,
        &[
            "cost", "费用", "token", "用量", "花费", "price", "pricing", "how much",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Cost));
    }
    if has_any(
        lower,
        &["config", "配置", "settings", "设置", "configuration"],
    ) {
        return Some(Intent::Command(SlashCommand::Config { section: None }));
    }
    if has_any(
        lower,
        &["memory", "记忆", "memories", "指导文件", "memory files"],
    ) {
        return Some(Intent::Command(SlashCommand::Memory));
    }
    if has_any(lower, &["version", "版本", "版本号"]) {
        return Some(Intent::Command(SlashCommand::Version));
    }
    None
}

fn match_search_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "专利检索",
            "新颖性检索",
            "可专利性检索",
            "patent search",
            "国知局检索",
            "CNIPA检索",
            "cnipa检索",
            "公布公告检索",
            "查新",
            "现有技术检索",
            "对比文件检索",
            "查一下专利",
            "检索专利",
            "专利查新",
        ],
    ) {
        return Some(Intent::Patent(PatentIntent::Search));
    }
    if has_any(
        lower,
        &[
            "search",
            "搜索",
            "查找",
            "find",
            "search history",
            "查找历史",
        ],
    ) {
        let query = extract_after(lower, &["for ", "搜索", "查找", "找"]);
        return Some(Intent::Command(SlashCommand::Search { query }));
    }
    None
}

fn match_git_session_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &["undo", "撤销", "撤回", "上一步", "回退", "rollback"],
    ) {
        return Some(Intent::Command(SlashCommand::Undo));
    }
    if has_any(
        lower,
        &[
            "session",
            "会话",
            "切换会话",
            "列表",
            "sessions",
            "list sessions",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Session {
            action: Some("list".to_string()),
            target: None,
        }));
    }
    if let Some(intent) = match_export_diff_init(lower) {
        return Some(intent);
    }
    if let Some(intent) = match_commit_pr_issue_bughunter(lower) {
        return Some(intent);
    }
    None
}

fn match_export_diff_init(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &["export", "导出", "save", "保存", "export conversation"],
    ) {
        return Some(Intent::Command(SlashCommand::Export { path: None }));
    }
    if has_any(
        lower,
        &["diff", "git diff", "changes", "改动", "修改", "工作区"],
    ) {
        return Some(Intent::Command(SlashCommand::Diff));
    }
    if has_any(
        lower,
        &[
            "init",
            "初始化",
            "生成 claude.md",
            "生成 readme",
            "generate claude.md",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Init));
    }
    None
}

fn match_commit_pr_issue_bughunter(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "commit",
            "提交",
            "git commit",
            "create commit",
            "写提交信息",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Commit));
    }
    if has_any(
        lower,
        &["pr", "pull request", "创建 pr", "draft pr", "起草 pr"],
    ) {
        return Some(Intent::Command(SlashCommand::Pr { context: None }));
    }
    if has_any(
        lower,
        &[
            "issue",
            "创建 issue",
            "draft issue",
            "起草 issue",
            "bug report",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Issue { context: None }));
    }
    if has_any(
        lower,
        &[
            "bughunter",
            "找 bug",
            "检查缺陷",
            "bug hunt",
            "code review",
            "代码审查",
        ],
    ) {
        return Some(Intent::Command(SlashCommand::Bughunter { scope: None }));
    }
    None
}

fn match_planning_debug_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &["teleport", "跳转", "跳转到", "goto", "go to", "navigate to"],
    ) {
        let target = extract_after(lower, &["to ", "跳转到", "goto ", "go to ", "navigate to "]);
        return Some(Intent::Command(SlashCommand::Teleport { target }));
    }
    if has_any(
        lower,
        &["ultraplan", "深度规划", "规划任务", "deep plan", "制定计划"],
    ) {
        let task = extract_after(lower, &["to ", "for ", "任务：", "任务:", "plan "]);
        return Some(Intent::Command(SlashCommand::Ultraplan { task }));
    }
    if has_any(
        lower,
        &["resume", "恢复", "加载会话", "load session", "继续会话"],
    ) {
        return Some(Intent::Command(SlashCommand::Resume { session_path: None }));
    }
    if has_any(
        lower,
        &["debug-tool-call", "调试", "debug", "调试工具", "tool debug"],
    ) {
        return Some(Intent::Command(SlashCommand::DebugToolCall));
    }
    None
}

fn match_patent_keywords(lower: &str) -> Option<Intent> {
    if let Some(intent) = match_drafting_keywords(lower) {
        return Some(intent);
    }
    if let Some(intent) = match_oa_keywords(lower) {
        return Some(intent);
    }
    if let Some(intent) = match_quality_keywords(lower) {
        return Some(intent);
    }
    if let Some(intent) = match_formality_keywords(lower) {
        return Some(intent);
    }
    if let Some(intent) = match_strategy_keywords(lower) {
        return Some(intent);
    }
    match_compare_keywords(lower)
}

fn match_drafting_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "撰写专利",
            "写专利",
            "专利撰写",
            "专利申请",
            "写一份专利",
            "draft patent",
            "write patent",
            "patent application",
            "new patent",
            "prepare patent",
            "file patent",
            "全套申请文件",
            "权利要求书",
            "说明书撰写",
            "专利起草",
            "invention disclosure",
            "起草专利",
            "准备专利",
            "专利申请文件",
        ],
    ) {
        return Some(Intent::Patent(PatentIntent::Draft));
    }
    None
}

fn match_oa_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "审查意见",
            "答复审查",
            "审查意见答复",
            "oa答复",
            "一通",
            "二通",
            "意见陈述书",
            "审查通知书",
            "驳回决定",
            "respond to office action",
            "office action",
            "official action",
            "examination opinion",
            "rejection",
            "claim rejection",
            "答辩",
            "申述",
            "答复审查员",
            "审查意见通知",
            "补正",
        ],
    ) {
        return Some(Intent::Patent(PatentIntent::OAResponse));
    }
    None
}

fn match_quality_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "创新评估",
            "可专利性评估",
            "创造性评估",
            "技术高度",
            "值得申请",
            "质量检查",
            "专利质量",
            "质量评估",
            "quality check",
            "evaluate innovation",
            "creativity assessment",
            "创造性",
            "可专利性",
            "创新性",
            "专利评估",
            "评估专利",
            "发明评估",
        ],
    ) {
        return Some(Intent::Patent(PatentIntent::QualityCheck));
    }
    None
}

fn match_formality_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "格式检查",
            "形式审查",
            "格式合规",
            "说明书格式",
            "形式检查",
            "formality check",
            "format check",
            "格式审查",
            "检查专利",
            "检查格式",
            "专利格式",
            "形式合规",
            "合规检查",
        ],
    ) {
        return Some(Intent::Patent(PatentIntent::FormalityCheck));
    }
    None
}

fn match_strategy_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "专利策略",
            "布局策略",
            "申请策略",
            "专利布局",
            "技术布局",
            "patent strategy",
            "portfolio strategy",
            "ip strategy",
            "布局专利",
            "专利战略",
            "ip布局",
        ],
    ) {
        return Some(Intent::Patent(PatentIntent::Strategy));
    }
    None
}

fn match_compare_keywords(lower: &str) -> Option<Intent> {
    if has_any(
        lower,
        &[
            "专利对比",
            "技术特征对比",
            "侵权对比",
            "对比分析",
            "compare patents",
            "patent comparison",
            "对比专利",
            "技术比对",
            "比对专利",
            "专利比对",
            "技术对比",
            "专利比较",
        ],
    ) {
        return Some(Intent::Patent(PatentIntent::Compare));
    }
    None
}

// ============================================================================
// Helper functions
// ============================================================================

fn is_negated(lower: &str) -> bool {
    const NEGATIONS: &[&str] = &["don't", "do not", "never", "不要", "别", "从未"];
    NEGATIONS.iter().any(|word| lower.contains(word))
        || is_standalone_word(lower, "no")
        || is_standalone_word(lower, "not")
        || is_standalone_word(lower, "none")
        || is_standalone_word(lower, "不")
}

/// Check whether `word` appears as a standalone token in `text`.
/// A word is standalone when it is surrounded by word boundaries (space, punctuation,
/// or string start/end). Chinese characters are treated as word boundaries.
fn is_standalone_word(text: &str, word: &str) -> bool {
    let word_lower = word.to_ascii_lowercase();
    let text_lower = text.to_ascii_lowercase();
    let mut start = 0;
    while let Some(pos) = text_lower[start..].find(&word_lower) {
        let abs = start + pos;
        let before = text_lower[..abs].chars().last();
        let after = text_lower[abs + word_lower.len()..].chars().next();
        let is_boundary = |c: Option<char>| {
            c.is_none()
                || c.unwrap().is_ascii_whitespace()
                || c.unwrap().is_ascii_punctuation()
                || !c.unwrap().is_ascii()
        };
        if is_boundary(before) && is_boundary(after) {
            return true;
        }
        start = abs + word_lower.len();
    }
    false
}

fn has_any(lower: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| {
        if keyword.chars().all(|c| c.is_ascii_alphanumeric()) {
            is_standalone_word(lower, keyword)
        } else {
            lower.contains(keyword)
        }
    })
}

/// Try to extract a trailing argument after one of the given prefixes.
/// Prefers the right-most match so that "search for foo for bar" returns "bar".
fn extract_after(lower: &str, prefixes: &[&str]) -> Option<String> {
    let mut best_pos: Option<usize> = None;
    let mut best_value: Option<String> = None;
    for prefix in prefixes {
        if let Some(pos) = lower.rfind(prefix) {
            let start = pos + prefix.len();
            let value = lower[start..].trim();
            if !value.is_empty() && best_pos.is_none_or(|p| pos > p) {
                best_pos = Some(pos);
                best_value = Some(value.to_string());
            }
        }
    }
    best_value
}