use opensyria_node_cli::Node;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod api;
pub mod auth;
pub mod models;
pub mod rate_limit;
pub mod server;

pub use server::start_server;

/// Shared application state
pub struct AppState {
    pub node: Arc<RwLock<Node>>,
    pub api_key_manager: Arc<auth::ApiKeyManager>,
    pub rate_limiter: Arc<rate_limit::RateLimiter>,
}

impl AppState {
    pub fn new(node: Node) -> Self {
        Self {
            node: Arc::new(RwLock::new(node)),
            api_key_manager: Arc::new(auth::ApiKeyManager::new()),
            rate_limiter: Arc::new(rate_limit::RateLimiter::new()),
        }
    }
}
