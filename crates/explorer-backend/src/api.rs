//! API route definitions

use crate::handlers::*;
use crate::websocket::{ws_handler, WsState};
use axum::{routing::get, Router};

/// Create API router with all routes
pub fn create_router(state: AppState) -> Router {
    // Create WebSocket state from AppState
    let ws_state = WsState {
        blockchain: state.blockchain.clone(),
        state: state.state.clone(),
    };

    Router::new()
        // WebSocket for real-time updates (separate state)
        .route("/ws", get(ws_handler).with_state(ws_state))
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
