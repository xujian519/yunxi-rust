//! 服务器启动入口

use crate::auth::AuthConfig;
use crate::routes;
use knowledge::{KnowledgePaths, UnifiedSearch};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};

/// 应用共享状态
#[derive(Clone)]
pub struct AppState {
    pub search_engine: Arc<Mutex<UnifiedSearch>>,
    pub auth_config: AuthConfig,
}

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub auth: AuthConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 8765,
            auth: AuthConfig::default(),
        }
    }
}

/// 启动 HTTP + WebSocket 服务器
pub async fn start(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    let paths = KnowledgePaths::discover();
    let search_engine = UnifiedSearch::new(
        paths.patent_kg_db.as_deref(),
        paths.laws_db.as_deref(),
        paths.card_index.as_deref(),
    );

    let state = AppState {
        search_engine: Arc::new(Mutex::new(search_engine)),
        auth_config: config.auth,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = routes::build_routes(state).layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("云熙智能体服务器启动于 http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
