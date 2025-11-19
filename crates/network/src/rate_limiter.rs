use libp2p::PeerId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Message rate limiter for DOS protection
/// محدد معدل الرسائل للحماية من هجمات الحرمان من الخدمة
pub struct RateLimiter {
    /// Per-peer rate limit state
    peer_limits: HashMap<PeerId, PeerRateLimit>,
}

/// Rate limit state for individual peer
#[derive(Debug, Clone)]
struct PeerRateLimit {
    /// Blocks received in current window
    blocks_received: u32,
    /// Transactions received in current window
    txs_received: u32,
    /// Bytes received in current window
    bytes_received: u64,
    /// Last window reset time
    last_reset: Instant,
}

/// Rate limit constants
pub const MAX_BLOCKS_PER_SECOND: u32 = 10;
pub const MAX_TXS_PER_SECOND: u32 = 100;
pub const MAX_BYTES_PER_SECOND: u64 = 5_000_000; // 5 MB/sec
pub const RATE_LIMIT_WINDOW_SECS: u64 = 1;

/// Message type for rate limiting
#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    Block,
    Transaction,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            peer_limits: HashMap::new(),
        }
    }

    /// Check if peer has exceeded rate limit for message type
    /// Returns true if rate limit is OK, false if exceeded
    pub fn check_rate_limit(&mut self, peer_id: &PeerId, msg_type: MessageType) -> bool {
        let limit = self.peer_limits.entry(*peer_id).or_insert(PeerRateLimit {
            blocks_received: 0,
            txs_received: 0,
            bytes_received: 0,
            last_reset: Instant::now(),
        });

        // Reset counters if window expired
        if limit.last_reset.elapsed() > Duration::from_secs(RATE_LIMIT_WINDOW_SECS) {
            limit.blocks_received = 0;
            limit.txs_received = 0;
            limit.bytes_received = 0;
            limit.last_reset = Instant::now();
        }

        // Check limit based on message type
        match msg_type {
            MessageType::Block => {
                limit.blocks_received += 1;
                if limit.blocks_received > MAX_BLOCKS_PER_SECOND {
                    return false; // Rate limit exceeded
                }
            }
            MessageType::Transaction => {
                limit.txs_received += 1;
                if limit.txs_received > MAX_TXS_PER_SECOND {
                    return false; // Rate limit exceeded
                }
            }
        }

        true // Rate limit OK
    }

    /// Check bandwidth limit for peer
    /// Returns true if bandwidth is OK, false if exceeded
    pub fn check_bandwidth_limit(&mut self, peer_id: &PeerId, message_size: u64) -> bool {
        let limit = self.peer_limits.entry(*peer_id).or_insert(PeerRateLimit {
            blocks_received: 0,
            txs_received: 0,
            bytes_received: 0,
            last_reset: Instant::now(),
        });

        // Reset counters if window expired
        if limit.last_reset.elapsed() > Duration::from_secs(RATE_LIMIT_WINDOW_SECS) {
            limit.blocks_received = 0;
            limit.txs_received = 0;
            limit.bytes_received = 0;
            limit.last_reset = Instant::now();
        }

        // Add message size to bytes received
        limit.bytes_received += message_size;

        // Check if bandwidth limit exceeded
        if limit.bytes_received > MAX_BYTES_PER_SECOND {
            return false; // Bandwidth limit exceeded
        }

        true // Bandwidth OK
    }

    /// Check both rate and bandwidth limits
    /// Returns true if all limits are OK, false if any exceeded
    pub fn check_all_limits(
        &mut self,
        peer_id: &PeerId,
        msg_type: MessageType,
        message_size: u64,
    ) -> bool {
        self.check_rate_limit(peer_id, msg_type) && self.check_bandwidth_limit(peer_id, message_size)
    }

    /// Remove peer from rate limiting
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.peer_limits.remove(peer_id);
    }

    /// Get current statistics for a peer (blocks, txs, bytes)
    pub fn get_stats(&self, peer_id: &PeerId) -> Option<(u32, u32, u64)> {
        self.peer_limits
            .get(peer_id)
            .map(|limit| (limit.blocks_received, limit.txs_received, limit.bytes_received))
    }

    /// Cleanup stale peer limits (memory leak prevention)
    pub fn cleanup_stale_peers(&mut self, max_age_secs: u64) {
        let now = Instant::now();
        self.peer_limits.retain(|_peer_id, limit| {
            now.duration_since(limit.last_reset).as_secs() < max_age_secs
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_blocks() {
        let mut limiter = RateLimiter::new();
        let peer_id = PeerId::random();

        // Send MAX_BLOCKS_PER_SECOND blocks - should all pass
        for _ in 0..MAX_BLOCKS_PER_SECOND {
            assert!(limiter.check_rate_limit(&peer_id, MessageType::Block));
        }

        // Next block should be rate limited
        assert!(!limiter.check_rate_limit(&peer_id, MessageType::Block));
    }

    #[test]
    fn test_rate_limiter_transactions() {
        let mut limiter = RateLimiter::new();
        let peer_id = PeerId::random();

        // Send MAX_TXS_PER_SECOND transactions - should all pass
        for _ in 0..MAX_TXS_PER_SECOND {
            assert!(limiter.check_rate_limit(&peer_id, MessageType::Transaction));
        }

        // Next transaction should be rate limited
        assert!(!limiter.check_rate_limit(&peer_id, MessageType::Transaction));
    }

    #[test]
    fn test_rate_limiter_window_reset() {
        let mut limiter = RateLimiter::new();
        let peer_id = PeerId::random();

        // Fill up the limit
        for _ in 0..MAX_BLOCKS_PER_SECOND {
            limiter.check_rate_limit(&peer_id, MessageType::Block);
        }

        // Manually reset the window
        if let Some(limit) = limiter.peer_limits.get_mut(&peer_id) {
            limit.last_reset = Instant::now() - Duration::from_secs(2);
        }

        // Should pass now after window reset
        assert!(limiter.check_rate_limit(&peer_id, MessageType::Block));
    }
}
