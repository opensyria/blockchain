/// OpenSyria Blockchain Metrics for Prometheus
/// مقاييس سلسلة كتل OpenSyria لبروميثيوس
///
/// ✅  OPERATIONAL READINESS (P1-005): Prometheus metrics for monitoring
/// Provides comprehensive metrics for Grafana dashboards and alerting

pub mod server;

use lazy_static::lazy_static;
use prometheus::{
    register_gauge, register_gauge_vec, register_histogram_vec, register_int_counter_vec,
    register_int_gauge, register_int_gauge_vec, Encoder, Gauge, GaugeVec, HistogramVec,
    IntCounterVec, IntGauge, IntGaugeVec, TextEncoder,
};

lazy_static! {
    // Blockchain metrics
    /// Current blockchain height
    pub static ref CHAIN_HEIGHT: IntGauge = register_int_gauge!(
        "opensyria_chain_height",
        "Current height of the blockchain"
    )
    .unwrap();

    /// Total supply of SYL in circulation
    pub static ref TOTAL_SUPPLY: Gauge = register_gauge!(
        "opensyria_total_supply_syl",
        "Total supply of SYL tokens in circulation"
    )
    .unwrap();

    /// Current block time in seconds
    pub static ref BLOCK_TIME: Gauge = register_gauge!(
        "opensyria_block_time_seconds",
        "Time between last two blocks in seconds"
    )
    .unwrap();

    /// Current difficulty target
    pub static ref DIFFICULTY: IntGauge = register_int_gauge!(
        "opensyria_difficulty",
        "Current mining difficulty target"
    )
    .unwrap();

    // Network metrics
    /// Number of connected peers
    pub static ref PEER_COUNT: IntGauge = register_int_gauge!(
        "opensyria_peer_count",
        "Number of connected peers"
    )
    .unwrap();

    /// Inbound peer connections
    pub static ref INBOUND_PEERS: IntGauge = register_int_gauge!(
        "opensyria_inbound_peers",
        "Number of inbound peer connections"
    )
    .unwrap();

    /// Outbound peer connections
    pub static ref OUTBOUND_PEERS: IntGauge = register_int_gauge!(
        "opensyria_outbound_peers",
        "Number of outbound peer connections"
    )
    .unwrap();

    /// Bytes received from network
    pub static ref NETWORK_RX_BYTES: IntCounterVec = register_int_counter_vec!(
        "opensyria_network_rx_bytes_total",
        "Total bytes received from network",
        &["message_type"]
    )
    .unwrap();

    /// Bytes transmitted to network
    pub static ref NETWORK_TX_BYTES: IntCounterVec = register_int_counter_vec!(
        "opensyria_network_tx_bytes_total",
        "Total bytes transmitted to network",
        &["message_type"]
    )
    .unwrap();

    // Mempool metrics
    /// Current mempool size (number of transactions)
    pub static ref MEMPOOL_SIZE: IntGauge = register_int_gauge!(
        "opensyria_mempool_size",
        "Number of transactions in mempool"
    )
    .unwrap();

    /// Total mempool bytes
    pub static ref MEMPOOL_BYTES: IntGauge = register_int_gauge!(
        "opensyria_mempool_bytes",
        "Total size of mempool in bytes"
    )
    .unwrap();

    /// Transactions accepted to mempool
    pub static ref MEMPOOL_ACCEPTED: IntCounterVec = register_int_counter_vec!(
        "opensyria_mempool_accepted_total",
        "Total transactions accepted to mempool",
        &["tx_type"]
    )
    .unwrap();

    /// Transactions rejected from mempool
    pub static ref MEMPOOL_REJECTED: IntCounterVec = register_int_counter_vec!(
        "opensyria_mempool_rejected_total",
        "Total transactions rejected from mempool",
        &["reason"]
    )
    .unwrap();

    // Mining metrics
    /// Current hashrate estimate (hashes per second)
    pub static ref HASHRATE: Gauge = register_gauge!(
        "opensyria_hashrate",
        "Estimated network hashrate in hashes/second"
    )
    .unwrap();

    /// Blocks mined by this node
    pub static ref BLOCKS_MINED: IntCounterVec = register_int_counter_vec!(
        "opensyria_blocks_mined_total",
        "Total blocks mined by this node",
        &["status"]
    )
    .unwrap();

    // Storage metrics
    /// Database size in bytes
    pub static ref DB_SIZE: IntGaugeVec = register_int_gauge_vec!(
        "opensyria_db_size_bytes",
        "Database size in bytes",
        &["db_name"]
    )
    .unwrap();

    /// RocksDB cache hit rate
    pub static ref DB_CACHE_HITS: IntCounterVec = register_int_counter_vec!(
        "opensyria_db_cache_hits_total",
        "Database cache hits",
        &["db_name"]
    )
    .unwrap();

    /// RocksDB cache misses
    pub static ref DB_CACHE_MISSES: IntCounterVec = register_int_counter_vec!(
        "opensyria_db_cache_misses_total",
        "Database cache misses",
        &["db_name"]
    )
    .unwrap();

    // Transaction metrics
    /// Transactions processed
    pub static ref TRANSACTIONS_PROCESSED: IntCounterVec = register_int_counter_vec!(
        "opensyria_transactions_processed_total",
        "Total transactions processed",
        &["status"]
    )
    .unwrap();

    /// Transaction fees collected
    pub static ref TRANSACTION_FEES: Gauge = register_gauge!(
        "opensyria_transaction_fees_syl",
        "Total transaction fees collected in SYL"
    )
    .unwrap();

    // Governance metrics
    /// Active governance proposals
    pub static ref ACTIVE_PROPOSALS: IntGauge = register_int_gauge!(
        "opensyria_active_proposals",
        "Number of active governance proposals"
    )
    .unwrap();

    /// Total governance votes cast
    pub static ref GOVERNANCE_VOTES: IntCounterVec = register_int_counter_vec!(
        "opensyria_governance_votes_total",
        "Total governance votes cast",
        &["proposal_id", "vote_type"]
    )
    .unwrap();

    // Identity/NFT metrics
    /// Total identity tokens issued
    pub static ref IDENTITY_TOKENS: IntGauge = register_int_gauge!(
        "opensyria_identity_tokens",
        "Total identity/NFT tokens issued"
    )
    .unwrap();

    // Performance metrics
    /// Block validation time histogram
    pub static ref BLOCK_VALIDATION_TIME: HistogramVec = register_histogram_vec!(
        "opensyria_block_validation_seconds",
        "Time to validate a block in seconds",
        &["result"],
        vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0]
    )
    .unwrap();

    /// Transaction validation time histogram
    pub static ref TX_VALIDATION_TIME: HistogramVec = register_histogram_vec!(
        "opensyria_tx_validation_seconds",
        "Time to validate a transaction in seconds",
        &["result"],
        vec![0.0001, 0.001, 0.01, 0.1, 0.5, 1.0]
    )
    .unwrap();

    /// State query time histogram
    pub static ref STATE_QUERY_TIME: HistogramVec = register_histogram_vec!(
        "opensyria_state_query_seconds",
        "Time to query state in seconds",
        &["operation"],
        vec![0.0001, 0.001, 0.01, 0.1, 0.5]
    )
    .unwrap();

    // Sync metrics
    /// Sync progress percentage (0-100)
    pub static ref SYNC_PROGRESS: Gauge = register_gauge!(
        "opensyria_sync_progress_percent",
        "Blockchain sync progress percentage"
    )
    .unwrap();

    /// Blocks behind tip
    pub static ref BLOCKS_BEHIND: IntGauge = register_int_gauge!(
        "opensyria_blocks_behind",
        "Number of blocks behind network tip"
    )
    .unwrap();

    // System metrics
    /// Node uptime in seconds
    pub static ref NODE_UPTIME: IntGauge = register_int_gauge!(
        "opensyria_node_uptime_seconds",
        "Node uptime in seconds"
    )
    .unwrap();
}

/// Get all metrics in Prometheus text format
pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

/// Update blockchain metrics
pub fn update_chain_metrics(height: u64, supply: u64, difficulty: u64) {
    CHAIN_HEIGHT.set(height as i64);
    TOTAL_SUPPLY.set((supply as f64) / 100_000_000.0); // Convert from base units to SYL
    DIFFICULTY.set(difficulty as i64);
}

/// Update network metrics
pub fn update_network_metrics(total_peers: usize, inbound: usize, outbound: usize) {
    PEER_COUNT.set(total_peers as i64);
    INBOUND_PEERS.set(inbound as i64);
    OUTBOUND_PEERS.set(outbound as i64);
}

/// Update mempool metrics
pub fn update_mempool_metrics(tx_count: usize, total_bytes: usize) {
    MEMPOOL_SIZE.set(tx_count as i64);
    MEMPOOL_BYTES.set(total_bytes as i64);
}

/// Update sync metrics
pub fn update_sync_metrics(current_height: u64, target_height: u64) {
    if target_height > 0 {
        let progress = (current_height as f64 / target_height as f64) * 100.0;
        SYNC_PROGRESS.set(progress.min(100.0));
        BLOCKS_BEHIND.set((target_height.saturating_sub(current_height)) as i64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_metrics() {
        update_chain_metrics(1000, 50_000_000_000_000, 12345);
        assert_eq!(CHAIN_HEIGHT.get(), 1000);
        assert_eq!(DIFFICULTY.get(), 12345);
    }

    #[test]
    fn test_network_metrics() {
        update_network_metrics(25, 15, 10);
        assert_eq!(PEER_COUNT.get(), 25);
        assert_eq!(INBOUND_PEERS.get(), 15);
        assert_eq!(OUTBOUND_PEERS.get(), 10);
    }

    #[test]
    fn test_mempool_metrics() {
        update_mempool_metrics(100, 50000);
        assert_eq!(MEMPOOL_SIZE.get(), 100);
        assert_eq!(MEMPOOL_BYTES.get(), 50000);
    }

    #[test]
    fn test_sync_metrics() {
        update_sync_metrics(500, 1000);
        assert_eq!(SYNC_PROGRESS.get(), 50.0);
        assert_eq!(BLOCKS_BEHIND.get(), 500);
    }

    #[test]
    fn test_gather_metrics() {
        update_chain_metrics(100, 10_000_000_000, 1000);
        let metrics = gather_metrics();
        assert!(metrics.contains("opensyria_chain_height"));
        assert!(metrics.contains("opensyria_total_supply_syl"));
    }
}
