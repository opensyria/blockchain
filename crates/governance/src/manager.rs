use crate::state::{GovernanceError, GovernanceState, GovernanceStats};
use crate::types::{
    GovernanceConfig, Proposal, ProposalId, ProposalStatus, ProposalType, Vote, VoteRecord,
};
use opensyria_core::crypto::PublicKey;
use opensyria_storage::StateStorage;
use serde::{Deserialize, Serialize};

/// Main governance manager
pub struct GovernanceManager {
    state: GovernanceState,
    config: GovernanceConfig,
}

impl GovernanceManager {
    /// Create new governance manager
    pub fn new(config: GovernanceConfig) -> Self {
        Self {
            state: GovernanceState::new(),
            config,
        }
    }

    /// Create a new proposal
    #[allow(clippy::too_many_arguments)]
    pub fn create_proposal(
        &mut self,
        proposer: PublicKey,
        proposer_stake: u64,
        proposal_type: ProposalType,
        title: String,
        description: String,
        current_height: u64,
        total_voting_power: u64,
        state_storage: &StateStorage,
    ) -> Result<ProposalId, GovernanceError> {
        // Check if governance is enabled
        if !self.config.enabled {
            return Err(GovernanceError::InvalidProposal);
        }

        // Check minimum stake requirement
        if proposer_stake < self.config.min_proposal_stake {
            return Err(GovernanceError::InsufficientStake);
        }

        // Validate proposal
        if title.is_empty() || description.is_empty() {
            return Err(GovernanceError::InvalidProposal);
        }

        // Validate proposal parameters
        proposal_type
            .validate()
            .map_err(|_| GovernanceError::InvalidProposal)?;

        let proposal = Proposal::new(
            self.state.next_proposal_id(),
            proposer,
            proposal_type,
            title,
            description,
            current_height,
            self.config.default_voting_period,
            self.config.default_execution_delay,
            total_voting_power,
        );

        let id = self.state.add_proposal(proposal);
        
        // SECURITY: Snapshot all account balances at proposal creation time
        // This prevents flash loan attacks where attackers borrow tokens,
        // vote with inflated balance, then return tokens in same block
        self.state.snapshot_balances(id, state_storage)?;
        
        Ok(id)
    }

    /// Cast a vote on a proposal
    /// 
    /// ✅  SECURITY FIX (CRITICAL-006): Now uses async atomic vote recording
    /// This prevents double-voting race conditions by ensuring check-and-insert
    /// operations are serialized per proposal using mutex locks.
    pub async fn vote(
        &mut self,
        proposal_id: ProposalId,
        voter: PublicKey,
        vote: Vote,
        _state_storage: &StateStorage,
        current_height: u64,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .state
            .get_proposal(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        // Check if voting is active
        if !proposal.is_active(current_height) {
            if current_height < proposal.voting_start {
                return Err(GovernanceError::VotingNotActive);
            } else {
                return Err(GovernanceError::VotingEnded);
            }
        }

        // SECURITY FIX: Only allow voting with pre-snapshotted balances to prevent flash-loan attacks
        // Snapshot must be taken at proposal creation time, not at vote time
        let snapshot_balance = self.state.get_snapshot_balance(proposal_id, &voter)
            .ok_or(GovernanceError::NotEligibleToVote)?;

        let vote_record = VoteRecord {
            voter,
            vote,
            voting_power: snapshot_balance,
            snapshot_balance,
            timestamp: current_height,
            delegated_from: None, // Direct vote, not delegated
        };

        // SECURITY: Use atomic vote recording to prevent double-voting race
        self.state.record_vote(proposal_id, vote_record).await?;
        Ok(())
    }
    
    /// Cast a vote synchronously (blocking wrapper for non-async contexts)
    /// 
    /// ⚠️  WARNING: This blocks the current thread. Prefer using vote() in async contexts.
    pub fn vote_blocking(
        &mut self,
        proposal_id: ProposalId,
        voter: PublicKey,
        vote: Vote,
        state_storage: &StateStorage,
        current_height: u64,
    ) -> Result<(), GovernanceError> {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(self.vote(proposal_id, voter, vote, state_storage, current_height))
    }

    /// Process proposals at current block height (finalize ended proposals)
    pub fn process_proposals(&mut self, current_height: u64) {
        self.state.finalize_proposals(current_height);
    }

    /// Get proposals ready for execution
    pub fn get_ready_for_execution(&self, current_height: u64) -> Vec<&Proposal> {
        self.state.get_ready_for_execution(current_height)
    }

    /// Execute a proposal (mark as executed, actual execution happens externally)
    /// This method requires the caller to verify execution occurred
    pub fn mark_proposal_executed(
        &mut self,
        proposal_id: ProposalId,
        current_height: u64,
    ) -> Result<(), GovernanceError> {
        // Get proposal to verify it's ready for execution
        let proposal = self
            .state
            .get_proposal(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        // Verify proposal is in passed state
        if proposal.status != ProposalStatus::Passed {
            return Err(GovernanceError::NotReadyForExecution);
        }

        // Verify execution delay has passed
        if !proposal.ready_for_execution(current_height) {
            return Err(GovernanceError::NotReadyForExecution);
        }

        self.state.mark_executed(proposal_id)
    }

    /// Cancel a proposal
    pub fn cancel_proposal(
        &mut self,
        proposal_id: ProposalId,
        canceller: &PublicKey,
    ) -> Result<(), GovernanceError> {
        self.state.cancel_proposal(proposal_id, canceller)
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, proposal_id: ProposalId) -> Option<&Proposal> {
        self.state.get_proposal(proposal_id)
    }

    /// Get all proposals
    pub fn get_all_proposals(&self) -> Vec<&Proposal> {
        self.state.get_all_proposals()
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.state.get_active_proposals()
    }

    /// Get proposals by status
    pub fn get_proposals_by_status(&self, status: ProposalStatus) -> Vec<&Proposal> {
        self.state.get_proposals_by_status(status)
    }

    /// Get vote record
    pub fn get_vote(&self, proposal_id: ProposalId, voter: &PublicKey) -> Option<&VoteRecord> {
        self.state.get_vote(proposal_id, voter)
    }

    /// Get all votes for a proposal
    pub fn get_proposal_votes(&self, proposal_id: ProposalId) -> Vec<&VoteRecord> {
        self.state.get_proposal_votes(proposal_id)
    }

    /// Get governance statistics
    pub fn get_statistics(&self) -> GovernanceStats {
        self.state.get_statistics()
    }

    /// Get configuration
    pub fn config(&self) -> &GovernanceConfig {
        &self.config
    }

    /// Update configuration (typically via executed proposal)
    pub fn update_config(&mut self, new_config: GovernanceConfig) {
        self.config = new_config;
    }
}

/// Serializable governance snapshot for storage
#[derive(Debug, Clone, Serialize, Deserialize, bincode::Encode, bincode::Decode)]
pub struct GovernanceSnapshot {
    pub proposals: Vec<Proposal>,
    pub votes: Vec<(ProposalId, PublicKey, VoteRecord)>,
    pub balance_snapshots: Vec<(ProposalId, PublicKey, u64)>, // Store snapshots for each proposal
    pub next_proposal_id: ProposalId,
    pub config: GovernanceConfig,
}

impl GovernanceManager {
    /// Create snapshot for persistence
    pub fn create_snapshot(&self) -> GovernanceSnapshot {
        let mut votes = Vec::new();
        for proposal in self.state.get_all_proposals() {
            for vote_record in self.state.get_proposal_votes(proposal.id) {
                votes.push((proposal.id, vote_record.voter, vote_record.clone()));
            }
        }

        GovernanceSnapshot {
            proposals: self
                .state
                .get_all_proposals()
                .into_iter()
                .cloned()
                .collect(),
            votes,
            balance_snapshots: self.state.get_all_snapshots(),
            next_proposal_id: self.state.next_proposal_id(),
            config: self.config.clone(),
        }
    }

    /// Restore from snapshot
    pub fn from_snapshot(snapshot: GovernanceSnapshot) -> Self {
        let mut manager = Self::new(snapshot.config);

        // Restore proposals
        for proposal in snapshot.proposals {
            manager.state.add_proposal(proposal);
        }

        // Restore votes (use blocking version since this is initialization)
        for (proposal_id, _voter, vote_record) in snapshot.votes {
            let _ = manager.state.record_vote_blocking(proposal_id, vote_record);
        }

        // Restore balance snapshots
        for (proposal_id, address, balance) in snapshot.balance_snapshots {
            manager.state.store_snapshot(proposal_id, &address, balance);
        }

        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;
    use opensyria_storage::StateStorage;

    // Helper to create a test StateStorage
    fn create_test_state() -> StateStorage {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("test_gov_{}", nanos));
        StateStorage::open(temp_dir).unwrap()
    }

    #[test]
    fn test_create_proposal() {
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let result = manager.create_proposal(
            proposer.public_key(),
            2_000_000_000, // Enough stake
            ProposalType::TextProposal {
                description: "Test proposal".to_string(),
            },
            "Test Proposal".to_string(),
            "This is a test".to_string(),
            100,
            10_000_000_000,
            &state,
        );

        assert!(result.is_ok());
        let id = result.unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn test_insufficient_stake() {
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let result = manager.create_proposal(
            proposer.public_key(),
            100_000, // Not enough stake
            ProposalType::TextProposal {
                description: "Test".to_string(),
            },
            "Test".to_string(),
            "Desc".to_string(),
            100,
            10_000_000_000,
            &state,
        );

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GovernanceError::InsufficientStake
        ));
    }

    #[test]
    fn test_voting() {
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let voter = KeyPair::generate();
        
        // Set voter balance for voting power
        state.set_balance(&voter.public_key(), 500_000).unwrap();

        let proposal_id = manager
            .create_proposal(
                proposer.public_key(),
                2_000_000_000,
                ProposalType::TextProposal {
                    description: "Test".to_string(),
                },
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10_000_000_000,
                &state,
            )
            .unwrap();

        // Vote during active period
        let result = manager.vote(proposal_id, voter.public_key(), Vote::Yes, &state, 150);
        assert!(result.is_ok());

        // Check vote was recorded
        let vote = manager.get_vote(proposal_id, &voter.public_key());
        assert!(vote.is_some());
        assert_eq!(vote.unwrap().vote, Vote::Yes);
        assert_eq!(vote.unwrap().voting_power, 500_000); // Validated power
    }

    #[test]
    fn test_voting_before_start() {
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let voter = KeyPair::generate();
        state.set_balance(&voter.public_key(), 500_000).unwrap();

        let proposal_id = manager
            .create_proposal(
                proposer.public_key(),
                2_000_000_000,
                ProposalType::TextProposal {
                    description: "Test".to_string(),
                },
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10_000_000_000,
                &state,
            )
            .unwrap();

        // Try to vote before voting starts (at height 50, voting starts at 100)
        let result = manager.vote(proposal_id, voter.public_key(), Vote::Yes, &state, 50);
        assert!(result.is_err());
    }

    #[test]
    fn test_voting_after_end() {
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let voter = KeyPair::generate();
        state.set_balance(&voter.public_key(), 500_000).unwrap();

        let proposal_id = manager
            .create_proposal(
                proposer.public_key(),
                2_000_000_000,
                ProposalType::TextProposal {
                    description: "Test".to_string(),
                },
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10_000_000_000,
                &state,
            )
            .unwrap();

        // Try to vote after voting ends (voting ends at 100 + 10080 = 10180)
        let result = manager.vote(proposal_id, voter.public_key(), Vote::Yes, &state, 20000);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::VotingEnded));
    }

    #[test]
    fn test_proposal_finalization() {
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let total_power = 10_000_000_000;

        let proposal_id = manager
            .create_proposal(
                proposer.public_key(),
                2_000_000_000,
                ProposalType::MinimumFee { new_fee: 5000 },
                "Increase Fee".to_string(),
                "Increase the minimum fee".to_string(),
                100,
                total_power,
                &state,
            )
            .unwrap();

        // Cast votes (need 30% quorum, 60% threshold)
        for i in 0..4 {
            let voter = KeyPair::generate();
            let voter_power = total_power / 10; // 10% each
            state.set_balance(&voter.public_key(), voter_power).unwrap();
            manager
                .vote(
                    proposal_id,
                    voter.public_key(),
                    Vote::Yes,
                    &state,
                    150 + i,
                )
                .unwrap();
        }

        // Process proposals after voting ends
        manager.process_proposals(100 + 10080 + 1);

        let proposal = manager.get_proposal(proposal_id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);
    }

    #[test]
    fn test_snapshot_and_restore() {
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let proposal_id = manager
            .create_proposal(
                proposer.public_key(),
                2_000_000_000,
                ProposalType::TextProposal {
                    description: "Test".to_string(),
                },
                "Test".to_string(),
                "Desc".to_string(),
                100,
                10_000_000_000,
                &state,
            )
            .unwrap();

        let voter = KeyPair::generate();
        state.set_balance(&voter.public_key(), 500_000).unwrap();
        manager
            .vote(proposal_id, voter.public_key(), Vote::Yes, &state, 150)
            .unwrap();

        // Create snapshot
        let snapshot = manager.create_snapshot();

        // Restore from snapshot
        let restored = GovernanceManager::from_snapshot(snapshot);

        assert_eq!(
            restored.get_statistics().total_proposals,
            manager.get_statistics().total_proposals
        );
        assert!(restored.get_proposal(proposal_id).is_some());
        assert!(restored
            .get_vote(proposal_id, &voter.public_key())
            .is_some());
    }

    #[test]
    fn test_flash_loan_attack_prevented() {
        // SECURITY TEST: Verify that flash loan attacks are prevented by balance snapshots
        // 
        // Attack scenario:
        // 1. Attacker has 100 tokens initially
        // 2. Proposal is created (snapshot taken: attacker=100, victim=900)
        // 3. Attacker borrows 10,000 tokens via flash loan (balance now 10,100)
        // 4. Attacker votes with 10,100 voting power (SHOULD FAIL - only 100 allowed)
        // 5. Attacker returns flash loan (balance back to 100)
        //
        // This test verifies that voting power is determined by snapshot balance,
        // not current balance, preventing flash loan manipulation.
        
        let config = GovernanceConfig::default();
        let mut manager = GovernanceManager::new(config);
        let state = create_test_state();

        let proposer = KeyPair::generate();
        let attacker = KeyPair::generate();
        let victim = KeyPair::generate();
        
        // Initial state: attacker has 100, victim has 900
        state.set_balance(&attacker.public_key(), 100).unwrap();
        state.set_balance(&victim.public_key(), 900).unwrap();
        
        // Create proposal - this takes a snapshot of all balances
        let proposal_id = manager
            .create_proposal(
                proposer.public_key(),
                2_000_000_000,
                ProposalType::TextProposal {
                    description: "Test flash loan attack".to_string(),
                },
                "Flash Loan Test".to_string(),
                "Testing flash loan prevention".to_string(),
                100,
                10_000_000_000,
                &state,
            )
            .unwrap();
        
        // Verify snapshots were created correctly
        let attacker_snapshot = manager.state.get_snapshot_balance(proposal_id, &attacker.public_key());
        let victim_snapshot = manager.state.get_snapshot_balance(proposal_id, &victim.public_key());
        assert_eq!(attacker_snapshot, Some(100), "Attacker snapshot should be 100");
        assert_eq!(victim_snapshot, Some(900), "Victim snapshot should be 900");
        
        // ATTACK: Simulate flash loan - attacker borrows 10,000 tokens
        state.set_balance(&attacker.public_key(), 10_100).unwrap();
        
        // Verify current balance is inflated
        let current_balance = state.get_balance(&attacker.public_key()).unwrap();
        assert_eq!(current_balance, 10_100, "Current balance should be inflated by flash loan");
        
        // Attacker tries to vote with inflated balance
        let vote_result = manager.vote(
            proposal_id, 
            attacker.public_key(), 
            Vote::Yes, 
            &state, 
            150
        );
        
        // Vote should succeed (can vote with any balance)
        assert!(vote_result.is_ok(), "Vote should be allowed");
        
        // CRITICAL CHECK: Voting power should be based on SNAPSHOT, not current balance
        let attacker_voting_power = manager.get_vote(proposal_id, &attacker.public_key()).unwrap().voting_power;
        assert_eq!(
            attacker_voting_power, 
            100, 
            "Voting power MUST be snapshot balance (100), not current balance (10,100)"
        );
        
        // Victim votes with legitimate balance
        let victim_vote = manager.vote(
            proposal_id,
            victim.public_key(),
            Vote::No,
            &state,
            151,
        );
        assert!(victim_vote.is_ok());
        
        let victim_voting_power = manager.get_vote(proposal_id, &victim.public_key()).unwrap().voting_power;
        assert_eq!(victim_voting_power, 900, "Victim voting power should be snapshot balance");
        
        // Simulate flash loan repayment - attacker returns tokens
        state.set_balance(&attacker.public_key(), 100).unwrap();
        
        // Verify attack was prevented: victim has 9x voting power of attacker
        assert_eq!(
            victim_voting_power / attacker_voting_power,
            9,
            "Victim should have 9x voting power (900/100), preventing governance takeover"
        );
        
        println!("✓ Flash loan attack prevented:");
        println!("  - Attacker snapshot balance: 100");
        println!("  - Attacker current balance (during attack): 10,100");
        println!("  - Attacker voting power: {} (snapshot enforced)", attacker_voting_power);
        println!("  - Attack prevented: voting power locked to snapshot");
    }
}
