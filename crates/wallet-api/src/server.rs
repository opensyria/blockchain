use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{api, AppState};

/// Start the wallet API server
pub async fn start_server(state: AppState, host: &str, port: u16) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

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
    info!("   POST /api/v1/transaction/submit");
    info!("   POST /api/v1/transaction/create");
    info!("   GET  /api/v1/account/:address/balance");
    info!("   GET  /api/v1/blockchain/info");
    info!("   GET  /api/v1/mempool/status");
    info!("   GET  /health");

    axum::serve(listener, app).await?;

    Ok(())
}
