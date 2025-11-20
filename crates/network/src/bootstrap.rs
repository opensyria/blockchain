/// Bootstrap node configuration for network discovery
/// تكوين عقد التمهيد لاكتشاف الشبكة
/// 
/// NET-P2-009: Decentralized bootstrap mechanism
/// 
/// Provides multiple discovery methods to prevent single point of failure:
/// 1. DNS seeds - Query DNS for peer addresses
/// 2. Hardcoded bootstrap nodes - Fallback when DNS fails
/// 3. Peer cache - Previously connected peers
/// 4. Peer exchange (PEX) - Get peers from peers

use libp2p::Multiaddr;
use std::net::{IpAddr, ToSocketAddrs};

/// Network type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
}

/// Mainnet bootstrap nodes (to be updated with real production nodes)
/// عقد التمهيد للشبكة الرئيسية (سيتم تحديثها بالعقد الإنتاجية الحقيقية)
pub const MAINNET_BOOTSTRAP_NODES: &[&str] = &[
    // Syria-based nodes (to be deployed)
    "/dns4/node1.opensyria.network/tcp/9000",
    "/dns4/node2.opensyria.network/tcp/9000",
    "/dns4/node3.opensyria.network/tcp/9000",
    
    // Regional nodes - Middle East (to be deployed)
    "/dns4/me-node1.opensyria.network/tcp/9000",
    "/dns4/me-node2.opensyria.network/tcp/9000",
    
    // Global fallback nodes (to be deployed)
    // These will be replaced with actual IP addresses once infrastructure is ready
    // "/ip4/195.201.94.21/tcp/9000",  // Europe (Germany)
    // "/ip4/167.99.14.235/tcp/9000",  // US East
    // "/ip4/146.190.102.45/tcp/9000", // Asia (Singapore)
];

/// Testnet bootstrap nodes
/// عقد التمهيد لشبكة الاختبار
pub const TESTNET_BOOTSTRAP_NODES: &[&str] = &[
    "/dns4/testnet1.opensyria.network/tcp/19000",
    "/dns4/testnet2.opensyria.network/tcp/19000",
    "/ip4/127.0.0.1/tcp/19000", // Local testnet node
];

/// Get bootstrap peers for specified network
/// الحصول على عقد التمهيد للشبكة المحددة
pub fn get_bootstrap_peers(network: NetworkType) -> Vec<Multiaddr> {
    let nodes = match network {
        NetworkType::Mainnet => MAINNET_BOOTSTRAP_NODES,
        NetworkType::Testnet => TESTNET_BOOTSTRAP_NODES,
    };

    nodes
        .iter()
        .filter_map(|addr| addr.parse().ok())
        .collect()
}

/// Check if bootstrap peers list is empty (development mode)
/// التحقق من أن قائمة عقد التمهيد فارغة (وضع التطوير)
pub fn has_bootstrap_peers(network: NetworkType) -> bool {
    !get_bootstrap_peers(network).is_empty()
}

/// DNS seed domains for peer discovery
/// 
/// DNS seeds provide decentralized peer discovery by querying multiple
/// independent DNS servers. This prevents single point of failure.
pub const MAINNET_DNS_SEEDS: &[&str] = &[
    "seed1.opensyria.network",
    "seed2.opensyria.network",
    "seed3.opensyria.network",
    "dnsseed.opensyria.network",
];

pub const TESTNET_DNS_SEEDS: &[&str] = &[
    "testseed.opensyria.network",
];

/// Query DNS seeds for peer addresses
/// 
/// NET-P2-009: DNS seed-based peer discovery
/// 
/// Returns list of peer addresses from DNS queries. Multiple DNS seeds
/// ensure discovery works even if some seeds are down.
pub fn query_dns_seeds(network: NetworkType) -> Vec<Multiaddr> {
    let seeds = match network {
        NetworkType::Mainnet => MAINNET_DNS_SEEDS,
        NetworkType::Testnet => TESTNET_DNS_SEEDS,
    };

    let mut peers = Vec::new();

    for seed in seeds {
        match query_single_dns_seed(seed) {
            Ok(mut addresses) => {
                tracing::info!("DNS seed {} returned {} peers", seed, addresses.len());
                peers.append(&mut addresses);
            }
            Err(e) => {
                tracing::warn!("Failed to query DNS seed {}: {}", seed, e);
            }
        }
    }

    // Deduplicate peers
    peers.sort();
    peers.dedup();

    tracing::info!("DNS discovery found {} unique peers", peers.len());
    peers
}

/// Query a single DNS seed
fn query_single_dns_seed(seed: &str) -> Result<Vec<Multiaddr>, std::io::Error> {
    // Query DNS with timeout
    let port = 9000; // Default OpenSyria port
    let socket_addr_str = format!("{}:{}", seed, port);
    
    let addresses: Vec<Multiaddr> = socket_addr_str
        .to_socket_addrs()?
        .take(10) // Limit to 10 addresses per seed
        .map(|addr| {
            let ip = addr.ip();
            let port = addr.port();
            match ip {
                IpAddr::V4(ipv4) => format!("/ip4/{}/tcp/{}", ipv4, port),
                IpAddr::V6(ipv6) => format!("/ip6/{}/tcp/{}", ipv6, port),
            }
            .parse()
            .unwrap()
        })
        .collect();

    Ok(addresses)
}

/// Bootstrap configuration with multiple discovery methods
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// Network type (mainnet/testnet)
    pub network: NetworkType,
    
    /// Enable DNS seed discovery
    pub use_dns_seeds: bool,
    
    /// Enable hardcoded bootstrap nodes
    pub use_hardcoded_peers: bool,
    
    /// Enable peer cache (previously connected peers)
    pub use_peer_cache: bool,
    
    /// Maximum peers to bootstrap from
    pub max_bootstrap_peers: usize,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            network: NetworkType::Testnet,
            use_dns_seeds: true,
            use_hardcoded_peers: true,
            use_peer_cache: true,
            max_bootstrap_peers: 50,
        }
    }
}

impl BootstrapConfig {
    /// Create mainnet bootstrap config
    pub fn mainnet() -> Self {
        Self {
            network: NetworkType::Mainnet,
            ..Default::default()
        }
    }

    /// Create testnet bootstrap config
    pub fn testnet() -> Self {
        Self {
            network: NetworkType::Testnet,
            ..Default::default()
        }
    }

    /// Discover peers using all enabled methods
    /// 
    /// Tries discovery methods in order of preference:
    /// 1. Peer cache (fastest, most reliable)
    /// 2. DNS seeds (decentralized)
    /// 3. Hardcoded bootstrap nodes (last resort)
    pub fn discover_peers(&self) -> Vec<Multiaddr> {
        let mut peers = Vec::new();

        // Method 1: Try peer cache first (if available)
        if self.use_peer_cache {
            // Peer cache is loaded separately by the network layer
            // This is a placeholder for integration
            tracing::debug!("Peer cache will be checked by network layer");
        }

        // Method 2: Query DNS seeds
        if self.use_dns_seeds {
            let mut dns_peers = query_dns_seeds(self.network);
            peers.append(&mut dns_peers);
        }

        // Method 3: Use hardcoded bootstrap nodes as fallback
        if self.use_hardcoded_peers {
            let mut hardcoded = get_bootstrap_peers(self.network);
            peers.append(&mut hardcoded);
        }

        // Deduplicate and limit
        peers.sort();
        peers.dedup();
        peers.truncate(self.max_bootstrap_peers);

        if peers.is_empty() {
            tracing::warn!("No bootstrap peers discovered! Node may not connect to network");
        } else {
            tracing::info!("Discovered {} bootstrap peers", peers.len());
        }

        peers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mainnet_bootstrap_parsing() {
        let peers = get_bootstrap_peers(NetworkType::Mainnet);
        // At minimum, DNS entries should parse correctly
        assert!(!peers.is_empty(), "Should have at least one bootstrap peer");
    }

    #[test]
    fn test_testnet_bootstrap_parsing() {
        let peers = get_bootstrap_peers(NetworkType::Testnet);
        assert!(!peers.is_empty(), "Testnet should have bootstrap peers");
        
        // Testnet should include localhost
        let has_localhost = peers.iter().any(|addr| addr.to_string().contains("127.0.0.1"));
        assert!(has_localhost, "Testnet should include localhost bootstrap");
    }

    #[test]
    fn test_bootstrap_peer_availability() {
        assert!(has_bootstrap_peers(NetworkType::Mainnet));
        assert!(has_bootstrap_peers(NetworkType::Testnet));
    }

    #[test]
    fn test_bootstrap_config_discovery() {
        let config = BootstrapConfig::testnet();
        let peers = config.discover_peers();
        
        // Should discover at least hardcoded peers
        assert!(!peers.is_empty(), "Should discover at least some peers");
    }

    #[test]
    fn test_bootstrap_config_peer_limit() {
        let mut config = BootstrapConfig::testnet();
        config.max_bootstrap_peers = 5;
        
        let peers = config.discover_peers();
        assert!(peers.len() <= 5, "Should respect max_bootstrap_peers limit");
    }

    #[test]
    fn test_bootstrap_with_only_hardcoded() {
        let mut config = BootstrapConfig::testnet();
        config.use_dns_seeds = false;
        config.use_peer_cache = false;
        config.use_hardcoded_peers = true;
        
        let peers = config.discover_peers();
        assert!(!peers.is_empty(), "Hardcoded peers should be available");
    }

    #[test]
    fn test_dns_seed_format() {
        // Test that DNS seed domains are valid
        for seed in MAINNET_DNS_SEEDS {
            assert!(!seed.is_empty());
            assert!(seed.contains('.'), "DNS seed should be a domain name");
        }
    }
}
