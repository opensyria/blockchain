use libp2p::PeerId;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Peer reputation tracking for network security
/// تتبع سمعة النظير لأمن الشبكة
pub struct PeerReputation {
    /// Reputation scores per peer
    scores: HashMap<PeerId, PeerScore>,
    /// Banned peers with ban expiration time
    banned_peers: HashMap<PeerId, Instant>,
    /// Last time decay was applied
    last_decay: Instant,
}

/// Individual peer reputation score
#[derive(Debug, Clone)]
pub struct PeerScore {
    /// Overall reputation score (starts at 0, can go negative)
    pub score: i32,
    /// Count of invalid blocks sent
    pub invalid_blocks: u32,
    /// Count of invalid transactions sent
    pub invalid_txs: u32,
    /// Count of valid blocks sent
    pub valid_blocks: u32,
    /// Count of valid transactions sent
    pub valid_txs: u32,
    /// Last time peer violated protocol
    pub last_violation: Option<Instant>,
    /// Peer creation time
    pub connected_at: Instant,
}

/// Reputation thresholds and penalties
pub const PEER_SCORE_THRESHOLD_BAN: i32 = -100;
pub const PEER_SCORE_THRESHOLD_WARN: i32 = -50;
pub const BAN_DURATION_SECS: u64 = 3600; // 1 hour
pub const DECAY_INTERVAL_SECS: u64 = 300; // 5 minutes
pub const DECAY_AMOUNT: i32 = 2; // Gradual forgiveness
pub const PENALTY_INVALID_BLOCK: i32 = -10;
pub const PENALTY_INVALID_TX: i32 = -2;
pub const PENALTY_RATE_LIMIT: i32 = -5;
pub const PENALTY_OVERSIZED_MSG: i32 = -15;
pub const REWARD_VALID_BLOCK: i32 = 2;
pub const REWARD_VALID_TX: i32 = 1;

impl PeerReputation {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            banned_peers: HashMap::new(),
            last_decay: Instant::now(),
        }
    }

    /// Register a new peer
    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.scores.entry(peer_id).or_insert(PeerScore {
            score: 0,
            invalid_blocks: 0,
            invalid_txs: 0,
            valid_blocks: 0,
            valid_txs: 0,
            last_violation: None,
            connected_at: Instant::now(),
        });
    }

    /// Check if peer is banned
    pub fn is_banned(&mut self, peer_id: &PeerId) -> bool {
        // Apply decay if interval elapsed
        self.maybe_apply_decay();

        if let Some(ban_expires) = self.banned_peers.get(peer_id) {
            if Instant::now() < *ban_expires {
                return true; // Still banned
            } else {
                // Ban expired, remove from banned list and restore some reputation
                self.banned_peers.remove(peer_id);
                if let Some(score) = self.scores.get_mut(peer_id) {
                    // Give a fresh start but not full reset
                    score.score = PEER_SCORE_THRESHOLD_WARN;
                }
                return false;
            }
        }
        false
    }

    /// Apply gradual reputation decay (move scores toward 0)
    fn maybe_apply_decay(&mut self) {
        if self.last_decay.elapsed() < Duration::from_secs(DECAY_INTERVAL_SECS) {
            return; // Not time yet
        }

        for score in self.scores.values_mut() {
            if score.score < 0 {
                // Negative scores move up toward 0
                score.score = (score.score + DECAY_AMOUNT).min(0);
            } else if score.score > 0 {
                // Positive scores decay slightly (prevent infinite accumulation)
                score.score = (score.score - 1).max(0);
            }
        }

        self.last_decay = Instant::now();
    }

    /// Manually trigger reputation decay (for testing)
    pub fn apply_decay(&mut self) {
        self.last_decay = Instant::now() - Duration::from_secs(DECAY_INTERVAL_SECS + 1);
        self.maybe_apply_decay();
    }

    /// Apply penalty for invalid block
    pub fn penalize_invalid_block(&mut self, peer_id: &PeerId) -> bool {
        if let Some(score) = self.scores.get_mut(peer_id) {
            score.score += PENALTY_INVALID_BLOCK;
            score.invalid_blocks += 1;
            score.last_violation = Some(Instant::now());
            return self.check_ban_threshold(peer_id);
        }
        false
    }

    /// Apply penalty for invalid transaction
    pub fn penalize_invalid_tx(&mut self, peer_id: &PeerId) -> bool {
        if let Some(score) = self.scores.get_mut(peer_id) {
            score.score += PENALTY_INVALID_TX;
            score.invalid_txs += 1;
            score.last_violation = Some(Instant::now());
            return self.check_ban_threshold(peer_id);
        }
        false
    }

    /// Apply penalty for rate limit violation
    pub fn penalize_rate_limit(&mut self, peer_id: &PeerId) -> bool {
        if let Some(score) = self.scores.get_mut(peer_id) {
            score.score += PENALTY_RATE_LIMIT;
            score.last_violation = Some(Instant::now());
            return self.check_ban_threshold(peer_id);
        }
        false
    }

    /// Apply penalty for oversized message
    pub fn penalize_oversized_msg(&mut self, peer_id: &PeerId) -> bool {
        if let Some(score) = self.scores.get_mut(peer_id) {
            score.score += PENALTY_OVERSIZED_MSG;
            score.last_violation = Some(Instant::now());
            return self.check_ban_threshold(peer_id);
        }
        false
    }

    /// Reward peer for valid block
    pub fn reward_valid_block(&mut self, peer_id: &PeerId) {
        if let Some(score) = self.scores.get_mut(peer_id) {
            score.score += REWARD_VALID_BLOCK;
            score.valid_blocks += 1;
        }
    }

    /// Reward peer for valid transaction
    pub fn reward_valid_tx(&mut self, peer_id: &PeerId) {
        if let Some(score) = self.scores.get_mut(peer_id) {
            score.score += REWARD_VALID_TX;
            score.valid_txs += 1;
        }
    }

    /// Check if peer score dropped below ban threshold
    fn check_ban_threshold(&mut self, peer_id: &PeerId) -> bool {
        if let Some(score) = self.scores.get(peer_id) {
            if score.score < PEER_SCORE_THRESHOLD_BAN {
                let ban_until = Instant::now() + Duration::from_secs(BAN_DURATION_SECS);
                self.banned_peers.insert(*peer_id, ban_until);
                return true; // Peer should be banned
            }
        }
        false
    }

    /// Get peer score
    pub fn get_score(&self, peer_id: &PeerId) -> Option<&PeerScore> {
        self.scores.get(peer_id)
    }

    /// Get peers with high reputation (for prioritization)
    pub fn get_high_reputation_peers(&self, min_score: i32, max_count: usize) -> Vec<PeerId> {
        let mut peers: Vec<_> = self
            .scores
            .iter()
            .filter(|(_, score)| score.score >= min_score)
            .map(|(peer_id, score)| (*peer_id, score.score))
            .collect();

        // Sort by score descending
        peers.sort_by_key(|(_, score)| std::cmp::Reverse(*score));

        peers
            .into_iter()
            .take(max_count)
            .map(|(peer_id, _)| peer_id)
            .collect()
    }

    /// Remove peer from tracking
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.scores.remove(peer_id);
        self.banned_peers.remove(peer_id);
    }
}

impl Default for PeerReputation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_reputation_ban() {
        let mut reputation = PeerReputation::new();
        let peer_id = PeerId::random();

        reputation.add_peer(peer_id);

        // Apply enough penalties to trigger ban (need > -100, so 11 penalties = -110)
        for _ in 0..11 {
            reputation.penalize_invalid_block(&peer_id);
        }

        // Should be banned
        assert!(reputation.is_banned(&peer_id));
    }

    #[test]
    fn test_peer_reputation_rewards() {
        let mut reputation = PeerReputation::new();
        let peer_id = PeerId::random();

        reputation.add_peer(peer_id);

        // Reward peer
        for _ in 0..10 {
            reputation.reward_valid_block(&peer_id);
        }

        let score = reputation.get_score(&peer_id).unwrap();
        assert_eq!(score.score, 20); // 10 blocks * 2 points
        assert_eq!(score.valid_blocks, 10);
    }
}
