//! 斜杠命令中的同步/半同步任务（git、单次 LLM 轮次等），供 REPL 与 TUI 共用。

use runtime::PermissionMode;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::cli_action::AllowedToolSet;
use crate::format_report::{git_output, git_status_ok, recent_user_context, truncate_for_prompt};
use crate::permission_ui::CliPermissionPrompter;
use crate::runtime_bridge::{build_runtime_with_workspace, final_assistant_text};
use crate::tui::turn::SharedRuntime;
use runtime::Session;

/// `/bughunter` 对应的 agent 提示词。
pub(crate) fn bughunter_prompt(scope: Option<&str>) -> String {
    let scope = scope.unwrap_or("the current repository");
    format!(
        "You are /bughunter. Inspect {scope} and identify the most likely bugs or correctness issues. Prioritize concrete findings with file paths, severity, and suggested fixes. Use tools if needed."
    )
}

/// `/ultraplan` 对应的 agent 提示词。
pub(crate) fn ultraplan_prompt(task: Option<&str>) -> String {
    let task = task.unwrap_or("the current repo work");
    format!(
        "You are /ultraplan. Produce a deep multi-step execution plan for {task}. Include goals, risks, implementation sequence, verification steps, and rollback considerations. Use tools if needed."
    )
}

/// 在独立 runtime 上跑一轮 LLM（无会话副作用写入主 session）。
pub(crate) fn run_internal_prompt_text(
    runtime: SharedRuntime,
    model: String,
    system_prompt: Vec<String>,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    prompt: &str,
    enable_tools: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let session = runtime
        .lock()
        .map_err(|_| "runtime lock poisoned")?
        .session()
        .clone();
    let workspace_root = crate::session_mgr::workspace_root().unwrap_or_else(|_| {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });
    run_internal_prompt_text_for_session(
        session,
        model,
        system_prompt,
        allowed_tools,
        permission_mode,
        workspace_root,
        prompt,
        enable_tools,
    )
}

/// 基于给定 Session 跑一轮 LLM（桌面斜杠 / commit 等使用）。
pub(crate) fn run_internal_prompt_text_for_session(
    session: Session,
    model: String,
    system_prompt: Vec<String>,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    workspace_root: PathBuf,
    prompt: &str,
    enable_tools: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut ephemeral = build_runtime_with_workspace(
        session,
        model,
        system_prompt,
        enable_tools,
        false,
        allowed_tools,
        permission_mode,
        workspace_root,
    )?;
    let mut permission_prompter = CliPermissionPrompter::new(permission_mode);
    let summary = ephemeral.run_turn(prompt, Some(&mut permission_prompter))?;
    Ok(final_assistant_text(&summary).trim().to_string())
}

fn workspace_root_or_cwd() -> PathBuf {
    crate::session_mgr::workspace_root()
        .unwrap_or_else(|_| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// 生成 git commit（桌面端 / 给定 Session）。
pub(crate) fn run_commit_for_session(
    session: Session,
    model: String,
    system_prompt: Vec<String>,
    workspace_root: PathBuf,
    permission_mode: PermissionMode,
) -> Result<String, Box<dyn std::error::Error>> {
    let status = git_output(&["status", "--short"])?;
    if status.trim().is_empty() {
        return Ok(
            "Commit\n  Result           skipped\n  Reason           no workspace changes"
                .to_string(),
        );
    }

    git_status_ok(&["add", "-A"])?;
    let staged_stat = git_output(&["diff", "--cached", "--stat"])?;
    let prompt = format!(
        "Generate a git commit message in plain text Lore format only. Base it on this staged diff summary:\n\n{}\n\nRecent conversation context:\n{}",
        truncate_for_prompt(&staged_stat, 8_000),
        recent_user_context(&session, 6)
    );
    let message = sanitize_generated_message(&run_internal_prompt_text_for_session(
        session,
        model,
        system_prompt,
        None,
        permission_mode,
        workspace_root,
        &prompt,
        false,
    )?);
    if message.trim().is_empty() {
        return Err("generated commit message was empty".into());
    }

    let path = write_temp_text_file("yunxi-commit-message.txt", &message)?;
    let output = Command::new("git")
        .args(["commit", "--file"])
        .arg(&path)
        .current_dir(env::current_dir()?)
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("git commit failed: {stderr}").into());
    }

    Ok(format!(
        "Commit\n  Result           created\n  Message file     {}\n\n{}",
        path.display(),
        message.trim()
    ))
}

/// 生成 PR 草稿或创建 PR（桌面端）。
pub(crate) fn run_pr_for_session(
    session: Session,
    model: String,
    system_prompt: Vec<String>,
    workspace_root: PathBuf,
    permission_mode: PermissionMode,
    context: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let staged = git_output(&["diff", "--stat"])?;
    let prompt = format!(
        "Generate a pull request title and body from this conversation and diff summary. Output plain text in this format exactly:\nTITLE: <title>\nBODY:\n<body markdown>\n\nContext hint: {}\n\nDiff summary:\n{}",
        context.unwrap_or("none"),
        truncate_for_prompt(&staged, 10_000)
    );
    let draft = sanitize_generated_message(&run_internal_prompt_text_for_session(
        session,
        model.clone(),
        system_prompt,
        None,
        permission_mode,
        workspace_root,
        &prompt,
        false,
    )?);
    let (title, body) = parse_titled_body(&draft)
        .ok_or_else(|| "failed to parse generated PR title/body".to_string())?;

    if command_exists("gh") {
        let body_path = write_temp_text_file("yunxi-pr-body.md", &body)?;
        let output = Command::new("gh")
            .args(["pr", "create", "--title", &title, "--body-file"])
            .arg(&body_path)
            .current_dir(env::current_dir()?)
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(format!(
                "PR\n  Result           created\n  Title            {title}\n  URL              {}",
                if stdout.is_empty() {
                    "<unknown>"
                } else {
                    &stdout
                }
            ));
        }
    }

    Ok(format!("PR draft\n  Title            {title}\n\n{body}"))
}

/// 生成 Issue 草稿或创建 Issue（桌面端）。
pub(crate) fn run_issue_for_session(
    session: Session,
    model: String,
    system_prompt: Vec<String>,
    workspace_root: PathBuf,
    permission_mode: PermissionMode,
    context: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = format!(
        "Generate a GitHub issue title and body from this conversation. Output plain text in this format exactly:\nTITLE: <title>\nBODY:\n<body markdown>\n\nContext hint: {}\n\nConversation context:\n{}",
        context.unwrap_or("none"),
        truncate_for_prompt(&recent_user_context(&session, 10), 10_000)
    );
    let draft = sanitize_generated_message(&run_internal_prompt_text_for_session(
        session,
        model,
        system_prompt,
        None,
        permission_mode,
        workspace_root,
        &prompt,
        false,
    )?);
    let (title, body) = parse_titled_body(&draft)
        .ok_or_else(|| "failed to parse generated issue title/body".to_string())?;

    if command_exists("gh") {
        let body_path = write_temp_text_file("yunxi-issue-body.md", &body)?;
        let output = Command::new("gh")
            .args(["issue", "create", "--title", &title, "--body-file"])
            .arg(&body_path)
            .current_dir(env::current_dir()?)
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(format!(
                "Issue\n  Result           created\n  Title            {title}\n  URL              {}",
                if stdout.is_empty() { "<unknown>" } else { &stdout }
            ));
        }
    }

    Ok(format!("Issue draft\n  Title            {title}\n\n{body}"))
}

/// 生成 git commit（返回报告文本）。
pub(crate) fn run_commit(
    runtime: SharedRuntime,
    model: String,
    system_prompt: Vec<String>,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
) -> Result<String, Box<dyn std::error::Error>> {
    let _ = allowed_tools;
    let session = runtime
        .lock()
        .map_err(|_| "runtime lock poisoned")?
        .session()
        .clone();
    run_commit_for_session(
        session,
        model,
        system_prompt,
        workspace_root_or_cwd(),
        permission_mode,
    )
}

/// 生成 PR 草稿或创建 PR。
pub(crate) fn run_pr(
    runtime: SharedRuntime,
    model: String,
    system_prompt: Vec<String>,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    context: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let _ = allowed_tools;
    let session = runtime
        .lock()
        .map_err(|_| "runtime lock poisoned")?
        .session()
        .clone();
    run_pr_for_session(
        session,
        model,
        system_prompt,
        workspace_root_or_cwd(),
        permission_mode,
        context,
    )
}

/// 生成 Issue 草稿或创建 Issue。
pub(crate) fn run_issue(
    runtime: SharedRuntime,
    model: String,
    system_prompt: Vec<String>,
    allowed_tools: Option<AllowedToolSet>,
    permission_mode: PermissionMode,
    context: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let _ = allowed_tools;
    let session = runtime
        .lock()
        .map_err(|_| "runtime lock poisoned")?
        .session()
        .clone();
    run_issue_for_session(
        session,
        model,
        system_prompt,
        workspace_root_or_cwd(),
        permission_mode,
        context,
    )
}

fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn write_temp_text_file(
    filename: &str,
    contents: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = env::temp_dir().join(filename);
    fs::write(&path, contents)?;
    Ok(path)
}

fn sanitize_generated_message(value: &str) -> String {
    value.trim().trim_matches('`').trim().replace("\r\n", "\n")
}

fn parse_titled_body(value: &str) -> Option<(String, String)> {
    let normalized = sanitize_generated_message(value);
    let title = normalized
        .lines()
        .find_map(|line| line.strip_prefix("TITLE:").map(str::trim))?;
    let body_start = normalized.find("BODY:")?;
    let body = normalized[body_start + "BODY:".len()..].trim();
    Some((title.to_string(), body.to_string()))
}
