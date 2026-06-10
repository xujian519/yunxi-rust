use std::env;
use std::io;
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command as TokioCommand;
use tokio::runtime::Builder;
use tokio::time::timeout;

use crate::sandbox::{
    build_linux_sandbox_command, resolve_sandbox_status_for_request, FilesystemIsolationMode,
    SandboxConfig, SandboxStatus,
};
use crate::ConfigLoader;

/// Bash 命令输入
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BashCommandInput {
    /// 命令字符串
    pub command: String,
    /// 超时时间（毫秒）
    pub timeout: Option<u64>,
    /// 命令描述
    pub description: Option<String>,
    /// 是否在后台运行
    #[serde(rename = "run_in_background")]
    pub run_in_background: Option<bool>,
    /// 是否禁用沙盒
    #[serde(rename = "dangerouslyDisableSandbox")]
    pub dangerously_disable_sandbox: Option<bool>,
    /// 是否启用命名空间限制
    #[serde(rename = "namespaceRestrictions")]
    pub namespace_restrictions: Option<bool>,
    /// 是否隔离网络
    #[serde(rename = "isolateNetwork")]
    pub isolate_network: Option<bool>,
    /// 文件系统模式
    #[serde(rename = "filesystemMode")]
    pub filesystem_mode: Option<FilesystemIsolationMode>,
    /// 允许挂载点
    #[serde(rename = "allowedMounts")]
    pub allowed_mounts: Option<Vec<String>>,
}

/// Bash 命令输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BashCommandOutput {
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 原始输出路径
    #[serde(rename = "rawOutputPath")]
    pub raw_output_path: Option<String>,
    /// 是否被中断
    pub interrupted: bool,
    /// 是否为图片输出
    #[serde(rename = "isImage")]
    pub is_image: Option<bool>,
    /// 后台任务 ID
    #[serde(rename = "backgroundTaskId")]
    pub background_task_id: Option<String>,
    /// 是否由用户后台运行
    #[serde(rename = "backgroundedByUser")]
    pub backgrounded_by_user: Option<bool>,
    /// 是否由助手自动后台运行
    #[serde(rename = "assistantAutoBackgrounded")]
    pub assistant_auto_backgrounded: Option<bool>,
    /// 是否禁用沙盒
    #[serde(rename = "dangerouslyDisableSandbox")]
    pub dangerously_disable_sandbox: Option<bool>,
    /// 返回码解释
    #[serde(rename = "returnCodeInterpretation")]
    pub return_code_interpretation: Option<String>,
    /// 无输出预期
    #[serde(rename = "noOutputExpected")]
    pub no_output_expected: Option<bool>,
    /// 结构化内容
    #[serde(rename = "structuredContent")]
    pub structured_content: Option<Vec<serde_json::Value>>,
    /// 持久化输出路径
    #[serde(rename = "persistedOutputPath")]
    pub persisted_output_path: Option<String>,
    /// 持久化输出大小
    #[serde(rename = "persistedOutputSize")]
    pub persisted_output_size: Option<u64>,
    /// 沙盒状态
    #[serde(rename = "sandboxStatus")]
    pub sandbox_status: Option<SandboxStatus>,
}

/// 命令最大长度限制
const MAX_COMMAND_LENGTH: usize = 100_000;

/// 危险命令模式 — 禁止不可逆的破坏性操作
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "mkfs.",
    "dd if=",
    "> /dev/sd",
    "chmod 000",
    "chown root",
    ":(){ :|:& };:",
];

/// 验证命令安全性
fn validate_command(command: &str) -> io::Result<()> {
    if command.len() > MAX_COMMAND_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("命令超过最大长度限制 ({MAX_COMMAND_LENGTH} 字符)"),
        ));
    }
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "命令不能为空"));
    }
    let lower = trimmed.to_ascii_lowercase();
    for pattern in DANGEROUS_PATTERNS {
        if lower.contains(pattern) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("命令包含禁止的危险模式: {pattern}"),
            ));
        }
    }
    Ok(())
}

/// 执行 Bash 命令
///
/// # 参数
/// - `input`: Bash 命令输入
///
/// # 返回
/// Bash 命令输出
///
/// # Errors
///
/// - 如果当前目录不可访问,返回 IO 错误
/// - 如果命令执行失败,返回 IO 错误
pub fn execute_bash(input: BashCommandInput) -> io::Result<BashCommandOutput> {
    validate_command(&input.command)?;
    let cwd = env::current_dir()?;
    let sandbox_status = sandbox_status_for_input(&input, &cwd);

    if input.run_in_background.unwrap_or(false) {
        let mut child = prepare_command(&input.command, &cwd, &sandbox_status, false);
        let child = child
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        return Ok(BashCommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            raw_output_path: None,
            interrupted: false,
            is_image: None,
            background_task_id: Some(child.id().to_string()),
            backgrounded_by_user: Some(false),
            assistant_auto_backgrounded: Some(false),
            dangerously_disable_sandbox: input.dangerously_disable_sandbox,
            return_code_interpretation: None,
            no_output_expected: Some(true),
            structured_content: None,
            persisted_output_path: None,
            persisted_output_size: None,
            sandbox_status: Some(sandbox_status),
        });
    }

    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        tokio::task::block_in_place(|| {
            handle.block_on(execute_bash_async(input, sandbox_status, cwd))
        })
    } else {
        let runtime = Builder::new_current_thread().enable_all().build()?;
        runtime.block_on(execute_bash_async(input, sandbox_status, cwd))
    }
}

async fn execute_bash_async(
    input: BashCommandInput,
    sandbox_status: SandboxStatus,
    cwd: std::path::PathBuf,
) -> io::Result<BashCommandOutput> {
    let mut command = prepare_tokio_command(&input.command, &cwd, &sandbox_status, true);

    let output_result = if let Some(timeout_ms) = input.timeout {
        match timeout(Duration::from_millis(timeout_ms), command.output()).await {
            Ok(result) => (result?, false),
            Err(_) => {
                return Ok(BashCommandOutput {
                    stdout: String::new(),
                    stderr: format!("Command exceeded timeout of {timeout_ms} ms"),
                    raw_output_path: None,
                    interrupted: true,
                    is_image: None,
                    background_task_id: None,
                    backgrounded_by_user: None,
                    assistant_auto_backgrounded: None,
                    dangerously_disable_sandbox: input.dangerously_disable_sandbox,
                    return_code_interpretation: Some(String::from("timeout")),
                    no_output_expected: Some(true),
                    structured_content: None,
                    persisted_output_path: None,
                    persisted_output_size: None,
                    sandbox_status: Some(sandbox_status),
                });
            }
        }
    } else {
        (command.output().await?, false)
    };

    let (output, interrupted) = output_result;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let no_output_expected = Some(stdout.trim().is_empty() && stderr.trim().is_empty());
    let return_code_interpretation = output.status.code().and_then(|code| {
        if code == 0 {
            None
        } else {
            Some(format!("exit_code:{code}"))
        }
    });

    Ok(BashCommandOutput {
        stdout,
        stderr,
        raw_output_path: None,
        interrupted,
        is_image: None,
        background_task_id: None,
        backgrounded_by_user: None,
        assistant_auto_backgrounded: None,
        dangerously_disable_sandbox: input.dangerously_disable_sandbox,
        return_code_interpretation,
        no_output_expected,
        structured_content: None,
        persisted_output_path: None,
        persisted_output_size: None,
        sandbox_status: Some(sandbox_status),
    })
}

fn sandbox_status_for_input(input: &BashCommandInput, cwd: &std::path::Path) -> SandboxStatus {
    let config = ConfigLoader::default_for(cwd).load().map_or_else(
        |_| SandboxConfig::default(),
        |runtime_config| runtime_config.sandbox().clone(),
    );
    let request = config.resolve_request(
        input.dangerously_disable_sandbox.map(|disabled| !disabled),
        input.namespace_restrictions,
        input.isolate_network,
        input.filesystem_mode,
        input.allowed_mounts.clone(),
    );
    resolve_sandbox_status_for_request(&request, cwd)
}

fn prepare_command(
    command: &str,
    cwd: &std::path::Path,
    sandbox_status: &SandboxStatus,
    create_dirs: bool,
) -> Command {
    if create_dirs {
        prepare_sandbox_dirs(cwd);
    }

    if let Some(launcher) = build_linux_sandbox_command(command, cwd, sandbox_status) {
        let mut prepared = Command::new(launcher.program);
        prepared.args(launcher.args);
        prepared.current_dir(cwd);
        prepared.envs(launcher.env);
        return prepared;
    }

    let mut prepared = Command::new("sh");
    prepared.arg("-lc").arg(command).current_dir(cwd);
    if sandbox_status.filesystem_active {
        prepared.env("HOME", cwd.join(".sandbox-home"));
        prepared.env("TMPDIR", cwd.join(".sandbox-tmp"));
    }
    prepared
}

fn prepare_tokio_command(
    command: &str,
    cwd: &std::path::Path,
    sandbox_status: &SandboxStatus,
    create_dirs: bool,
) -> TokioCommand {
    if create_dirs {
        prepare_sandbox_dirs(cwd);
    }

    if let Some(launcher) = build_linux_sandbox_command(command, cwd, sandbox_status) {
        let mut prepared = TokioCommand::new(launcher.program);
        prepared.args(launcher.args);
        prepared.current_dir(cwd);
        prepared.envs(launcher.env);
        return prepared;
    }

    let mut prepared = TokioCommand::new("sh");
    prepared.arg("-lc").arg(command).current_dir(cwd);
    if sandbox_status.filesystem_active {
        prepared.env("HOME", cwd.join(".sandbox-home"));
        prepared.env("TMPDIR", cwd.join(".sandbox-tmp"));
    }
    prepared
}

fn prepare_sandbox_dirs(cwd: &std::path::Path) {
    let _ = std::fs::create_dir_all(cwd.join(".sandbox-home"));
    let _ = std::fs::create_dir_all(cwd.join(".sandbox-tmp"));
}

#[cfg(test)]
mod tests {
    use super::{execute_bash, BashCommandInput};
    use crate::sandbox::FilesystemIsolationMode;

    #[test]
    fn executes_simple_command() {
        let output = execute_bash(BashCommandInput {
            command: String::from("printf 'hello'"),
            timeout: Some(1_000),
            description: None,
            run_in_background: Some(false),
            dangerously_disable_sandbox: Some(false),
            namespace_restrictions: Some(false),
            isolate_network: Some(false),
            filesystem_mode: Some(FilesystemIsolationMode::WorkspaceOnly),
            allowed_mounts: None,
        })
        .expect("bash command should execute");

        assert_eq!(output.stdout, "hello");
        assert!(!output.interrupted);
        assert!(output.sandbox_status.is_some());
    }

    #[test]
    fn disables_sandbox_when_requested() {
        let output = execute_bash(BashCommandInput {
            command: String::from("printf 'hello'"),
            timeout: Some(1_000),
            description: None,
            run_in_background: Some(false),
            dangerously_disable_sandbox: Some(true),
            namespace_restrictions: None,
            isolate_network: None,
            filesystem_mode: None,
            allowed_mounts: None,
        })
        .expect("bash command should execute");

        assert!(!output.sandbox_status.expect("sandbox status").enabled);
    }

    #[test]
    fn test_echo_command_produces_expected_output() {
        let output = execute_bash(BashCommandInput {
            command: String::from("echo 'test output'"),
            timeout: Some(1_000),
            description: None,
            run_in_background: Some(false),
            dangerously_disable_sandbox: Some(true),
            namespace_restrictions: None,
            isolate_network: None,
            filesystem_mode: None,
            allowed_mounts: None,
        })
        .expect("echo command should execute");

        assert_eq!(output.stdout.trim(), "test output");
        assert!(!output.interrupted);
        assert!(output.return_code_interpretation.is_none());
    }

    #[test]
    fn test_true_command_exits_successfully() {
        let output = execute_bash(BashCommandInput {
            command: String::from("true"),
            timeout: Some(1_000),
            description: None,
            run_in_background: Some(false),
            dangerously_disable_sandbox: Some(true),
            namespace_restrictions: None,
            isolate_network: None,
            filesystem_mode: None,
            allowed_mounts: None,
        })
        .expect("true command should execute");

        assert!(output.stdout.trim().is_empty());
        assert!(output.return_code_interpretation.is_none());
    }

    #[test]
    fn test_false_command_exits_with_error() {
        let output = execute_bash(BashCommandInput {
            command: String::from("false"),
            timeout: Some(1_000),
            description: None,
            run_in_background: Some(false),
            dangerously_disable_sandbox: Some(true),
            namespace_restrictions: None,
            isolate_network: None,
            filesystem_mode: None,
            allowed_mounts: None,
        })
        .expect("false command should execute");

        assert!(output.return_code_interpretation.is_some());
        assert_eq!(
            output.return_code_interpretation.expect("return code"),
            "exit_code:1"
        );
    }
}
