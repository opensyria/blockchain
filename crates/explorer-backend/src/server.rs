//! Block Explorer HTTP Server

use crate::api::create_router;
use crate::handlers::AppState;
use axum::Router;
use opensyria_storage::{BlockchainStorage, StateStorage};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Block Explorer Server
pub struct ExplorerServer {
    blockchain: Arc<RwLock<BlockchainStorage>>,
    state: Arc<RwLock<StateStorage>>,
    addr: SocketAddr,
    static_dir: Option<PathBuf>,
}

impl ExplorerServer {
    /// Create new explorer server
    pub fn new(data_dir: PathBuf, addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        let blockchain_dir = data_dir.join("blocks");
        let state_dir = data_dir.join("state");

        let blockchain = BlockchainStorage::open(blockchain_dir)?;
        let state = StateStorage::open(state_dir)?;

        Ok(Self {
            blockchain: Arc::new(RwLock::new(blockchain)),
            state: Arc::new(RwLock::new(state)),
            addr,
            static_dir: None,
        })
    }

    /// Set static files directory
    pub fn with_static_dir(mut self, dir: PathBuf) -> Self {
        self.static_dir = Some(dir);
        self
    }

    /// Start the server
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let app_state = AppState {
            blockchain: self.blockchain.clone(),
            state: self.state.clone(),
        };

        let mut app = create_router(app_state);

        // Serve static files if directory provided
        if let Some(static_dir) = self.static_dir {
            app = Router::new()
                .nest_service("/", ServeDir::new(static_dir))
                .merge(app);
        }

        app = app
            // Enable CORS for frontend
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            // Add tracing
            .layer(TraceLayer::new_for_http());

        tracing::info!("Starting explorer server on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
