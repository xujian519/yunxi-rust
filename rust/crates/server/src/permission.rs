//! HTTP Server 权限等待与提示器。

use std::collections::HashMap;
use std::sync::{Arc, Mutex, mpsc};

use runtime::{PermissionPromptDecision, PermissionPrompter, PermissionRequest};

use crate::ws_stream::StreamEvent;

pub type PermissionWaiters = Arc<Mutex<HashMap<String, mpsc::Sender<bool>>>>;

pub fn permission_request_id(request: &PermissionRequest) -> String {
    format!(
        "{}:{}",
        request.tool_name,
        request.input.chars().take(32).collect::<String>()
    )
}

pub fn resolve_permission(waiters: &PermissionWaiters, request_id: &str, allow: bool) -> bool {
    waiters
        .lock()
        .ok()
        .and_then(|mut map| map.remove(request_id).map(|tx| (tx, allow)))
        .map(|(tx, allow)| tx.send(allow).is_ok())
        .unwrap_or(false)
}

/// 阻塞等待前端通过 WebSocket / REST 响应权限。
pub struct ServerPermissionPrompter {
    waiters: PermissionWaiters,
    on_event: Box<dyn Fn(StreamEvent) + Send>,
}

impl ServerPermissionPrompter {
    pub fn new(
        waiters: PermissionWaiters,
        on_event: impl Fn(StreamEvent) + Send + 'static,
    ) -> Self {
        Self {
            waiters,
            on_event: Box::new(on_event),
        }
    }
}

impl PermissionPrompter for ServerPermissionPrompter {
    fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision {
        let request_id = permission_request_id(request);
        let (tx, rx) = mpsc::channel();
        if let Ok(mut waiters) = self.waiters.lock() {
            waiters.insert(request_id.clone(), tx);
        }

        (self.on_event)(StreamEvent::PermissionRequest {
            request_id: request_id.clone(),
            tool: request.tool_name.clone(),
            input: request.input.clone(),
        });

        match rx.recv() {
            Ok(true) => PermissionPromptDecision::Allow,
            Ok(false) | Err(_) => PermissionPromptDecision::Deny {
                reason: format!("tool '{}' denied by user", request.tool_name),
            },
        }
    }
}
