use serde::{Deserialize, Serialize};

/// Request to submit a transaction
#[derive(Debug, Deserialize)]
pub struct SubmitTransactionRequest {
    /// Hex-encoded sender public key
    pub from: String,
    /// Hex-encoded recipient public key
    pub to: String,
    /// Amount in smallest units
    pub amount: u64,
    /// Transaction fee
    pub fee: u64,
    /// Hex-encoded signature
    pub signature: String,
}

/// Request to create and sign a transaction
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    /// Hex-encoded sender public key
    pub from: String,
    /// Hex-encoded recipient public key
    pub to: String,
    /// Amount in smallest units
    pub amount: u64,
    /// Transaction fee
    pub fee: u64,
    /// Hex-encoded private key (for signing)
    pub private_key: String,
}

/// Response after submitting a transaction
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub message: String,
}

/// Balance query response
#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
}

/// Blockchain info response
#[derive(Debug, Serialize)]
pub struct BlockchainInfoResponse {
    pub chain_height: u64,
    pub latest_block_hash: String,
    pub difficulty: u32,
    pub total_transactions: u64,
}

/// Transaction details
#[derive(Debug, Serialize)]
pub struct TransactionDetails {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub nonce: u64,
    pub block_height: Option<u64>,
    pub confirmed: bool,
}

/// Mempool status
#[derive(Debug, Serialize)]
pub struct MempoolStatus {
    pub pending_count: usize,
    pub total_fees: u64,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
