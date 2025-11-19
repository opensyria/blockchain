/// Peer cache persistence for fallback peer discovery
/// ذاكرة التخزين المؤقت للأقران للاكتشاف الاحتياطي

use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;

/// Cached peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPeer {
    /// Peer ID
    pub peer_id: String,
    /// Multiaddress
    pub address: String,
    /// Last seen timestamp
    pub last_seen: u64,
    /// Connection success count
    pub success_count: u32,
    /// Connection failure count
    pub failure_count: u32,
}

/// Peer cache manager
pub struct PeerCache {
    /// Cache file path
    cache_path: PathBuf,
    /// Cached peers
    peers: HashMap<String, CachedPeer>,
}

impl PeerCache {
    /// Create a new peer cache
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            cache_path,
            peers: HashMap::new(),
        }
    }

    /// Load peers from disk
    pub fn load(&mut self) -> Result<()> {
        if !self.cache_path.exists() {
            return Ok(());
        }

        let data = std::fs::read_to_string(&self.cache_path)?;
        let peers: HashMap<String, CachedPeer> = serde_json::from_str(&data)?;
        self.peers = peers;

        tracing::info!("Loaded {} cached peers", self.peers.len());
        Ok(())
    }

    /// Save peers to disk
    pub fn save(&self) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(&self.peers)?;
        std::fs::write(&self.cache_path, data)?;

        tracing::debug!("Saved {} peers to cache", self.peers.len());
        Ok(())
    }

    /// Add or update a peer in the cache
    pub fn add_peer(&mut self, peer_id: PeerId, address: Multiaddr) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let key = peer_id.to_string();
        
        self.peers
            .entry(key.clone())
            .and_modify(|peer| {
                peer.last_seen = now;
                peer.address = address.to_string();
            })
            .or_insert(CachedPeer {
                peer_id: key,
                address: address.to_string(),
                last_seen: now,
                success_count: 0,
                failure_count: 0,
            });
    }

    /// Mark a peer connection as successful
    pub fn mark_success(&mut self, peer_id: &PeerId) {
        let key = peer_id.to_string();
        if let Some(peer) = self.peers.get_mut(&key) {
            peer.success_count += 1;
            peer.last_seen = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
    }

    /// Mark a peer connection as failed
    pub fn mark_failure(&mut self, peer_id: &PeerId) {
        let key = peer_id.to_string();
        if let Some(peer) = self.peers.get_mut(&key) {
            peer.failure_count += 1;
        }
    }

    /// Get reliable peers (sorted by success rate)
    pub fn get_reliable_peers(&self, max_count: usize) -> Vec<Multiaddr> {
        let mut peers: Vec<_> = self.peers.values().collect();
        
        // Sort by success rate (success_count - failure_count)
        peers.sort_by_key(|p| {
            std::cmp::Reverse(p.success_count.saturating_sub(p.failure_count))
        });

        peers
            .iter()
            .take(max_count)
            .filter_map(|p| p.address.parse().ok())
            .collect()
    }

    /// Get recent peers (within last N seconds)
    pub fn get_recent_peers(&self, max_age_secs: u64, max_count: usize) -> Vec<Multiaddr> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut peers: Vec<_> = self
            .peers
            .values()
            .filter(|p| now.saturating_sub(p.last_seen) < max_age_secs)
            .collect();

        // Sort by most recent
        peers.sort_by_key(|p| std::cmp::Reverse(p.last_seen));

        peers
            .iter()
            .take(max_count)
            .filter_map(|p| p.address.parse().ok())
            .collect()
    }

    /// Prune old peers from cache
    pub fn prune_old_peers(&mut self, max_age_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let before_count = self.peers.len();

        self.peers.retain(|_key, peer| {
            now.saturating_sub(peer.last_seen) < max_age_secs
        });

        let pruned = before_count - self.peers.len();
        if pruned > 0 {
            tracing::info!("Pruned {} old peers from cache", pruned);
        }
    }

    /// Get total cached peer count
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_peer_cache_save_load() {
        let dir = tempdir().unwrap();
        let cache_path = dir.path().join("peers.json");

        let mut cache = PeerCache::new(cache_path.clone());
        
        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/9000".parse().unwrap();
        
        cache.add_peer(peer_id, addr.clone());
        cache.save().unwrap();

        let mut cache2 = PeerCache::new(cache_path);
        cache2.load().unwrap();
        
        assert_eq!(cache2.len(), 1);
    }

    #[test]
    fn test_peer_cache_reliability_tracking() {
        let dir = tempdir().unwrap();
        let mut cache = PeerCache::new(dir.path().join("peers.json"));

        let peer1 = PeerId::random();
        let peer2 = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/9000".parse().unwrap();

        cache.add_peer(peer1, addr.clone());
        cache.add_peer(peer2, addr.clone());

        cache.mark_success(&peer1);
        cache.mark_success(&peer1);
        cache.mark_failure(&peer2);

        let reliable = cache.get_reliable_peers(10);
        assert_eq!(reliable.len(), 2);
    }

    #[test]
    fn test_peer_cache_pruning() {
        let dir = tempdir().unwrap();
        let mut cache = PeerCache::new(dir.path().join("peers.json"));

        let peer_id = PeerId::random();
        let addr: Multiaddr = "/ip4/127.0.0.1/tcp/9000".parse().unwrap();
        
        cache.add_peer(peer_id, addr);
        assert_eq!(cache.len(), 1);

        // Prune peers older than 0 seconds (should remove all)
        cache.prune_old_peers(0);
        assert_eq!(cache.len(), 0);
    }
}
