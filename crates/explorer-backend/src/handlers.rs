//! HTTP request handlers

use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use opensyria_mempool::Mempool;
use opensyria_storage::{BlockchainIndexer, BlockchainStorage, StateStorage};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub blockchain: Arc<RwLock<BlockchainStorage>>,
    pub state: Arc<RwLock<StateStorage>>,
    pub indexer: Arc<BlockchainIndexer>,
    pub mempool: Arc<RwLock<Mempool>>,
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_page() -> usize {
    1
}
fn default_per_page() -> usize {
    20
}

const MAX_PER_PAGE: usize = 100;
const MAX_ADDRESS_TX_HISTORY: usize = 100;

impl Pagination {
    fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.per_page
    }
    
    fn validate(&self) -> Result<(), ApiError> {
        if self.per_page > MAX_PER_PAGE {
            return Err(ApiError::bad_request(format!(
                "per_page cannot exceed {} (requested: {})",
                MAX_PER_PAGE, self.per_page
            )));
        }
        if self.page == 0 {
            return Err(ApiError::bad_request("page must be >= 1"));
        }
        Ok(())
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
        let body = Json(ErrorResponse::new(self.status.to_string(), self.message));
        (self.status, body).into_response()
    }
}

/// GET /api/stats - Get blockchain statistics
pub async fn get_chain_stats(State(state): State<AppState>) -> ApiResult<ChainStats> {
    // Try cache first
    if let Ok(Some(cached_stats)) = state.indexer.get_cached_stats() {
        if let Ok(stats) = serde_json::from_str::<ChainStats>(&cached_stats) {
            return Ok(Json(stats));
        }
    }

    let blockchain = state.blockchain.read().await;

    let height = blockchain
        .get_chain_height()
        .map_err(|e| ApiError::internal_error(format!("Failed to get height: {}", e)))?;

    let tip_hash = blockchain
        .get_chain_tip()
        .map_err(|e| ApiError::internal_error(format!("Failed to get tip: {}", e)))?
        .ok_or_else(|| ApiError::not_found("No blocks in chain"))?;

    let tip_block = blockchain
        .get_block(&tip_hash)
        .map_err(|e| ApiError::internal_error(format!("Failed to get block: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Tip block not found"))?;

    // Count total transactions using cached approach
    // In production, this would be maintained as a counter
    let total_transactions = (0..=height.min(1000)) // Sample first 1000 or all blocks
        .filter_map(|h| blockchain.get_block_by_height(h).ok().flatten())
        .map(|block| block.transactions.len())
        .sum::<usize>() as u64;

    let stats = ChainStats {
        height,
        total_blocks: height + 1,
        total_transactions,
        difficulty: tip_block.header.difficulty,
        latest_block_hash: hex::encode(tip_hash),
        latest_block_timestamp: tip_block.header.timestamp,
    };

    // Cache for 10 seconds
    let _ = state
        .indexer
        .cache_stats(&serde_json::to_string(&stats).unwrap_or_default());

    Ok(Json(stats))
}

/// GET /api/blocks/:height - Get block by height
pub async fn get_block_by_height(
    Path(height): Path<u64>,
    State(state): State<AppState>,
) -> ApiResult<BlockInfo> {
    let blockchain = state.blockchain.read().await;

    let block = blockchain
        .get_block_by_height(height)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found(format!("Block at height {} not found", height)))?;

    Ok(Json(BlockInfo::from_block(&block, height)))
}

/// GET /api/blocks/hash/:hash - Get block by hash
pub async fn get_block_by_hash(
    Path(hash_str): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<BlockInfo> {
    let hash_bytes =
        hex::decode(&hash_str).map_err(|_| ApiError::bad_request("Invalid hash format"))?;

    if hash_bytes.len() != 32 {
        return Err(ApiError::bad_request("Hash must be 32 bytes"));
    }

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hash_bytes);

    let blockchain = state.blockchain.read().await;
    let block = blockchain
        .get_block(&hash)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Block not found"))?;

    // Use index for O(1) height lookup
    let height = state
        .indexer
        .get_block_height(&hash)
        .map_err(|e| ApiError::internal_error(format!("Index error: {}", e)))?
        .ok_or_else(|| ApiError::internal_error("Block exists but not indexed"))?;

    Ok(Json(BlockInfo::from_block(&block, height)))
}

/// GET /api/blocks - Get recent blocks with pagination
pub async fn get_recent_blocks(
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> ApiResult<PaginatedResponse<BlockInfo>> {
    pagination.validate()?;
    
    let blockchain = state.blockchain.read().await;

    let total = blockchain
        .get_chain_height()
        .map_err(|e| ApiError::internal_error(format!("Failed to get height: {}", e)))?;

    let per_page = pagination.per_page.min(MAX_PER_PAGE);
    let start_height = total.saturating_sub(pagination.offset() as u64);
    let end_height = start_height.saturating_sub(per_page as u64);

    let mut blocks = Vec::new();
    for block_height in (end_height..start_height).rev() {
        if blocks.len() >= per_page {
            break;
        }

        if let Ok(Some(block)) = blockchain.get_block_by_height(block_height) {
            blocks.push(BlockInfo::from_block(&block, block_height));
        }
    }

    Ok(Json(PaginatedResponse::new(
        blocks,
        total as usize,
        pagination.page,
        per_page,
    )))
}

/// GET /api/transactions/:hash - Get transaction by hash
pub async fn get_transaction(
    Path(hash_str): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<TransactionInfo> {
    let hash_bytes =
        hex::decode(&hash_str).map_err(|_| ApiError::bad_request("Invalid hash format"))?;

    if hash_bytes.len() != 32 {
        return Err(ApiError::bad_request("Hash must be 32 bytes"));
    }

    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hash_bytes);

    // Use index for O(1) lookup
    let location = state
        .indexer
        .get_tx_location(&hash)
        .map_err(|e| ApiError::internal_error(format!("Index error: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Transaction not found"))?;

    let blockchain = state.blockchain.read().await;
    let block = blockchain
        .get_block_by_height(location.block_height)
        .map_err(|e| ApiError::internal_error(format!("Failed to get block: {}", e)))?
        .ok_or_else(|| ApiError::not_found("Block not found"))?;

    let tx = block
        .transactions
        .get(location.tx_index)
        .ok_or_else(|| ApiError::not_found("Transaction index out of bounds"))?;

    let info = TransactionInfo::from_transaction(tx).with_block_info(&block, location.block_height);
    Ok(Json(info))
}

/// GET /api/address/:address - Get address information
pub async fn get_address_info(
    Path(address_str): Path<String>,
    Query(pagination): Query<Pagination>,
    State(state): State<AppState>,
) -> ApiResult<AddressInfo> {
    pagination.validate()?;
    
    let address_bytes =
        hex::decode(&address_str).map_err(|_| ApiError::bad_request("Invalid address format"))?;

    if address_bytes.len() != 32 {
        return Err(ApiError::bad_request("Address must be 32 bytes"));
    }

    let mut address = [0u8; 32];
    address.copy_from_slice(&address_bytes);

    let state_db = state.state.read().await;

    let public_key = opensyria_core::crypto::PublicKey(address);

    let balance = state_db
        .get_balance(&public_key)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    let nonce = state_db
        .get_nonce(&public_key)
        .map_err(|e| ApiError::internal_error(format!("Database error: {}", e)))?;

    // Use index for O(k) lookup where k = tx count for address
    let tx_hashes = state
        .indexer
        .get_address_tx_hashes(&public_key)
        .map_err(|e| ApiError::internal_error(format!("Index error: {}", e)))?;

    let transaction_count = tx_hashes.len().min(MAX_ADDRESS_TX_HISTORY);

    Ok(Json(AddressInfo {
        address: address_str,
        balance,
        nonce,
        transaction_count,
    }))
}

/// GET /api/search/:query - Search for block/transaction/address (supports partial hash)
pub async fn search(
    Path(query): Path<String>,
    State(state): State<AppState>,
) -> ApiResult<SearchResult> {
    // Support partial hash search (minimum 8 characters)
    if query.len() >= 8 && query.len() < 64 {
        // Partial hash - search all possibilities
        let prefix_lower = query.to_lowercase();
        
        // Try blocks
        let blockchain = state.blockchain.read().await;
        let height = blockchain.get_chain_height().unwrap_or(0);
        
        for block_height in (0..=height).rev().take(100) {
            if let Ok(Some(block)) = blockchain.get_block_by_height(block_height) {
                let block_hash = hex::encode(block.hash());
                if block_hash.starts_with(&prefix_lower) {
                    return Ok(Json(SearchResult::Block {
                        info: BlockInfo::from_block(&block, block_height),
                    }));
                }
                
                // Check transactions in this block
                for (_tx_index, tx) in block.transactions.iter().enumerate() {
                    let tx_hash = hex::encode(tx.hash());
                    if tx_hash.starts_with(&prefix_lower) {
                        let info = TransactionInfo::from_transaction(tx)
                            .with_block_info(&block, block_height);
                        return Ok(Json(SearchResult::Transaction { info }));
                    }
                }
            }
        }
    }
    
    // Try to decode as full hash
    if let Ok(hash_bytes) = hex::decode(&query) {
        if hash_bytes.len() == 32 {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&hash_bytes);

            // Try as block hash using index
            if let Ok(Some(height)) = state.indexer.get_block_height(&hash) {
                let blockchain = state.blockchain.read().await;
                if let Ok(Some(block)) = blockchain.get_block_by_height(height) {
                    return Ok(Json(SearchResult::Block {
                        info: BlockInfo::from_block(&block, height),
                    }));
                }
            }

            // Try as transaction hash using index
            if let Ok(Some(location)) = state.indexer.get_tx_location(&hash) {
                let blockchain = state.blockchain.read().await;
                if let Ok(Some(block)) = blockchain.get_block_by_height(location.block_height) {
                    if let Some(tx) = block.transactions.get(location.tx_index) {
                        let info = TransactionInfo::from_transaction(tx)
                            .with_block_info(&block, location.block_height);
                        return Ok(Json(SearchResult::Transaction { info }));
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

/// GET /api/mempool - Get mempool status and pending transactions
pub async fn get_mempool(State(state): State<AppState>) -> ApiResult<MempoolInfo> {
    let mempool = state.mempool.read().await;
    
    let pending_txs = mempool.get_all_transactions();
    let transaction_count = pending_txs.len();
    
    // Convert to TransactionInfo
    let transactions: Vec<TransactionInfo> = pending_txs
        .into_iter()
        .take(50) // Limit to 50 most recent
        .map(|tx| TransactionInfo::from_transaction(&tx))
        .collect();
    
    Ok(Json(MempoolInfo {
        transaction_count,
        transactions,
    }))
}
