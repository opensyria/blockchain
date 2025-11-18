//! HTTP request handlers

use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use opensyria_storage::{BlockchainStorage, StateStorage};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<RwLock<BlockchainStorage>>,
    pub state: Arc<RwLock<StateStorage>>,
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_page() -> usize { 1 }
fn default_per_page() -> usize { 20 }

impl Pagination {
    fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

/// API Result type
type ApiResult<T> = Result<Json<T>, ApiError>;

/// API Error wrapper
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }

    fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }

    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse::new(
            self.status.to_string(),
            self.message,
        ));
        (self.status, body).into_response()
    }
}

/// GET /api/stats - Get blockchain statistics
pub async fn get_chain_stats(
    State(state): State<AppState>,
) -> ApiResult<ChainStats> {
    let blockchain = state.blockchain.read().await;
    
    let height = blockchain.get_chain_height()
        .map_err(|e| ApiError::internal_error(format!("Failed to get height: {}", e)))?;
    
    let tip_hash = blockchain.get_chain_tip()
        .map_err(|e| ApiError::internal_error(format!("Failed to get tip: {}", e)))?
        .ok_or_else(|| ApiError::not_found("No blocks in chain"))?;
    
    let tip_block = blockchain.get_block(&tip_hash)
        .map_err(|e| ApiError::internal_error(format!("Failed to get block: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Tip block not found"))?;
    
    // Count total transactions (simplified - would need index in production)
    let total_transactions = (0..=height)
        .filter_map(|h| blockchain.get_block_by_height(h).ok().flatten())
        .map(|block| block.transactions.len())
        .sum::<usize>() as u64;
    
    Ok(Json(ChainStats {
        height,
        total_blocks: height + 1,
        total_transactions,
        difficulty: tip_block.header.difficulty,
        latest_block_hash: hex::encode(tip_hash),
        latest_block_timestamp: tip_block.header.timestamp,
    }))
}

/// GET /api/blocks/:height - Get block by height
pub async fn get_block_by_height(
    Path(height): Path<u64>,
    State(state): State<AppState>,
) -> ApiResult<BlockInfo> {
    let blockchain = state.blockchain.read().await;
    
    let block = blockchain.get_block_by_height(height)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found(format!("Block at height {} not found", height)))?;
    
    Ok(Json(BlockInfo::from_block(&block, height)))
}

/// GET /api/blocks/hash/:hash - Get block by hash
pub async fn get_block_by_hash(
    Path(hash_str): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<BlockInfo> {
    let hash_bytes = hex::decode(&hash_str)
        .map_err(|_| ApiError::bad_request("Invalid hash format"))?;
    
    if hash_bytes.len() != 32 {
        return Err(ApiError::bad_request("Hash must be 32 bytes"));
    }
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hash_bytes);
    
    let blockchain = state.blockchain.read().await;
    let block = blockchain.get_block(&hash)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Block not found"))?;
    
    // Find height (inefficient - would need index in production)
    let height = find_block_height(&blockchain, &hash)
        .ok_or_else(|| ApiError::internal_error("Block exists but height not found"))?;
    
    Ok(Json(BlockInfo::from_block(&block, height)))
}

/// GET /api/blocks - Get recent blocks (paginated)
pub async fn get_recent_blocks(
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> ApiResult<PaginatedResponse<BlockInfo>> {
    let blockchain = state.blockchain.read().await;
    
    let height = blockchain.get_chain_height()
        .map_err(|e| ApiError::internal_error(format!("Failed to get height: {}", e)))?;
    
    let total = (height + 1) as usize;
    let offset = pagination.offset();
    let per_page = pagination.per_page.min(100); // Max 100 per page
    
    // Get blocks in reverse order (newest first)
    let mut blocks = Vec::new();
    for i in 0..per_page {
        let block_height = height.saturating_sub((offset + i) as u64);
        if block_height > height {
            break;
        }
        
        if let Ok(Some(block)) = blockchain.get_block_by_height(block_height) {
            blocks.push(BlockInfo::from_block(&block, block_height));
        }
    }
    
    Ok(Json(PaginatedResponse::new(blocks, total, pagination.page, per_page)))
}

/// GET /api/transactions/:hash - Get transaction by hash
pub async fn get_transaction(
    Path(hash_str): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<TransactionInfo> {
    let hash_bytes = hex::decode(&hash_str)
        .map_err(|_| ApiError::bad_request("Invalid hash format"))?;
    
    if hash_bytes.len() != 32 {
        return Err(ApiError::bad_request("Hash must be 32 bytes"));
    }
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hash_bytes);
    
    let blockchain = state.blockchain.read().await;
    
    // Search for transaction in blocks (inefficient - would need index in production)
    let height = blockchain.get_chain_height()
        .map_err(|e| ApiError::internal_error(format!("Failed to get height: {}", e)))?;
    
    for h in 0..=height {
        if let Ok(Some(block)) = blockchain.get_block_by_height(h) {
            for tx in &block.transactions {
                if tx.hash() == hash {
                    let info = TransactionInfo::from_transaction(tx)
                        .with_block_info(&block, h);
                    return Ok(Json(info));
                }
            }
        }
    }
    
    Err(ApiError::not_found("Transaction not found"))
}

/// GET /api/address/:address - Get address information
pub async fn get_address_info(
    Path(address_str): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<AddressInfo> {
    let address_bytes = hex::decode(&address_str)
        .map_err(|_| ApiError::bad_request("Invalid address format"))?;
    
    if address_bytes.len() != 32 {
        return Err(ApiError::bad_request("Address must be 32 bytes"));
    }
    
    let mut address = [0u8; 32];
    address.copy_from_slice(&address_bytes);
    
    let state_db = state.state.read().await;
    
    let public_key = opensyria_core::crypto::PublicKey(address);
    
    let balance = state_db.get_balance(&public_key)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;
    
    let nonce = state_db.get_nonce(&public_key)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;
    
    // Count transactions (simplified - would need index)
    let blockchain = state.blockchain.read().await;
    let height = blockchain.get_chain_height()
        .map_err(|e| ApiError::internal_error(format!("Failed to get height: {}", e)))?;
    
    let mut transaction_count = 0;
    for h in 0..=height {
        if let Ok(Some(block)) = blockchain.get_block_by_height(h) {
            transaction_count += block.transactions.iter()
                .filter(|tx| tx.from.0 == address || tx.to.0 == address)
                .count();
        }
    }
    
    Ok(Json(AddressInfo {
        address: address_str,
        balance,
        nonce,
        transaction_count,
    }))
}

/// GET /api/search/:query - Search for block/transaction/address
pub async fn search(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<SearchResult> {
    // Try to decode as hash
    if let Ok(hash_bytes) = hex::decode(&query) {
        if hash_bytes.len() == 32 {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);
            
            let blockchain = state.blockchain.read().await;
            
            // Try as block hash
            if let Ok(Some(block)) = blockchain.get_block(&hash) {
                if let Some(height) = find_block_height(&blockchain, &hash) {
                    return Ok(Json(SearchResult::Block {
                        info: BlockInfo::from_block(&block, height),
                    }));
                }
            }
            
            // Try as transaction hash
            let height = blockchain.get_chain_height().unwrap_or(0);
            for h in 0..=height {
                if let Ok(Some(block)) = blockchain.get_block_by_height(h) {
                    for tx in &block.transactions {
                        if tx.hash() == hash {
                            let info = TransactionInfo::from_transaction(tx)
                                .with_block_info(&block, h);
                            return Ok(Json(SearchResult::Transaction { info }));
                        }
                    }
                }
            }
            
            // Try as address
            let state_db = state.state.read().await;
            let public_key = opensyria_core::crypto::PublicKey(hash);
            if let Ok(balance) = state_db.get_balance(&public_key) {
                if let Ok(nonce) = state_db.get_nonce(&public_key) {
                    return Ok(Json(SearchResult::Address {
                        info: AddressInfo {
                            address: query,
                            balance,
                            nonce,
                            transaction_count: 0,
                        },
                    }));
                }
            }
        }
    }
    
    // Try as block height
    if let Ok(height) = query.parse::<u64>() {
        let blockchain = state.blockchain.read().await;
        if let Ok(Some(block)) = blockchain.get_block_by_height(height) {
            return Ok(Json(SearchResult::Block {
                info: BlockInfo::from_block(&block, height),
            }));
        }
    }
    
    Ok(Json(SearchResult::NotFound))
}

/// Helper: Find block height by hash (inefficient - needs index)
fn find_block_height(blockchain: &BlockchainStorage, target_hash: &[u8; 32]) -> Option<u64> {
    let height = blockchain.get_chain_height().ok()?;
    for h in 0..=height {
        if let Ok(Some(block)) = blockchain.get_block_by_height(h) {
            if block.hash() == *target_hash {
                return Some(h);
            }
        }
    }
    None
}
