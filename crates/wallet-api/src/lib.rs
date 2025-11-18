use std::sync::Arc;
use tokio::sync::RwLock;
use opensyria_node_cli::Node;

pub mod api;
pub mod models;
pub mod server;

pub use server::start_server;

/// Shared application state
pub struct AppState {
    pub node: Arc<RwLock<Node>>,
}

impl AppState {
    pub fn new(node: Node) -> Self {
        Self {
            node: Arc::new(RwLock::new(node)),
        }
    }
}
