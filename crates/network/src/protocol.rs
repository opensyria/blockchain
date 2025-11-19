use opensyria_core::{Block, Transaction};
use serde::{Deserialize, Serialize};

/// Maximum gossipsub message size: 2MB
/// الحد الأقصى لحجم رسالة gossipsub: 2 ميغابايت
pub const MAX_GOSSIPSUB_MESSAGE_SIZE: usize = 2 * 1024 * 1024;

/// Message size validation error
#[derive(Debug, Clone)]
pub enum ValidationError {
    MessageTooLarge { size: usize, max_size: usize },
    DeserializationFailed(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MessageTooLarge { size, max_size } => {
                write!(f, "Message size {} exceeds maximum {}", size, max_size)
            }
            ValidationError::DeserializationFailed(err) => {
                write!(f, "Deserialization failed: {}", err)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Network protocol messages for OpenSyria blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Request blocks starting from a specific height
    GetBlocks {
        start_height: u64,
        max_blocks: usize,
    },

    /// Response with requested blocks
    Blocks { blocks: Vec<Block> },

    /// Request the current chain tip height
    GetChainTip,

    /// Response with chain tip height and hash
    ChainTip { height: u64, block_hash: [u8; 32] },

    /// Broadcast a new block
    NewBlock { block: Block },

    /// Broadcast a new transaction
    NewTransaction { transaction: Transaction },

    /// Request peer list
    GetPeers,

    /// Response with peer list
    Peers { peers: Vec<String> },
}

/// Protocol configuration
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Maximum number of blocks to request at once
    pub max_blocks_per_request: usize,

    /// Maximum number of pending block requests
    pub max_pending_requests: usize,

    /// Block propagation timeout (seconds)
    pub block_timeout: u64,

    /// Transaction propagation timeout (seconds)
    pub tx_timeout: u64,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            max_blocks_per_request: 500,
            max_pending_requests: 10,
            block_timeout: 30,
            tx_timeout: 10,
        }
    }
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: String,

    /// Multiaddr
    pub address: String,

    /// Chain height
    pub height: u64,

    /// Last seen timestamp
    pub last_seen: u64,

    /// Connection status
    pub connected: bool,
}

impl NetworkMessage {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes with size validation
    /// يفكك تسلسل الرسالة من البايتات مع التحقق من الحجم
    pub fn from_bytes(data: &[u8]) -> Result<Self, ValidationError> {
        // Validate message size BEFORE deserialization
        if data.len() > MAX_GOSSIPSUB_MESSAGE_SIZE {
            return Err(ValidationError::MessageTooLarge {
                size: data.len(),
                max_size: MAX_GOSSIPSUB_MESSAGE_SIZE,
            });
        }

        // Deserialize message
        bincode::deserialize(data).map_err(|e| ValidationError::DeserializationFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;

    #[test]
    fn test_serialize_get_blocks() {
        let msg = NetworkMessage::GetBlocks {
            start_height: 100,
            max_blocks: 50,
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = NetworkMessage::from_bytes(&bytes).unwrap();

        match decoded {
            NetworkMessage::GetBlocks {
                start_height,
                max_blocks,
            } => {
                assert_eq!(start_height, 100);
                assert_eq!(max_blocks, 50);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_serialize_new_transaction() {
        let keypair = KeyPair::generate();
        let tx = Transaction::new(keypair.public_key(), keypair.public_key(), 100, 1, 0);

        let msg = NetworkMessage::NewTransaction {
            transaction: tx.clone(),
        };
        let bytes = msg.to_bytes().unwrap();
        let decoded = NetworkMessage::from_bytes(&bytes).unwrap();

        match decoded {
            NetworkMessage::NewTransaction { transaction } => {
                assert_eq!(transaction.amount, 100);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_oversized_message_rejected() {
        // Create a message larger than MAX_GOSSIPSUB_MESSAGE_SIZE
        let oversized_data = vec![0u8; MAX_GOSSIPSUB_MESSAGE_SIZE + 1];

        let result = NetworkMessage::from_bytes(&oversized_data);
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidationError::MessageTooLarge { size, max_size } => {
                assert_eq!(size, MAX_GOSSIPSUB_MESSAGE_SIZE + 1);
                assert_eq!(max_size, MAX_GOSSIPSUB_MESSAGE_SIZE);
            }
            _ => panic!("Expected MessageTooLarge error"),
        }
    }
}
