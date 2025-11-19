//! Block Explorer HTTP Server

use crate::api::create_router;
use crate::handlers::AppState;
use crate::rate_limit::ExplorerRateLimiter;
use axum::{middleware, routing::Router};
use opensyria_storage::{BlockchainIndexer, BlockchainStorage, StateStorage};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Block Explorer Server
pub struct ExplorerServer {
    blockchain: Arc<RwLock<BlockchainStorage>>,
    state: Arc<RwLock<StateStorage>>,
    indexer: Arc<BlockchainIndexer>,
    addr: SocketAddr,
    static_dir: Option<PathBuf>,
    allowed_origins: Vec<String>,
}

impl ExplorerServer {
    /// Create new explorer server
    pub fn new(data_dir: PathBuf, addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let blockchain_dir = data_dir.join("blocks");
        let state_dir = data_dir.join("state");
        let index_dir = data_dir.join("index");

        let blockchain = BlockchainStorage::open(blockchain_dir)?;
        let state = StateStorage::open(state_dir)?;
        let indexer = BlockchainIndexer::open(index_dir)?;

        // Build indexes if needed
        tracing::info!("Checking blockchain indexes...");
        let height = blockchain.get_chain_height().unwrap_or(0);
        let has_genesis_indexed = indexer.get_block_height(&[0u8; 32]).ok().flatten().is_some();
        
        if !has_genesis_indexed && height > 0 {
            tracing::info!("Building indexes for {} blocks...", height + 1);
            indexer.rebuild_indexes(
                |h| blockchain.get_block_by_height(h),
                height,
            )?;
        }

        Ok(Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            state: Arc::new(RwLock::new(state)),
            indexer: Arc::new(indexer),
            addr,
            static_dir: None,
            allowed_origins: vec!["http://localhost:3000".to_string()],
        })
    }

    /// Set static files directory
    pub fn with_static_dir(mut self, dir: PathBuf) -> Self {
        self.static_dir = Some(dir);
        self
    }

    /// Set allowed CORS origins
    pub fn with_allowed_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Start the server
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let rate_limiter = Arc::new(ExplorerRateLimiter::new());
        
        let app_state = AppState {
            blockchain: self.blockchain.clone(),
            state: self.state.clone(),
            indexer: self.indexer.clone(),
        };

        let api_router = create_router(app_state);

        let mut app = Router::new().merge(api_router);

        // Serve static files if directory provided (SPA mode)
        if let Some(static_dir) = self.static_dir {
            let serve_dir = ServeDir::new(static_dir.join("dist"));
            
            app = Router::new()
                .nest_service("/", serve_dir)
                .merge(app);
        }

        // Parse allowed origins
        let allow_origin = if self.allowed_origins.is_empty() {
            AllowOrigin::any()
        } else {
            let origins: Vec<_> = self.allowed_origins
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            AllowOrigin::list(origins)
        };

        app = app
            // Rate limiting (first layer - check before processing)
            .layer(middleware::from_fn(move |req, next| {
                let limiter = rate_limiter.clone();
                crate::rate_limit::rate_limit_middleware(limiter, req, next)
            }))
            // Enable CORS with specific origins
            .layer(
                CorsLayer::new()
                    .allow_origin(allow_origin)
                    .allow_methods([
                        axum::http::Method::GET,
                        axum::http::Method::OPTIONS,
                    ])
                    .allow_headers([axum::http::header::CONTENT_TYPE]),
            )
            // Add tracing
            .layer(TraceLayer::new_for_http());

        tracing::info!("🚀 Starting explorer server on {}", self.addr);
        tracing::info!("📊 Rate limit: 60 requests per minute per IP");
        tracing::info!("🔐 CORS origins: {:?}", self.allowed_origins);
        tracing::info!("⚡ Indexes ready for fast lookups");

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
