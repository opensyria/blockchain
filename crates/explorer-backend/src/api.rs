//! API route definitions

use crate::handlers::*;
use axum::{
    routing::get,
    Router,
};

/// Create API router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Chain statistics
        .route("/api/stats", get(get_chain_stats))
        
        // Blocks
        .route("/api/blocks", get(get_recent_blocks))
        .route("/api/blocks/:height", get(get_block_by_height))
        .route("/api/blocks/hash/:hash", get(get_block_by_hash))
        
        // Transactions
        .route("/api/transactions/:hash", get(get_transaction))
        
        // Address
        .route("/api/address/:address", get(get_address_info))
        
        // Search
        .route("/api/search/:query", get(search))
        
        .with_state(state)
}
