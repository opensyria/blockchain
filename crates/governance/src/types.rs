use opensyria_core::crypto::PublicKey;
use serde::{Deserialize, Serialize};

/// Unique identifier for a proposal
pub type ProposalId = u64;

/// Governance proposal types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    /// Change block difficulty adjustment parameters
    DifficultyAdjustment {
        target_block_time: u64,   // seconds
        adjustment_interval: u32, // blocks
    },

    /// Change minimum transaction fee
    MinimumFee { new_fee: u64 },

    /// Change block size limit
    BlockSizeLimit {
        new_limit: usize, // bytes
    },

    /// Change block reward (if implemented)
    BlockReward { new_reward: u64 },

    /// Treasury spending proposal
    TreasurySpending {
        recipient: PublicKey,
        amount: u64,
        description: String,
    },

    /// Protocol upgrade
    ProtocolUpgrade {
        version: u32,
        activation_height: u64,
        description: String,
    },

    /// Custom text proposal (non-binding)
    TextProposal { description: String },
}

/// Voting choice
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

/// Individual vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRecord {
    pub voter: PublicKey,
    pub vote: Vote,
    pub voting_power: u64, // Based on stake/balance at proposal creation
    pub timestamp: u64,
}

/// Proposal status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is open for voting
    Active,
    /// Voting period ended, passed
    Passed,
    /// Voting period ended, failed
    Rejected,
    /// Proposal was cancelled by proposer
    Cancelled,
    /// Passed and executed
    Executed,
}

/// Governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: PublicKey,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub created_at: u64,      // Block height
    pub voting_start: u64,    // Block height
    pub voting_end: u64,      // Block height
    pub execution_delay: u64, // Blocks after passage before execution
    pub status: ProposalStatus,
    pub required_quorum: u64,    // Percentage (0-100)
    pub required_threshold: u64, // Percentage (0-100) of yes votes
    pub votes_yes: u64,
    pub votes_no: u64,
    pub votes_abstain: u64,
    pub total_voting_power: u64, // Total stake at proposal creation
}

impl Proposal {
    /// Create a new proposal
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: ProposalId,
        proposer: PublicKey,
        proposal_type: ProposalType,
        title: String,
        description: String,
        created_at: u64,
        voting_period: u64,
        execution_delay: u64,
        total_voting_power: u64,
    ) -> Self {
        // Different proposal types have different requirements
        let (required_quorum, required_threshold) = match &proposal_type {
            ProposalType::ProtocolUpgrade { .. } => (50, 75), // 50% quorum, 75% yes
            ProposalType::TreasurySpending { .. } => (40, 66), // 40% quorum, 66% yes
            ProposalType::DifficultyAdjustment { .. } => (30, 60),
            ProposalType::MinimumFee { .. } => (30, 60),
            ProposalType::BlockSizeLimit { .. } => (30, 60),
            ProposalType::BlockReward { .. } => (40, 66),
            ProposalType::TextProposal { .. } => (20, 50), // Simple majority
        };

        Self {
            id,
            proposer,
            proposal_type,
            title,
            description,
            created_at,
            voting_start: created_at,
            voting_end: created_at + voting_period,
            execution_delay,
            status: ProposalStatus::Active,
            required_quorum,
            required_threshold,
            votes_yes: 0,
            votes_no: 0,
            votes_abstain: 0,
            total_voting_power,
        }
    }

    /// Check if voting period is active
    pub fn is_active(&self, current_height: u64) -> bool {
        self.status == ProposalStatus::Active
            && current_height >= self.voting_start
            && current_height < self.voting_end
    }

    /// Check if voting has ended
    pub fn has_ended(&self, current_height: u64) -> bool {
        current_height >= self.voting_end
    }

    /// Calculate current participation rate (percentage)
    pub fn participation_rate(&self) -> u64 {
        if self.total_voting_power == 0 {
            return 0;
        }
        let total_votes = self.votes_yes + self.votes_no + self.votes_abstain;
        (total_votes * 100) / self.total_voting_power
    }

    /// Calculate yes vote percentage (of total votes cast)
    pub fn yes_percentage(&self) -> u64 {
        let total_votes = self.votes_yes + self.votes_no + self.votes_abstain;
        if total_votes == 0 {
            return 0;
        }
        (self.votes_yes * 100) / total_votes
    }

    /// Check if quorum is met
    pub fn meets_quorum(&self) -> bool {
        self.participation_rate() >= self.required_quorum
    }

    /// Check if threshold is met (among votes cast)
    pub fn meets_threshold(&self) -> bool {
        self.yes_percentage() >= self.required_threshold
    }

    /// Finalize proposal after voting ends
    pub fn finalize(&mut self, current_height: u64) {
        if !self.has_ended(current_height) {
            return;
        }

        if self.status != ProposalStatus::Active {
            return;
        }

        if self.meets_quorum() && self.meets_threshold() {
            self.status = ProposalStatus::Passed;
        } else {
            self.status = ProposalStatus::Rejected;
        }
    }

    /// Mark proposal as executed
    pub fn mark_executed(&mut self) {
        if self.status == ProposalStatus::Passed {
            self.status = ProposalStatus::Executed;
        }
    }

    /// Cancel proposal (only by proposer before voting ends)
    pub fn cancel(&mut self) {
        if self.status == ProposalStatus::Active {
            self.status = ProposalStatus::Cancelled;
        }
    }

    /// Check if proposal is ready for execution
    pub fn ready_for_execution(&self, current_height: u64) -> bool {
        self.status == ProposalStatus::Passed
            && current_height >= self.voting_end + self.execution_delay
    }
}

/// Governance configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Minimum stake required to create a proposal
    pub min_proposal_stake: u64,

    /// Default voting period in blocks
    pub default_voting_period: u64,

    /// Default execution delay in blocks
    pub default_execution_delay: u64,

    /// Whether governance is enabled
    pub enabled: bool,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            min_proposal_stake: 1_000_000_000, // 1000 Lira minimum
            default_voting_period: 10_080,     // ~1 week at 1 min blocks
            default_execution_delay: 1_440,    // ~1 day at 1 min blocks
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;

    #[test]
    fn test_proposal_creation() {
        let proposer = KeyPair::generate();
        let proposal = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::TextProposal {
                description: "Test proposal".to_string(),
            },
            "Test".to_string(),
            "Description".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );

        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.status, ProposalStatus::Active);
        assert_eq!(proposal.voting_start, 100);
        assert_eq!(proposal.voting_end, 1100);
    }

    #[test]
    fn test_proposal_voting_period() {
        let proposer = KeyPair::generate();
        let proposal = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::TextProposal {
                description: "Test".to_string(),
            },
            "Test".to_string(),
            "Desc".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );

        assert!(proposal.is_active(100));
        assert!(proposal.is_active(500));
        assert!(proposal.is_active(1099));
        assert!(!proposal.is_active(1100));
        assert!(!proposal.is_active(50));
    }

    #[test]
    fn test_quorum_and_threshold() {
        let proposer = KeyPair::generate();
        let mut proposal = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::TextProposal {
                description: "Test".to_string(),
            },
            "Test".to_string(),
            "Desc".to_string(),
            100,
            1000,
            100,
            1_000_000, // Total voting power
        );

        // 20% quorum, 50% threshold for text proposals
        proposal.votes_yes = 150_000; // 15%
        proposal.votes_no = 50_000; // 5%
                                    // Total: 20%, quorum met exactly
        assert!(proposal.meets_quorum()); // 20% participation meets 20% quorum

        proposal.votes_yes = 250_000; // 25% yes
        proposal.votes_no = 50_000; // 5% no
                                    // Total: 30%, quorum met, 83% yes rate
        assert!(proposal.meets_quorum());
        assert!(proposal.meets_threshold());
    }

    #[test]
    fn test_proposal_finalization() {
        let proposer = KeyPair::generate();
        let mut proposal = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::MinimumFee { new_fee: 200 },
            "Increase fee".to_string(),
            "Double the fee".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );

        // Meets quorum (30%) and threshold (60%)
        proposal.votes_yes = 350_000; // 35%
        proposal.votes_no = 50_000; // 5%

        proposal.finalize(1100);
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }

    #[test]
    fn test_proposal_rejection() {
        let proposer = KeyPair::generate();
        let mut proposal = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::MinimumFee { new_fee: 200 },
            "Test".to_string(),
            "Desc".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );

        // Meets quorum but not threshold
        proposal.votes_yes = 150_000; // 15%
        proposal.votes_no = 250_000; // 25%
                                     // 40% participation (quorum met), but only 37.5% yes (threshold not met)

        proposal.finalize(1100);
        assert_eq!(proposal.status, ProposalStatus::Rejected);
    }

    #[test]
    fn test_execution_readiness() {
        let proposer = KeyPair::generate();
        let mut proposal = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::MinimumFee { new_fee: 200 },
            "Test".to_string(),
            "Desc".to_string(),
            100,
            1000,
            100, // 100 block execution delay
            1_000_000,
        );

        proposal.votes_yes = 700_000;
        proposal.finalize(1100);
        assert_eq!(proposal.status, ProposalStatus::Passed);

        // Not ready immediately after voting ends
        assert!(!proposal.ready_for_execution(1100));

        // Ready after execution delay
        assert!(proposal.ready_for_execution(1200));
    }

    #[test]
    fn test_different_proposal_thresholds() {
        let proposer = KeyPair::generate();

        // Protocol upgrade: 50% quorum, 75% threshold
        let protocol = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::ProtocolUpgrade {
                version: 2,
                activation_height: 10000,
                description: "Test".to_string(),
            },
            "Upgrade".to_string(),
            "Desc".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );
        assert_eq!(protocol.required_quorum, 50);
        assert_eq!(protocol.required_threshold, 75);

        // Text proposal: 20% quorum, 50% threshold
        let text = Proposal::new(
            2,
            proposer.public_key(),
            ProposalType::TextProposal {
                description: "Test".to_string(),
            },
            "Text".to_string(),
            "Desc".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );
        assert_eq!(text.required_quorum, 20);
        assert_eq!(text.required_threshold, 50);
    }
}
