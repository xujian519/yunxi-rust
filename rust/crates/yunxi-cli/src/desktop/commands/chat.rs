//! 流式对话 IPC。

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{mpsc, Arc};

use runtime::{
    AssistantEvent, PermissionMode, PermissionPromptDecision, PermissionPrompter,
    PermissionRequest, Session,
};
use serde::Serialize;
use tauri::{AppHandle, State};
use yunxi_cli::model_routing::{
    default_model_from_config, load_router_config, select_model_for_request,
};
use yunxi_cli::normalize_startup_model;
use yunxi_cli::runtime_bridge::{build_runtime_with_workspace, build_system_prompt_for};
use yunxi_cli::session_mgr::resolve_session_reference;

use crate::state::{permission_request_id, DesktopState};
use crate::stream::{
    emit_assistant_event, emit_stream, emit_tool_results_from_summary, StreamEvent,
};

#[derive(Debug, Serialize)]
pub struct ChatSendResult {
    pub turn_id: String,
    pub session_id: String,
}

struct DesktopPrompter {
    app: AppHandle,
    session_id: String,
    state: Arc<DesktopState>,
}

impl PermissionPrompter for DesktopPrompter {
    fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision {
        let request_id = permission_request_id(request);
        let (tx, rx) = mpsc::channel();
        self.state.register_permission_waiter(&request_id, tx);

        emit_stream(
            &self.app,
            &self.session_id,
            StreamEvent::PermissionRequest {
                request_id: request_id.clone(),
                tool: request.tool_name.clone(),
                input: request.input.clone(),
            },
        );

        match rx.recv() {
            Ok(true) => PermissionPromptDecision::Allow,
            Ok(false) | Err(_) => PermissionPromptDecision::Deny {
                reason: format!(
                    "tool '{}' denied by user approval prompt",
                    request.tool_name
                ),
            },
        }
    }
}

#[tauri::command]
pub async fn chat_send(
    app: AppHandle,
    state: State<'_, Arc<DesktopState>>,
    session_id: String,
    content: String,
    case_id: Option<String>,
    workspace_root: Option<String>,
) -> Result<ChatSendResult, String> {
    let _ = case_id;
    let cancel_flag = state.cancel_flag(&session_id);
    state.reset_cancel(&session_id);

    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let state_arc = Arc::clone(&state);
    let ws_root = workspace_root.map(PathBuf::from);

    let turn_id = format!(
        "turn-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let turn_id_clone = turn_id.clone();

    tauri::async_runtime::spawn_blocking(move || {
        run_chat_turn(
            &app_clone,
            &session_id_clone,
            &content,
            &turn_id_clone,
            cancel_flag,
            state_arc,
            ws_root,
        )
    })
    .await
    .map_err(|e| e.to_string())?
}

fn run_chat_turn(
    app: &AppHandle,
    session_id: &str,
    content: &str,
    turn_id: &str,
    cancel_flag: Arc<std::sync::atomic::AtomicBool>,
    desktop_state: Arc<DesktopState>,
    workspace_root: Option<PathBuf>,
) -> Result<ChatSendResult, String> {
    let handle = resolve_session_reference(session_id).map_err(|e| e.to_string())?;
    let session = Session::load_from_path(&handle.path).map_err(|e| e.to_string())?;

    let configured_model = default_model_from_config();
    let router = load_router_config();
    let model = normalize_startup_model(&select_model_for_request(
        &configured_model,
        content,
        session.messages.len(),
        0,
        &router,
    ));

    let root = workspace_root.unwrap_or_else(|| {
        yunxi_cli::session_mgr::workspace_root().unwrap_or_else(|_| PathBuf::from("."))
    });
    let system_prompt = build_system_prompt_for(root.clone()).map_err(|e| e.to_string())?;
    // 桌面端默认允许工具执行；后续可从 settings 读取权限模式
    let permission_mode = PermissionMode::DangerFullAccess;

    let mut runtime = build_runtime_with_workspace(
        session,
        model,
        system_prompt,
        true,
        false,
        None,
        permission_mode,
        root,
    )
    .map_err(|e| e.to_string())?;

    let mut prompter = DesktopPrompter {
        app: app.clone(),
        session_id: session_id.to_string(),
        state: desktop_state,
    };

    let app_for_stream = app.clone();
    let sid = session_id.to_string();
    let on_stream: Box<dyn FnMut(AssistantEvent) + Send> = Box::new(move |event| {
        emit_assistant_event(&app_for_stream, &sid, event);
    });

    let result = runtime.run_turn_with_stream(content, Some(&mut prompter), Some(on_stream));

    let summary = result.map_err(|e| {
        let message = e.to_string();
        emit_stream(
            app,
            session_id,
            StreamEvent::Error {
                message: message.clone(),
            },
        );
        message
    })?;

    if cancel_flag.load(Ordering::SeqCst) {
        emit_stream(
            app,
            session_id,
            StreamEvent::Error {
                message: "cancelled".to_string(),
            },
        );
        return Err("cancelled".to_string());
    }

    emit_tool_results_from_summary(app, session_id, &summary);
    emit_stream(app, session_id, StreamEvent::MessageStop);

    runtime
        .session()
        .save_to_path(&handle.path)
        .map_err(|e| e.to_string())?;

    Ok(ChatSendResult {
        turn_id: turn_id.to_string(),
        session_id: session_id.to_string(),
    })
}

#[tauri::command]
pub fn chat_cancel(state: State<'_, Arc<DesktopState>>, session_id: String) -> Result<(), String> {
    state.request_cancel(&session_id);
    Ok(())
}

#[tauri::command]
pub fn permission_respond(
    state: State<'_, Arc<DesktopState>>,
    request_id: String,
    outcome: String,
) -> Result<(), String> {
    let allow = matches!(outcome.as_str(), "allow" | "always");
    if !state.resolve_permission(&request_id, allow) {
        return Err(format!("permission request not found: {request_id}"));
    }
    Ok(())
}
