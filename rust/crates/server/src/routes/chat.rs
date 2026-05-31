//! WebSocket 聊天端点

use crate::AppState;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};
use router::workflow_router::WorkflowRouter;
use router::RoutingDecision;

pub async fn ws_handler(State(state): State<AppState>, ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, _state: AppState) {
    let _ = socket.send(Message::Text(json_connect_msg().into())).await;

    loop {
        let msg = socket.recv().await;
        match msg {
            Some(Ok(Message::Text(text))) => {
                let response = handle_text_message(&text);
                let _ = socket.send(Message::Text(response.into())).await;
            }
            Some(Ok(Message::Close(_))) => break,
            Some(Err(_)) => break,
            None => break,
            _ => {}
        }
    }
}

fn handle_text_message(text: &str) -> String {
    let msg: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => {
            return serde_json::json!({
                "type": "echo",
                "content": format!("收到: {text}")
            })
            .to_string();
        }
    };

    let msg_type = msg
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    match msg_type {
        "ping" => serde_json::json!({"type": "pong"}).to_string(),
        "user_message" => {
            let content = msg.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let decision = WorkflowRouter::default().route(content);
            let routing_prefix = build_routing_prefix(&decision);
            let routed_input = format!("{routing_prefix}{content}");

            serde_json::json!({
                "type": "assistant_message",
                "content": format!("已收到您的消息，正在处理: {content}"),
                "routing": {
                    "domain": format!("{}", decision.domain),
                    "complexity": format!("{}", decision.complexity),
                    "intent": decision.intent_name,
                    "confidence": decision.confidence,
                    "suggested_tools": decision.suggested_tools,
                },
                "routed_input": routed_input,
            })
            .to_string()
        }
        _ => serde_json::json!({
            "type": "error",
            "message": format!("未知的消息类型: {msg_type}")
        })
        .to_string(),
    }
}

fn build_routing_prefix(decision: &RoutingDecision) -> String {
    let tools = decision.suggested_tools.join(", ");
    let agents = if decision.suggested_agents.is_empty() {
        "无".to_string()
    } else {
        decision.suggested_agents.join("、")
    };
    let intent_line = if decision.intent_confidence > 0.3 {
        format!(
            "\n意图: {} ({:.0}%)",
            decision.intent_name,
            decision.intent_confidence * 100.0
        )
    } else {
        String::new()
    };
    format!(
        "<yunxi_routing>\n{}{}\n建议工具: {}\n建议子智能体: {}\n</yunxi_routing>\n\n",
        decision.reasoning, intent_line, tools, agents,
    )
}

fn json_connect_msg() -> String {
    serde_json::json!({
        "type": "connected",
        "message": "云熙智能体 WebSocket 连接已建立"
    })
    .to_string()
}
