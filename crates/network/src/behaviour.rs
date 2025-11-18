use libp2p::{
    gossipsub::{self, IdentTopic, MessageAuthenticity, ValidationMode},
    identify,
    kad::{self, store::MemoryStore},
    mdns,
    ping,
    request_response::{self, ProtocolSupport, cbor},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use serde::{Deserialize, Serialize};

/// OpenSyria network behavior combining multiple protocols
#[derive(NetworkBehaviour)]
pub struct OpenSyriaBehaviour {
    /// Gossipsub for block and transaction propagation
    pub gossipsub: gossipsub::Behaviour,

    /// mDNS for local peer discovery
    pub mdns: mdns::tokio::Behaviour,

    /// Kademlia DHT for peer discovery and routing
    pub kademlia: kad::Behaviour<MemoryStore>,

    /// Request-response for block sync
    pub request_response: cbor::Behaviour<NetworkRequest, NetworkResponse>,

    /// Identify protocol for peer info exchange
    pub identify: identify::Behaviour,

    /// Ping for connection health
    pub ping: ping::Behaviour,
}

/// Request types for request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkRequest {
    GetBlocks { start_height: u64, max_blocks: usize },
    GetChainTip,
    GetPeers,
}

/// Response types for request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkResponse {
    Blocks { blocks: Vec<Vec<u8>> }, // Serialized blocks
    ChainTip { height: u64, block_hash: [u8; 32] },
    Peers { peers: Vec<String> },
    Error { message: String },
}

/// Gossipsub topics
pub const TOPIC_BLOCKS: &str = "opensyria/blocks/1.0.0";
pub const TOPIC_TRANSACTIONS: &str = "opensyria/transactions/1.0.0";

impl OpenSyriaBehaviour {
    /// Create a new network behaviour
    pub fn new(local_key: &libp2p::identity::Keypair) -> Result<Self, String> {
        // Configure Gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .message_id_fn(|message: &gossipsub::Message| {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                message.data.hash(&mut hasher);
                gossipsub::MessageId::from(hasher.finish().to_string())
            })
            .build()
            .map_err(|e| format!("Gossipsub config error: {}", e))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        ).map_err(|e| format!("Gossipsub error: {}", e))?;

        // Subscribe to topics
        gossipsub.subscribe(&IdentTopic::new(TOPIC_BLOCKS))
            .map_err(|e| format!("Subscribe error: {}", e))?;
        gossipsub.subscribe(&IdentTopic::new(TOPIC_TRANSACTIONS))
            .map_err(|e| format!("Subscribe error: {}", e))?;

        // Configure mDNS
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            local_key.public().to_peer_id(),
        ).map_err(|e| format!("mDNS error: {}", e))?;

        // Configure Kademlia DHT
        let local_peer_id = local_key.public().to_peer_id();
        let store = MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, store);

        // Configure request-response
        let request_response = cbor::Behaviour::new(
            [(
                StreamProtocol::new("/opensyria/sync/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        // Configure identify
        let identify = identify::Behaviour::new(identify::Config::new(
            "/opensyria/1.0.0".to_string(),
            local_key.public(),
        ));

        // Configure ping
        let ping = ping::Behaviour::new(ping::Config::new());

        Ok(Self {
            gossipsub,
            mdns,
            kademlia,
            request_response,
            identify,
            ping,
        })
    }

    /// Get blocks topic
    pub fn blocks_topic() -> IdentTopic {
        IdentTopic::new(TOPIC_BLOCKS)
    }

    /// Get transactions topic
    pub fn transactions_topic() -> IdentTopic {
        IdentTopic::new(TOPIC_TRANSACTIONS)
    }
}
