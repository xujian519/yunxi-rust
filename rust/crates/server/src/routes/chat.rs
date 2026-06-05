//! WebSocket 聊天端点 — 完整 Agent 流式对话

use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use runtime::Session;
use tokio::sync::mpsc;

use crate::agent_bridge::{final_assistant_text, run_agent_turn, workspace_root};
use crate::permission::{resolve_permission, ServerPermissionPrompter};
use crate::ws_stream::{stream_event_json, tool_results_from_summary, StreamEvent};
use crate::AppState;

pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let _ = socket.send(Message::Text(json_connect_msg().into())).await;

    while let Some(result) = socket.recv().await {
        match result {
            Ok(Message::Text(text)) => {
                if let Some(events) = handle_text_message(&text, &state).await {
                    for json in events {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            return;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }
}

async fn handle_text_message(text: &str, state: &AppState) -> Option<Vec<String>> {
    let msg: serde_json::Value = serde_json::from_str(text).ok()?;
    let msg_type = msg.get("type")?.as_str()?;

    match msg_type {
        "ping" => Some(vec![serde_json::json!({"type":"pong"}).to_string()]),
        "user_message" => {
            let content = msg.get("content")?.as_str()?.to_string();
            let session_id = msg
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();
            let model = msg
                .get("model")
                .and_then(|v| v.as_str())
                .map(str::to_string);

            let mut session = {
                let sessions = state.chat_sessions.lock().ok()?;
                if let Some(cached) = sessions.get(&session_id) {
                    cached.clone()
                } else if state.session_store.exists(&session_id) {
                    state
                        .session_store
                        .load(&session_id)
                        .unwrap_or_else(|_| Session::new())
                } else {
                    Session::new()
                }
            };

            let (tx, mut rx) = mpsc::unbounded_channel::<String>();
            let root = workspace_root();
            let content_for_turn = content.clone();

            let waiters = std::sync::Arc::clone(&state.permission_waiters);
            let join = tokio::task::spawn_blocking(move || {
                let tx_stream = tx.clone();
                let mut prompter = ServerPermissionPrompter::new(waiters, move |event| {
                    let _ = tx_stream.send(stream_event_json(&event));
                });
                let result = run_agent_turn(
                    &mut session,
                    &content_for_turn,
                    model.as_deref(),
                    root,
                    {
                        let tx_events = tx.clone();
                        move |event| {
                            let _ = tx_events.send(stream_event_json(&StreamEvent::from(event)));
                        }
                    },
                    Some(&mut prompter),
                );

                match result {
                    Ok(summary) => {
                        for event in tool_results_from_summary(&summary) {
                            let _ = tx.send(stream_event_json(&event));
                        }
                        let _ = tx.send(stream_event_json(&StreamEvent::MessageStop));
                        Ok((session, summary))
                    }
                    Err(error) => {
                        let _ = tx.send(stream_event_json(&StreamEvent::Error {
                            message: error.clone(),
                        }));
                        Err(error)
                    }
                }
            });

            let mut out = Vec::new();
            while let Some(json) = rx.recv().await {
                out.push(json);
            }

            if let Ok(Ok((updated_session, summary))) = join.await {
                let _ = state.session_store.save(&session_id, &updated_session);
                if let Ok(mut sessions) = state.chat_sessions.lock() {
                    sessions.insert(session_id, updated_session);
                }
                let assistant = final_assistant_text(&summary);
                if !assistant.is_empty() {
                    out.push(stream_event_json(&StreamEvent::AssistantMessage {
                        content: assistant,
                    }));
                }
            }

            Some(out)
        }
        "permission_respond" => {
            let request_id = msg.get("request_id")?.as_str()?;
            let outcome = msg.get("outcome")?.as_str()?;
            let allow = matches!(outcome, "allow" | "always");
            if !resolve_permission(&state.permission_waiters, request_id, allow) {
                return Some(vec![serde_json::json!({
                    "type": "error",
                    "message": format!("permission request not found: {request_id}")
                })
                .to_string()]);
            }
            None
        }
        _ => Some(vec![serde_json::json!({
            "type": "error",
            "message": format!("未知的消息类型: {msg_type}")
        })
        .to_string()]),
    }
}

fn json_connect_msg() -> String {
    serde_json::json!({
        "type": "connected",
        "message": "云熙智能体 WebSocket 连接已建立（Agent 模式）"
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_returns_pong() {
        let state = AppState::default_for_test();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let events = rt.block_on(async {
            handle_text_message(r#"{"type":"ping"}"#, &state)
                .await
                .unwrap()
        });
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("pong"));
    }
}
