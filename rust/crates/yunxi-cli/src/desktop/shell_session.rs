//! 交互式 PTY 终端会话（Tauri 事件推送输出）。

use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::state::DesktopState;

/// 单路 shell 会话句柄
pub struct ShellSessionHandle {
    writer: Mutex<Box<dyn Write + Send>>,
    master: Mutex<Box<dyn MasterPty + Send>>,
    kill_tx: Option<mpsc::Sender<()>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShellEvent {
    Output { data: String },
    Exit { code: Option<i32> },
    Error { message: String },
}

pub fn shell_channel(session_id: &str) -> String {
    format!("yunxi://shell/{session_id}")
}

fn next_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("shell-{ms}")
}

/// 启动 PTY 会话（登录 shell，工作目录为 cwd）
#[tauri::command]
pub fn shell_session_start(
    app: AppHandle,
    state: State<'_, Arc<DesktopState>>,
    working_dir: String,
) -> Result<String, String> {
    let dir = Path::new(working_dir.trim());
    if !dir.is_dir() {
        return Err(format!("工作目录不存在: {}", dir.display()));
    }

    // 仅保留一个活跃会话，避免泄漏
    shell_session_close_all(&state)?;

    let session_id = next_session_id();
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 100,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("打开 PTY 失败: {e}"))?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut cmd = CommandBuilder::new(shell);
    cmd.cwd(dir);
    cmd.env("TERM", "xterm-256color");

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("启动 shell 失败: {e}"))?;
    let master = pair.master;
    let mut reader = master
        .try_clone_reader()
        .map_err(|e| format!("PTY 读取端失败: {e}"))?;
    let writer = master
        .take_writer()
        .map_err(|e| format!("PTY 写入端失败: {e}"))?;

    let (kill_tx, kill_rx) = mpsc::channel::<()>();
    let app_read = app.clone();
    let sid_read = session_id.clone();

    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            if kill_rx.try_recv().is_ok() {
                break;
            }
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let _ = app_read.emit(&shell_channel(&sid_read), ShellEvent::Output { data });
                }
                Err(_) => break,
            }
        }
        let _ = app_read.emit(&shell_channel(&sid_read), ShellEvent::Exit { code: None });
    });

    let app_wait = app.clone();
    let sid_wait = session_id.clone();
    thread::spawn(move || {
        let status = child.wait();
        let code = status.ok().map(|s| s.exit_code() as i32);
        let _ = app_wait.emit(&shell_channel(&sid_wait), ShellEvent::Exit { code });
    });

    state
        .shell_sessions
        .lock()
        .expect("shell_sessions lock poisoned")
        .insert(
            session_id.clone(),
            ShellSessionHandle {
                writer: Mutex::new(writer),
                master: Mutex::new(master),
                kill_tx: Some(kill_tx),
            },
        );

    Ok(session_id)
}

/// 调整 PTY 行列（底栏高度 / 窗口宽度变化时调用）
#[tauri::command]
pub fn shell_session_resize(
    state: State<'_, Arc<DesktopState>>,
    session_id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let sessions = state.shell_sessions.lock().map_err(|e| e.to_string())?;
    let handle = sessions
        .get(&session_id)
        .ok_or_else(|| format!("会话不存在: {session_id}"))?;
    let master = handle.master.lock().map_err(|e| e.to_string())?;
    let rows = rows.clamp(8, 120);
    let cols = cols.clamp(40, 300);
    master
        .resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| format!("PTY resize 失败: {e}"))?;
    Ok(())
}

/// 向 PTY 写入数据（前端行模式请自行追加 `\r` 或 `\n`）
#[tauri::command]
pub fn shell_session_write(
    state: State<'_, Arc<DesktopState>>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let sessions = state.shell_sessions.lock().map_err(|e| e.to_string())?;
    let handle = sessions
        .get(&session_id)
        .ok_or_else(|| format!("会话不存在: {session_id}"))?;
    let mut writer = handle.writer.lock().map_err(|e| e.to_string())?;
    writer
        .write_all(data.as_bytes())
        .map_err(|e| format!("写入失败: {e}"))?;
    writer.flush().map_err(|e| format!("flush 失败: {e}"))?;
    Ok(())
}

/// 关闭指定 PTY 会话
#[tauri::command]
pub fn shell_session_close(
    state: State<'_, Arc<DesktopState>>,
    session_id: String,
) -> Result<(), String> {
    let mut sessions = state.shell_sessions.lock().map_err(|e| e.to_string())?;
    if let Some(handle) = sessions.remove(&session_id) {
        if let Some(tx) = handle.kill_tx {
            let _ = tx.send(());
        }
    }
    Ok(())
}

fn shell_session_close_all(state: &State<'_, Arc<DesktopState>>) -> Result<(), String> {
    let mut sessions = state.shell_sessions.lock().map_err(|e| e.to_string())?;
    for (_, handle) in sessions.drain() {
        if let Some(tx) = handle.kill_tx {
            let _ = tx.send(());
        }
    }
    Ok(())
}
