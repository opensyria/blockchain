/// Bootstrap node configuration for network discovery
/// تكوين عقد التمهيد لاكتشاف الشبكة

use libp2p::Multiaddr;

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
}
