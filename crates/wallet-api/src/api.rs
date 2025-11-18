use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use opensyria_core::{
    crypto::{KeyPair, PublicKey},
    transaction::Transaction,
};

use crate::{models::*, AppState};

/// Create API router
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Transaction endpoints
        .route("/api/v1/transaction/submit", post(submit_transaction))
        .route(
            "/api/v1/transaction/create",
            post(create_and_sign_transaction),
        )
        // Account endpoints
        .route("/api/v1/account/{address}/balance", get(get_balance))
        // Blockchain endpoints
        .route("/api/v1/blockchain/info", get(get_blockchain_info))
        .route("/api/v1/mempool/status", get(get_mempool_status))
        // Health check
        .route("/health", get(health_check))
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "opensyria-wallet-api"
    }))
}

/// Submit a signed transaction
async fn submit_transaction(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SubmitTransactionRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Parse sender public key
    let from = PublicKey::from_hex(&request.from).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid sender address".to_string(),
            }),
        )
    })?;

    // Parse recipient public key
    let to = PublicKey::from_hex(&request.to).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid recipient address".to_string(),
            }),
        )
    })?;

    // Parse signature
    let signature_bytes = hex::decode(&request.signature).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid signature format".to_string(),
            }),
        )
    })?;

    // Get node and current state
    let node = state.node.read().await;
    let state_storage = node.get_state();
    let balance = state_storage.get_balance(&from).unwrap_or(0);
    let nonce = state_storage.get_nonce(&from).unwrap_or(0);

    // Create transaction with signature
    let transaction = Transaction::new(from, to, request.amount, request.fee, nonce)
        .with_signature(signature_bytes);

    // Verify signature
    if transaction.verify().is_err() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid transaction signature".to_string(),
            }),
        ));
    }

    // Get transaction hash
    let tx_hash = hex::encode(transaction.hash());

    // Add to mempool
    drop(node); // Release read lock
    let mut node = state.node.write().await;

    match node.add_transaction_to_mempool(transaction) {
        Ok(_) => Ok(Json(TransactionResponse {
            success: true,
            tx_hash: Some(tx_hash),
            message: "Transaction submitted successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to submit transaction: {}", e),
            }),
        )),
    }
}

/// Create and sign a transaction
async fn create_and_sign_transaction(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<TransactionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Parse private key (must be exactly 32 bytes)
    let private_key_bytes = hex::decode(&request.private_key).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid private key format".to_string(),
            }),
        )
    })?;

    if private_key_bytes.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Private key must be 32 bytes".to_string(),
            }),
        ));
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&private_key_bytes);

    let keypair = KeyPair::from_bytes(&key_array).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid private key".to_string(),
            }),
        )
    })?;

    // Verify sender matches private key
    let from = PublicKey::from_hex(&request.from).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid sender address".to_string(),
            }),
        )
    })?;

    if keypair.public_key() != from {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Private key does not match sender address".to_string(),
            }),
        ));
    }

    // Parse recipient
    let to = PublicKey::from_hex(&request.to).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid recipient address".to_string(),
            }),
        )
    })?;

    // Get current nonce
    let node = state.node.read().await;
    let state_storage = node.get_state();
    let nonce = state_storage.get_nonce(&from).unwrap_or(0);

    // Create and sign transaction
    let mut transaction = Transaction::new(from, to, request.amount, request.fee, nonce);

    // Sign the transaction
    let signing_hash = transaction.signing_hash();
    let signature = keypair.sign(&signing_hash);
    transaction = transaction.with_signature(signature);

    let tx_hash = hex::encode(transaction.hash());

    // Add to mempool
    drop(node); // Release read lock
    let mut node = state.node.write().await;

    match node.add_transaction_to_mempool(transaction) {
        Ok(_) => Ok(Json(TransactionResponse {
            success: true,
            tx_hash: Some(tx_hash),
            message: "Transaction created and submitted successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to submit transaction: {}", e),
            }),
        )),
    }
}

/// Get account balance
async fn get_balance(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(address): axum::extract::Path<String>,
) -> Result<Json<BalanceResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Parse address
    let public_key = PublicKey::from_hex(&address).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Invalid address format".to_string(),
            }),
        )
    })?;

    // Get account info
    let node = state.node.read().await;
    let state_storage = node.get_state();
    let balance = state_storage.get_balance(&public_key).unwrap_or(0);
    let nonce = state_storage.get_nonce(&public_key).unwrap_or(0);

    Ok(Json(BalanceResponse {
        address,
        balance,
        nonce,
    }))
}

/// Get blockchain info
async fn get_blockchain_info(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BlockchainInfoResponse>, (StatusCode, Json<ErrorResponse>)> {
    let node = state.node.read().await;
    let blockchain = node.get_blockchain();

    let chain_height = blockchain.get_chain_height().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get chain height: {}", e),
            }),
        )
    })?;

    let tip_hash = blockchain
        .get_chain_tip()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get chain tip: {}", e),
                }),
            )
        })?
        .unwrap_or([0u8; 32]);

    let latest_block = blockchain.get_block_by_height(chain_height).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to get latest block: {}", e),
            }),
        )
    })?;

    let (difficulty, total_transactions) = if let Some(block) = latest_block {
        let mut tx_count = 0u64;
        for height in 1..=chain_height {
            if let Ok(Some(b)) = blockchain.get_block_by_height(height) {
                tx_count += b.transactions.len() as u64;
            }
        }
        (block.header.difficulty, tx_count)
    } else {
        (0, 0)
    };

    Ok(Json(BlockchainInfoResponse {
        chain_height,
        latest_block_hash: hex::encode(tip_hash),
        difficulty,
        total_transactions,
    }))
}

/// Get mempool status
async fn get_mempool_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<MempoolStatus>, (StatusCode, Json<ErrorResponse>)> {
    let node = state.node.read().await;

    let pending = node.get_pending_transactions();
    let total_fees: u64 = pending.iter().map(|tx| tx.fee).sum();

    Ok(Json(MempoolStatus {
        pending_count: pending.len(),
        total_fees,
    }))
}
