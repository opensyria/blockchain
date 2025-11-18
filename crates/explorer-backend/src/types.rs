//! API response types

use opensyria_core::{Block, Transaction};
use serde::{Deserialize, Serialize};

/// Block information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub hash: String,
    pub height: u64,
    pub timestamp: u64,
    pub difficulty: u32,
    pub nonce: u64,
    pub previous_hash: String,
    pub merkle_root: String,
    pub transaction_count: usize,
    pub transactions: Vec<String>, // Transaction hashes
}

impl BlockInfo {
    pub fn from_block(block: &Block, height: u64) -> Self {
        Self {
            hash: hex::encode(block.hash()),
            height,
            timestamp: block.header.timestamp,
            difficulty: block.header.difficulty,
            nonce: block.header.nonce,
            previous_hash: hex::encode(block.header.previous_hash),
            merkle_root: hex::encode(block.header.merkle_root),
            transaction_count: block.transactions.len(),
            transactions: block
                .transactions
                .iter()
                .map(|tx| hex::encode(tx.hash()))
                .collect(),
        }
    }
}

/// Transaction information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub signature: String,
    pub timestamp: Option<u64>, // Block timestamp if confirmed
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
}

impl TransactionInfo {
    pub fn from_transaction(tx: &Transaction) -> Self {
        Self {
            hash: hex::encode(tx.hash()),
            from: hex::encode(tx.from.0),
            to: hex::encode(tx.to.0),
            amount: tx.amount,
            fee: tx.fee,
            nonce: tx.nonce,
            signature: hex::encode(&tx.signature),
            timestamp: None,
            block_hash: None,
            block_height: None,
        }
    }

    pub fn with_block_info(mut self, block: &Block, height: u64) -> Self {
        self.timestamp = Some(block.header.timestamp);
        self.block_hash = Some(hex::encode(block.hash()));
        self.block_height = Some(height);
        self
    }
}

/// Chain statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub height: u64,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub difficulty: u32,
    pub latest_block_hash: String,
    pub latest_block_timestamp: u64,
}

/// Address balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub transaction_count: usize,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SearchResult {
    Block { info: BlockInfo },
    Transaction { info: TransactionInfo },
    Address { info: AddressInfo },
    NotFound,
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: usize, page: usize, per_page: usize) -> Self {
        let total_pages = (total + per_page - 1) / per_page;
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}
