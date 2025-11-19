use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{api, auth::Permission, AppState};

/// Start the wallet API server
pub async fn start_server(state: AppState, host: &str, port: u16) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // Create default admin API key if none exists
    let key_count = state.api_key_manager.list_keys().await.len();
    if key_count == 0 {
        let (_key_id, raw_key) = state
            .api_key_manager
            .generate_key(
                "default-admin".to_string(),
                vec![Permission::Admin],
                None,
            )
            .await;
        info!("🔑 Generated default admin API key:");
        info!("   {}", raw_key);
        info!("   ⚠️  SAVE THIS KEY - it will not be shown again!");
        info!("   Use as: Authorization: Bearer {}", raw_key);
    }

    let state = Arc::new(state);

    // Create router with CORS and tracing
    let app = api::create_router(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    // Bind server
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("🚀 Wallet API server running on http://{}", addr);
    info!("📡 Endpoints:");
    info!("   [PROTECTED] POST /api/v1/transaction/submit");
    info!("   [PROTECTED] GET  /api/v1/mempool/status");
    info!("   [PUBLIC]    GET  /api/v1/account/:address/balance");
    info!("   [PUBLIC]    GET  /api/v1/blockchain/info");
    info!("   [PUBLIC]    GET  /health");
    info!("");
    info!("🔐 Authentication: Bearer token required for protected endpoints");
    info!("🚦 Rate limiting: 100 requests per minute per IP");

    axum::serve(listener, app).await?;

    Ok(())
}
