use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use opensyria_storage::{BlockchainStorage, StateStorage};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// Maximum concurrent WebSocket connections
const MAX_WS_CONNECTIONS: usize = 1000;

/// Global connection counter
static WS_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    /// New block mined
    NewBlock {
        height: u64,
        hash: String,
        transactions: usize,
        timestamp: u64,
    },
    /// New transaction in mempool
    NewTransaction {
        hash: String,
        from: String,
        to: String,
        amount: u64,
    },
    /// Chain statistics update
    StatsUpdate {
        height: u64,
        total_transactions: u64,
        difficulty: String,
        hash_rate: f64,
    },
    /// Mempool status
    MempoolUpdate {
        pending_count: usize,
        total_fees: u64,
    },
    /// Client ping
    Ping,
    /// Server pong
    Pong,
}

/// Shared WebSocket state
#[derive(Clone)]
pub struct WsState {
    pub blockchain: Arc<RwLock<BlockchainStorage>>,
    pub state: Arc<RwLock<StateStorage>>,
}

/// WebSocket handler with connection limiting
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<WsState>,
) -> Response {
    // Check connection limit
    let current_connections = WS_CONNECTIONS.load(Ordering::Relaxed);
    if current_connections >= MAX_WS_CONNECTIONS {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many WebSocket connections. Please try again later.",
        )
            .into_response();
    }

    // Increment connection count
    WS_CONNECTIONS.fetch_add(1, Ordering::Relaxed);

    ws.on_upgrade(|socket| async move {
        handle_socket(socket, state).await;
        // Decrement on disconnect
        WS_CONNECTIONS.fetch_sub(1, Ordering::Relaxed);
    })
}

async fn handle_socket(socket: WebSocket, state: WsState) {
    let (mut sender, mut receiver) = socket.split();

    // Send initial stats
    if let Ok(msg) = get_stats_message(&state).await {
        let _ = sender
            .send(Message::Text(serde_json::to_string(&msg).unwrap()))
            .await;
    }

    // Idle timeout: 5 minutes
    const _IDLE_TIMEOUT: Duration = Duration::from_secs(300);
    let mut idle_ticks = 0;
    const MAX_IDLE_TICKS: u32 = 30; // 30 ticks * 10 sec = 5 min

    // Spawn task to send periodic updates
    let mut update_interval = interval(Duration::from_secs(10));
    let sender_state = state.clone();
    
    let mut send_task = tokio::spawn(async move {
        loop {
            update_interval.tick().await;
            
            // Send stats update
            if let Ok(msg) = get_stats_message(&sender_state).await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if sender.send(Message::Text(json)).await.is_err() {
                        break; // Connection closed
                    }
                } else {
                    idle_ticks += 1;
                }
            } else {
                idle_ticks += 1;
            }

            // Close connection if idle too long
            if idle_ticks > MAX_IDLE_TICKS {
                break;
            }
        }
    });

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg {
                        WsMessage::Ping => {
                            // Client wants to keep connection alive
                            tracing::debug!("WebSocket ping received");
                        }
                        _ => {
                            tracing::warn!("Unexpected message from client: {:?}", ws_msg);
                        }
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    tracing::info!("WebSocket connection closed");
}

async fn get_stats_message(state: &WsState) -> Result<WsMessage, String> {
    let blockchain = state.blockchain.read().await;

    let height = blockchain.get_chain_height().map_err(|e| e.to_string())?;
    // Total transactions approximation (would come from proper indexer)
    let total_transactions = height * 2; // Placeholder: average 2 tx per block

    // Use fixed difficulty for now (would come from latest block)
    let difficulty = "16".to_string();

    Ok(WsMessage::StatsUpdate {
        height,
        total_transactions,
        difficulty,
        hash_rate: 1_600_000.0, // Placeholder
    })
}
