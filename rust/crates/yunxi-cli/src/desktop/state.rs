//! 桌面端 Tauri 共享状态：权限决策、取消令牌、PTY、工作区监视。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use notify::RecommendedWatcher;
use runtime::PermissionRequest;

use crate::shell_session::ShellSessionHandle;

/// 桌面应用全局状态。
pub struct DesktopState {
    pub cancel_flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub permission_waiters: Mutex<HashMap<String, std::sync::mpsc::Sender<bool>>>,
    pub shell_sessions: Mutex<HashMap<String, ShellSessionHandle>>,
    pub workspace_watcher: Mutex<Option<RecommendedWatcher>>,
}

impl DesktopState {
    pub fn new() -> Self {
        Self {
            cancel_flags: Mutex::new(HashMap::new()),
            permission_waiters: Mutex::new(HashMap::new()),
            shell_sessions: Mutex::new(HashMap::new()),
            workspace_watcher: Mutex::new(None),
        }
    }

    pub fn cancel_flag(&self, session_id: &str) -> Arc<AtomicBool> {
        let mut flags = self
            .cancel_flags
            .lock()
            .expect("cancel_flags lock poisoned");
        flags
            .entry(session_id.to_string())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    }

    pub fn reset_cancel(&self, session_id: &str) {
        self.cancel_flag(session_id).store(false, Ordering::SeqCst);
    }

    pub fn request_cancel(&self, session_id: &str) {
        self.cancel_flag(session_id).store(true, Ordering::SeqCst);
        if let Ok(mut waiters) = self.permission_waiters.lock() {
            if let Some(tx) = waiters.remove(session_id) {
                let _ = tx.send(false);
            }
        }
    }

    pub fn register_permission_waiter(&self, request_id: &str, tx: std::sync::mpsc::Sender<bool>) {
        self.permission_waiters
            .lock()
            .expect("permission_waiters lock poisoned")
            .insert(request_id.to_string(), tx);
    }

    pub fn resolve_permission(&self, request_id: &str, allow: bool) -> bool {
        self.permission_waiters
            .lock()
            .ok()
            .and_then(|mut waiters| waiters.remove(request_id).map(|tx| (tx, allow)))
            .map(|(tx, allow)| tx.send(allow).is_ok())
            .unwrap_or(false)
    }
}

impl Default for DesktopState {
    fn default() -> Self {
        Self::new()
    }
}

/// 将权限请求转为前端可展示的 request_id。
pub fn permission_request_id(request: &PermissionRequest) -> String {
    format!(
        "{}:{}",
        request.tool_name,
        request.input.chars().take(32).collect::<String>()
    )
}
