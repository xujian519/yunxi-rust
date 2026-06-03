//! Server crate 单元测试

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use crate::{auth::AuthConfig, routes::build_routes, session_store::SessionStore, AppState, ServerConfig};
use knowledge::UnifiedSearch;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn test_state() -> AppState {
    // 禁用语义嵌入，避免在测试中创建 reqwest::blocking::Client（其内部 tokio 运行时
    // 在测试异步上下文中被丢弃会导致 panic）
    std::env::set_var("YUNXI_SEMANTIC_ENABLED", "0");
    std::env::set_var("YUNXI_SEMANTIC_AUTODETECT_DISABLED", "1");
    let search_engine = UnifiedSearch::new(None, None, None);
    AppState {
        search_engine: Arc::new(Mutex::new(search_engine)),
        auth_config: AuthConfig::default(),
        chat_sessions: Arc::new(Mutex::new(HashMap::new())),
        session_store: Arc::new(SessionStore::new()),
        permission_waiters: Arc::new(Mutex::new(HashMap::new())),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn health_check_returns_ok() {
    let state = test_state();
    let app = build_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn knowledge_search_returns_json() {
    let state = test_state();
    let app = build_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/knowledge/search?q=专利")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("total").is_some());
    assert!(json.get("results").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn router_detect_returns_json() {
    let state = test_state();
    let app = build_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/router/detect?input=帮我分析专利新颖性")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("domain").is_some());
    assert!(json.get("confidence").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn tools_list_returns_tools() {
    let state = test_state();
    let app = build_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/tools")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("count").is_some());
    assert!(json.get("tools").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn mcp_status_returns_json() {
    let state = test_state();
    let app = build_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/mcp/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("servers").is_some());
    assert!(json.get("total_tools").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn memory_search_returns_json() {
    let state = test_state();
    let app = build_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/memory/search?q=test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("count").is_some());
    assert!(json.get("entries").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn tool_execute_returns_result() {
    let state = test_state();
    let app = build_routes(state);

    let body = serde_json::json!({
        "name": "KnowledgeSearch",
        "input": { "query": "专利", "limit": 1 }
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/tools/execute")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("result").is_some());
}

#[tokio::test(flavor = "multi_thread")]
async fn sessions_create_and_load() {
    let state = test_state();
    let app = build_routes(state);

    let create_body = serde_json::json!({ "title": "测试会话" });
    let create_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/sessions")
                .header("content-type", "application/json")
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::OK);
    let created: serde_json::Value =
        serde_json::from_slice(&axum::body::to_bytes(create_res.into_body(), usize::MAX).await.unwrap()).unwrap();
    let id = created.get("id").and_then(|v| v.as_str()).unwrap();

    let load_res = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/sessions/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(load_res.status(), StatusCode::OK);
}

#[tokio::test(flavor = "multi_thread")]
async fn settings_get_put_roundtrip() {
    let state = test_state();
    let app = build_routes(state);

    let get_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    let put_body = serde_json::json!({
        "model": "deepseek-v4-pro",
        "hooks": { "PreToolUse": ["echo test"] }
    });
    let put_res = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/settings")
                .header("content-type", "application/json")
                .body(Body::from(put_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(put_res.status(), StatusCode::NO_CONTENT);
}

#[tokio::test(flavor = "multi_thread")]
async fn cases_list_returns_json() {
    let state = test_state();
    let app = build_routes(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/cases")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array());
}

#[test]
fn server_config_default() {
    let cfg = ServerConfig::default();
    assert_eq!(cfg.host, "127.0.0.1");
    assert_eq!(cfg.port, 8765);
    assert!(!cfg.auth.enabled);
}

#[test]
fn auth_config_default_allows_all() {
    let cfg = AuthConfig::default();
    assert!(!cfg.enabled);
    assert!(cfg.api_keys.is_empty());
    // 未启用鉴权时，verify_auth 应返回 Ok
    let headers = axum::http::HeaderMap::new();
    assert!(crate::auth::verify_auth(&headers, &cfg).is_ok());
}
