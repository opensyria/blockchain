use crate::{
    behaviour::{NetworkRequest, NetworkResponse, OpenSyriaBehaviour},
    protocol::NetworkMessage,
};
use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    core::upgrade,
    gossipsub::{self},
    identity, noise, tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use opensyria_core::{Block, Transaction};
use opensyria_mempool::{Mempool, MempoolConfig};
use opensyria_storage::{BlockchainStorage, StateStorage};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// P2P Network Node
pub struct NetworkNode {
    /// libp2p swarm
    swarm: Swarm<OpenSyriaBehaviour>,

    /// Local peer ID
    local_peer_id: PeerId,

    /// Blockchain storage
    blockchain: Arc<RwLock<BlockchainStorage>>,

    /// State storage
    #[allow(dead_code)]
    state: Arc<RwLock<StateStorage>>,

    /// Transaction mempool
    mempool: Arc<RwLock<Mempool>>,

    /// Connected peers
    peers: Arc<RwLock<HashSet<PeerId>>>,

    /// Pending block requests
    pending_blocks: Arc<RwLock<HashMap<PeerId, u64>>>,

    /// Event sender
    event_tx: mpsc::UnboundedSender<NetworkEvent>,
}

/// Network events
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerId),

    /// Peer disconnected
    PeerDisconnected(PeerId),

    /// Received a new block
    NewBlock(Block),

    /// Received a new transaction
    NewTransaction(Transaction),

    /// Chain tip updated
    ChainTipUpdated { height: u64, hash: [u8; 32] },

    /// Sync progress
    SyncProgress { current: u64, target: u64 },
}

/// Network node configuration
#[derive(Debug, Clone)]
pub struct NodeConfig {
    /// Listen address
    pub listen_addr: Multiaddr,

    /// Bootstrap peers
    pub bootstrap_peers: Vec<Multiaddr>,

    /// Data directory
    pub data_dir: PathBuf,

    /// Enable mDNS discovery
    pub enable_mdns: bool,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/9000".parse().unwrap(),
            bootstrap_peers: Vec::new(),
            data_dir: PathBuf::from("~/.opensyria/network"),
            enable_mdns: true,
        }
    }
}

impl NetworkNode {
    /// Create a new network node
    pub async fn new(config: NodeConfig) -> Result<(Self, mpsc::UnboundedReceiver<NetworkEvent>)> {
        // Generate or load keypair
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = local_key.public().to_peer_id();

        info!("Local peer ID: {}", local_peer_id);

        // Create transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(upgrade::Version::V1Lazy)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create behaviour
        let behaviour = OpenSyriaBehaviour::new(&local_key).map_err(|e| anyhow::anyhow!(e))?;

        // Create swarm
        let mut swarm_config = libp2p::swarm::Config::with_executor(Box::new(|fut| {
            tokio::spawn(fut);
        }));
        swarm_config = swarm_config.with_idle_connection_timeout(Duration::from_secs(60));

        let swarm = Swarm::new(transport, behaviour, local_peer_id, swarm_config);

        // Open storage
        let blockchain = Arc::new(RwLock::new(BlockchainStorage::open(
            config.data_dir.join("blockchain"),
        )?));
        let state = Arc::new(RwLock::new(StateStorage::open(
            config.data_dir.join("state"),
        )?));

        // Create mempool
        let mempool_config = MempoolConfig::default();
        let mempool = Arc::new(RwLock::new(Mempool::new(mempool_config, state.clone())));

        // Create event channel
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let node = Self {
            swarm,
            local_peer_id,
            blockchain,
            state,
            mempool,
            peers: Arc::new(RwLock::new(HashSet::new())),
            pending_blocks: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        };

        Ok((node, event_rx))
    }

    /// Start listening for connections
    pub async fn listen(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm.listen_on(addr)?;
        info!("Listening on all configured addresses");
        Ok(())
    }

    /// Dial a peer
    pub async fn dial(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm.dial(addr)?;
        info!("Dialing peer");
        Ok(())
    }

    /// Broadcast a new block
    pub async fn broadcast_block(&mut self, block: &Block) -> Result<()> {
        let msg = NetworkMessage::NewBlock {
            block: block.clone(),
        };
        let data = msg.to_bytes()?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(OpenSyriaBehaviour::blocks_topic(), data)?;

        debug!("Broadcast new block");
        Ok(())
    }

    /// Broadcast a new transaction
    pub async fn broadcast_transaction(&mut self, tx: &Transaction) -> Result<()> {
        let msg = NetworkMessage::NewTransaction {
            transaction: tx.clone(),
        };
        let data = msg.to_bytes()?;

        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(OpenSyriaBehaviour::transactions_topic(), data)?;

        debug!("Broadcast transaction");
        Ok(())
    }

    /// Get local peer ID
    pub fn local_peer_id(&self) -> PeerId {
        self.local_peer_id
    }

    /// Get connected peers count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Get local chain height
    pub async fn get_chain_height(&self) -> Result<u64> {
        let blockchain = self.blockchain.read().await;
        blockchain
            .get_chain_height()
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Request blocks from a peer
    pub async fn request_blocks(&mut self, peer_id: PeerId, start_height: u64, max_blocks: usize) {
        let request = NetworkRequest::GetBlocks {
            start_height,
            max_blocks,
        };

        let _request_id = self
            .swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer_id, request);

        debug!(
            "Requested blocks from {} starting at height {}",
            peer_id, start_height
        );

        // Track pending request
        self.pending_blocks
            .write()
            .await
            .insert(peer_id, start_height);
    }

    /// Sync with network
    pub async fn sync(&mut self) -> Result<()> {
        info!("Starting blockchain sync");

        let local_height = self.get_chain_height().await?;
        info!("Local chain height: {}", local_height);

        // Request chain tip from all peers
        let peers: Vec<PeerId> = self.peers.read().await.iter().cloned().collect();

        for peer in peers {
            let request = NetworkRequest::GetChainTip;
            self.swarm
                .behaviour_mut()
                .request_response
                .send_request(&peer, request);
        }

        Ok(())
    }

    /// Add transaction to mempool and broadcast to network
    pub async fn submit_transaction(&mut self, tx: Transaction) -> Result<()> {
        // Add to mempool
        let mut mempool = self.mempool.write().await;
        mempool
            .add_transaction(tx.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add transaction to mempool: {}", e))?;
        drop(mempool);

        // Broadcast to network
        self.broadcast_transaction(&tx).await?;

        info!(
            "Transaction submitted and broadcast: {} SYL",
            tx.amount as f64 / 1_000_000.0
        );
        Ok(())
    }

    /// Get pending transactions from mempool
    pub async fn get_pending_transactions(&self, max_count: usize) -> Vec<Transaction> {
        let mempool = self.mempool.read().await;
        mempool.get_priority_transactions(max_count)
    }

    /// Get mempool size
    pub async fn mempool_size(&self) -> usize {
        let mempool = self.mempool.read().await;
        mempool.size()
    }

    /// Clear confirmed transactions from mempool
    pub async fn clear_confirmed_transactions(&self, transactions: &[Transaction]) {
        let mut mempool = self.mempool.write().await;
        mempool.remove_confirmed_transactions(transactions);
    }

    /// Run the network node event loop
    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_swarm_event(event).await?;
                }
            }
        }
    }

    /// Handle swarm events
    async fn handle_swarm_event(
        &mut self,
        event: libp2p::swarm::SwarmEvent<OpenSyriaBehaviourEvent>,
    ) -> Result<()> {
        use libp2p::swarm::SwarmEvent;

        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on {}", address);
            }

            SwarmEvent::Behaviour(event) => {
                self.handle_behaviour_event(event).await?;
            }

            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connected to peer: {}", peer_id);
                self.peers.write().await.insert(peer_id);
                let _ = self.event_tx.send(NetworkEvent::PeerConnected(peer_id));
            }

            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("Disconnected from peer: {}", peer_id);
                self.peers.write().await.remove(&peer_id);
                let _ = self.event_tx.send(NetworkEvent::PeerDisconnected(peer_id));
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle behaviour events
    async fn handle_behaviour_event(&mut self, event: OpenSyriaBehaviourEvent) -> Result<()> {
        use crate::behaviour::OpenSyriaBehaviourEvent;

        match event {
            OpenSyriaBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. }) => {
                self.handle_gossipsub_message(message).await?;
            }

            OpenSyriaBehaviourEvent::Mdns(libp2p::mdns::Event::Discovered(peers)) => {
                for (peer_id, addr) in peers {
                    info!("Discovered peer via mDNS: {} at {}", peer_id, addr);
                    if let Err(e) = self.swarm.dial(addr.clone()) {
                        warn!("Failed to dial mDNS peer: {}", e);
                    }
                }
            }

            OpenSyriaBehaviourEvent::RequestResponse(
                libp2p::request_response::Event::Message { message, peer },
            ) => {
                self.handle_request_response(peer, message).await?;
            }

            OpenSyriaBehaviourEvent::Identify(libp2p::identify::Event::Received {
                peer_id,
                info,
            }) => {
                debug!("Identified peer {}: {:?}", peer_id, info.protocol_version);
            }

            _ => {}
        }

        Ok(())
    }

    /// Handle gossipsub messages
    async fn handle_gossipsub_message(&mut self, message: gossipsub::Message) -> Result<()> {
        let network_msg = NetworkMessage::from_bytes(&message.data)?;

        match network_msg {
            NetworkMessage::NewBlock { block } => {
                debug!("Received new block from gossipsub");

                // Validate and store block
                let blockchain = self.blockchain.read().await;
                let _current_height = blockchain.get_chain_height()?;
                drop(blockchain);

                // Try to append block
                let blockchain = self.blockchain.write().await;
                match blockchain.append_block(&block) {
                    Ok(()) => {
                        let new_height = blockchain.get_chain_height()?;
                        info!("Added new block at height {}", new_height);
                        let _ = self.event_tx.send(NetworkEvent::NewBlock(block));
                    }
                    Err(e) => {
                        debug!("Failed to append block: {:?}", e);
                    }
                }
            }

            NetworkMessage::NewTransaction { transaction } => {
                debug!("Received transaction from gossipsub");

                // Add to mempool
                let mut mempool = self.mempool.write().await;
                match mempool.add_transaction(transaction.clone()).await {
                    Ok(_) => {
                        info!("Added transaction to mempool from network");
                        let _ = self
                            .event_tx
                            .send(NetworkEvent::NewTransaction(transaction));
                    }
                    Err(e) => {
                        warn!("Failed to add transaction to mempool: {}", e);
                    }
                }
            }

            _ => {
                warn!("Unexpected message type in gossipsub");
            }
        }

        Ok(())
    }

    /// Handle request-response messages
    async fn handle_request_response(
        &mut self,
        peer: PeerId,
        message: libp2p::request_response::Message<NetworkRequest, NetworkResponse>,
    ) -> Result<()> {
        use libp2p::request_response::Message;

        match message {
            Message::Request {
                request, channel, ..
            } => {
                let response = self.handle_request(request).await;
                let _ = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, response);
            }

            Message::Response { response, .. } => {
                self.handle_response(peer, response).await?;
            }
        }

        Ok(())
    }

    /// Handle incoming requests
    async fn handle_request(&self, request: NetworkRequest) -> NetworkResponse {
        match request {
            NetworkRequest::GetBlocks {
                start_height,
                max_blocks,
            } => {
                let blockchain = self.blockchain.read().await;
                let mut blocks = Vec::new();

                for height in start_height..start_height + max_blocks as u64 {
                    if let Ok(Some(block)) = blockchain.get_block_by_height(height) {
                        if let Ok(serialized) = bincode::serialize(&block) {
                            blocks.push(serialized);
                        }
                    } else {
                        break;
                    }
                }

                NetworkResponse::Blocks { blocks }
            }

            NetworkRequest::GetChainTip => {
                let blockchain = self.blockchain.read().await;
                match blockchain.get_chain_height() {
                    Ok(height) => {
                        if let Ok(Some(block)) = blockchain.get_block_by_height(height) {
                            NetworkResponse::ChainTip {
                                height,
                                block_hash: block.hash(),
                            }
                        } else {
                            NetworkResponse::Error {
                                message: "Failed to get tip block".to_string(),
                            }
                        }
                    }
                    Err(e) => NetworkResponse::Error {
                        message: format!("Failed to get chain height: {}", e),
                    },
                }
            }

            NetworkRequest::GetPeers => {
                let peers: Vec<String> = self
                    .peers
                    .try_read()
                    .map(|p| p.iter().map(|id| id.to_string()).collect())
                    .unwrap_or_default();

                NetworkResponse::Peers { peers }
            }
        }
    }

    /// Handle incoming responses
    async fn handle_response(&mut self, peer: PeerId, response: NetworkResponse) -> Result<()> {
        match response {
            NetworkResponse::Blocks { blocks } => {
                info!("Received {} blocks from {}", blocks.len(), peer);

                let blockchain = self.blockchain.write().await;
                let mut added = 0;

                for block_data in blocks {
                    if let Ok(block) = bincode::deserialize::<Block>(&block_data) {
                        if let Ok(()) = blockchain.append_block(&block) {
                            added += 1;
                        }
                    }
                }

                info!("Added {} blocks to chain", added);
            }

            NetworkResponse::ChainTip {
                height,
                block_hash: _,
            } => {
                info!("Peer {} has chain height {}", peer, height);

                let local_height = self.get_chain_height().await?;
                if height > local_height {
                    info!(
                        "Peer ahead by {} blocks, requesting sync",
                        height - local_height
                    );
                    self.request_blocks(peer, local_height + 1, 500).await;
                }
            }

            NetworkResponse::Peers { peers } => {
                debug!("Received {} peer addresses", peers.len());
            }

            NetworkResponse::Error { message } => {
                warn!("Peer {} returned error: {}", peer, message);
            }
        }

        Ok(())
    }
}

// Re-export behaviour event type
pub use crate::behaviour::OpenSyriaBehaviourEvent;
