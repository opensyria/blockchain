use axum::http::{header, HeaderValue};
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{api, auth::Permission, AppState};

/// Start the wallet API server with optional TLS
pub async fn start_server(
    state: AppState,
    host: &str,
    port: u16,
    tls_cert: Option<PathBuf>,
    tls_key: Option<PathBuf>,
) -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
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

    // Create security headers
    let security_headers = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ));

    // Create router with CORS, security headers, and tracing
    let app = api::create_router(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(security_headers)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", host, port);

    // Start server with or without TLS
    match (tls_cert, tls_key) {
        (Some(cert_path), Some(key_path)) => {
            info!("🔒 Starting HTTPS server with TLS");
            info!("   Certificate: {}", cert_path.display());
            info!("   Private key: {}", key_path.display());

            let config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert_path, key_path)
                .await?;

            info!("🚀 Wallet API server running on https://{}", addr);
            info!("📡 Endpoints:");
            info!("   [PROTECTED] POST /api/v1/transaction/submit");
            info!("   [PROTECTED] GET  /api/v1/mempool/status");
            info!("   [PUBLIC]    GET  /api/v1/account/:address/balance");
            info!("   [PUBLIC]    GET  /api/v1/blockchain/info");
            info!("   [PUBLIC]    GET  /health");
            info!("");
            info!("🔐 Authentication: Bearer token required for protected endpoints");
            info!("🚦 Rate limiting: 100 requests per minute per IP");
            info!("🛡️  Security headers enabled: HSTS, X-Frame-Options, CSP");

            axum_server::bind_rustls(addr.parse()?, config)
                .serve(app.into_make_service())
                .await?;
        }
        _ => {
            info!("⚠️  Starting HTTP server WITHOUT TLS (not recommended for production)");

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
            info!("⚠️  PRODUCTION WARNING: Use --tls-cert and --tls-key for HTTPS");

            axum::serve(listener, app).await?;
        }
    }

    Ok(())
}
