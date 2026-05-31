//! 桌面终端：在工作区目录下执行 shell 命令。

use std::path::Path;
use std::process::Stdio;
use std::time::Instant;

use serde::Serialize;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const MAX_COMMAND_LEN: usize = 4000;
const MAX_OUTPUT_CHARS: usize = 64_000;
const EXEC_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn shell_exec(working_dir: String, command: String) -> Result<ShellExecResult, String> {
    let cmd = command.trim();
    if cmd.is_empty() {
        return Err("命令不能为空".to_string());
    }
    if cmd.len() > MAX_COMMAND_LEN {
        return Err("命令过长".to_string());
    }
    if cmd.contains('\0') {
        return Err("非法命令".to_string());
    }

    let dir = Path::new(working_dir.trim());
    if !dir.is_dir() {
        return Err(format!("工作目录不存在: {}", dir.display()));
    }

    let started = Instant::now();
    let child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("启动 shell 失败: {e}"))?;

    let wait = child.wait_with_output();
    let output = timeout(Duration::from_secs(EXEC_TIMEOUT_SECS), wait)
        .await
        .map_err(|_| format!("命令超时（>{EXEC_TIMEOUT_SECS}s）"))?
        .map_err(|e| format!("执行失败: {e}"))?;

    let stdout = truncate_output(String::from_utf8_lossy(&output.stdout).into_owned());
    let stderr = truncate_output(String::from_utf8_lossy(&output.stderr).into_owned());
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(ShellExecResult {
        stdout,
        stderr,
        exit_code,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

fn truncate_output(s: String) -> String {
    if s.chars().count() <= MAX_OUTPUT_CHARS {
        return s;
    }
    let mut out: String = s.chars().take(MAX_OUTPUT_CHARS).collect();
    out.push_str("\n…（输出已截断）");
    out
}
