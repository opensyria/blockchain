use crate::types::{Proposal, ProposalId, ProposalStatus, Vote, VoteRecord};
use opensyria_core::crypto::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Error types for governance operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GovernanceError {
    ProposalNotFound(ProposalId),
    VotingNotActive,
    VotingEnded,
    AlreadyVoted,
    InsufficientStake,
    InvalidProposal,
    NotProposer,
    CannotCancel,
    NotReadyForExecution,
    ExecutionFailed(String),
}

impl std::fmt::Display for GovernanceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProposalNotFound(id) => write!(f, "Proposal {} not found", id),
            Self::VotingNotActive => write!(f, "Voting period not active"),
            Self::VotingEnded => write!(f, "Voting period has ended"),
            Self::AlreadyVoted => write!(f, "Address has already voted"),
            Self::InsufficientStake => write!(f, "Insufficient stake to create proposal"),
            Self::InvalidProposal => write!(f, "Invalid proposal"),
            Self::NotProposer => write!(f, "Only proposer can cancel"),
            Self::CannotCancel => write!(f, "Cannot cancel proposal"),
            Self::NotReadyForExecution => write!(f, "Proposal not ready for execution"),
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
        }
    }
}

impl std::error::Error for GovernanceError {}

/// In-memory governance state manager
pub struct GovernanceState {
    /// All proposals indexed by ID
    proposals: HashMap<ProposalId, Proposal>,

    /// Vote records: proposal_id -> voter -> vote_record
    votes: HashMap<ProposalId, HashMap<PublicKey, VoteRecord>>,

    /// Next proposal ID
    next_proposal_id: ProposalId,

    /// Active proposals (for quick lookup)
    active_proposals: Vec<ProposalId>,

    /// Passed proposals pending execution
    pending_execution: Vec<ProposalId>,
}

impl GovernanceState {
    /// Create new governance state
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            next_proposal_id: 1,
            active_proposals: Vec::new(),
            pending_execution: Vec::new(),
        }
    }

    /// Get next proposal ID
    pub fn next_proposal_id(&self) -> ProposalId {
        self.next_proposal_id
    }

    /// Add a new proposal
    pub fn add_proposal(&mut self, proposal: Proposal) -> ProposalId {
        let id = proposal.id;
        self.active_proposals.push(id);
        self.proposals.insert(id, proposal);
        self.votes.insert(id, HashMap::new());
        self.next_proposal_id = id + 1;
        id
    }

    /// Get proposal by ID
    pub fn get_proposal(&self, id: ProposalId) -> Option<&Proposal> {
        self.proposals.get(&id)
    }

    /// Get mutable proposal by ID
    pub fn get_proposal_mut(&mut self, id: ProposalId) -> Option<&mut Proposal> {
        self.proposals.get_mut(&id)
    }

    /// Get all proposals
    pub fn get_all_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values().collect()
    }

    /// Get active proposals
    pub fn get_active_proposals(&self) -> Vec<&Proposal> {
        self.active_proposals
            .iter()
            .filter_map(|id| self.proposals.get(id))
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    /// Get proposals by status
    pub fn get_proposals_by_status(&self, status: ProposalStatus) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.status == status)
            .collect()
    }

    /// Record a vote
    pub fn record_vote(
        &mut self,
        proposal_id: ProposalId,
        vote_record: VoteRecord,
    ) -> Result<(), GovernanceError> {
        // Check if already voted first
        if let Some(votes) = self.votes.get(&proposal_id) {
            if votes.contains_key(&vote_record.voter) {
                return Err(GovernanceError::AlreadyVoted);
            }
        }

        // Get proposal and update vote counts
        let proposal = self
            .proposals
            .get_mut(&proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        // Update vote counts
        match vote_record.vote {
            Vote::Yes => proposal.votes_yes += vote_record.voting_power,
            Vote::No => proposal.votes_no += vote_record.voting_power,
            Vote::Abstain => proposal.votes_abstain += vote_record.voting_power,
        }

        // Store vote record
        self.votes
            .entry(proposal_id)
            .or_default()
            .insert(vote_record.voter, vote_record);

        Ok(())
    }

    /// Get vote record for an address on a proposal
    pub fn get_vote(&self, proposal_id: ProposalId, voter: &PublicKey) -> Option<&VoteRecord> {
        self.votes.get(&proposal_id)?.get(voter)
    }

    /// Get all votes for a proposal
    pub fn get_proposal_votes(&self, proposal_id: ProposalId) -> Vec<&VoteRecord> {
        self.votes
            .get(&proposal_id)
            .map(|votes| votes.values().collect())
            .unwrap_or_default()
    }

    /// Finalize all proposals that have ended
    pub fn finalize_proposals(&mut self, current_height: u64) {
        let proposal_ids: Vec<ProposalId> = self.active_proposals.clone();
        let mut newly_passed = Vec::new();
        let mut to_remove = Vec::new();

        for id in proposal_ids {
            if let Some(proposal) = self.proposals.get_mut(&id) {
                if proposal.has_ended(current_height) && proposal.status == ProposalStatus::Active {
                    proposal.finalize(current_height);
                    to_remove.push(id);

                    // Track if passed for later addition to pending_execution
                    if proposal.status == ProposalStatus::Passed {
                        newly_passed.push(id);
                    }
                }
            }
        }

        // Remove from active list
        for id in to_remove {
            self.active_proposals.retain(|pid| *pid != id);
        }

        // Add to pending execution
        self.pending_execution.extend(newly_passed);
    }

    /// Get proposals ready for execution
    pub fn get_ready_for_execution(&self, current_height: u64) -> Vec<&Proposal> {
        self.pending_execution
            .iter()
            .filter_map(|id| self.proposals.get(id))
            .filter(|p| p.ready_for_execution(current_height))
            .collect()
    }

    /// Mark proposal as executed
    pub fn mark_executed(&mut self, proposal_id: ProposalId) -> Result<(), GovernanceError> {
        let proposal = self
            .get_proposal_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        proposal.mark_executed();

        // Remove from pending execution
        self.pending_execution.retain(|id| *id != proposal_id);

        Ok(())
    }

    /// Cancel a proposal
    pub fn cancel_proposal(
        &mut self,
        proposal_id: ProposalId,
        canceller: &PublicKey,
    ) -> Result<(), GovernanceError> {
        let proposal = self
            .get_proposal_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound(proposal_id))?;

        if proposal.proposer != *canceller {
            return Err(GovernanceError::NotProposer);
        }

        if proposal.status != ProposalStatus::Active {
            return Err(GovernanceError::CannotCancel);
        }

        proposal.cancel();

        // Remove from active list
        self.active_proposals.retain(|id| *id != proposal_id);

        Ok(())
    }

    /// Get total number of proposals
    pub fn total_proposals(&self) -> usize {
        self.proposals.len()
    }

    /// Get statistics
    pub fn get_statistics(&self) -> GovernanceStats {
        let mut stats = GovernanceStats {
            total_proposals: self.proposals.len(),
            active_proposals: 0,
            passed_proposals: 0,
            rejected_proposals: 0,
            executed_proposals: 0,
            cancelled_proposals: 0,
            total_votes_cast: 0,
        };

        for proposal in self.proposals.values() {
            match proposal.status {
                ProposalStatus::Active => stats.active_proposals += 1,
                ProposalStatus::Passed => stats.passed_proposals += 1,
                ProposalStatus::Rejected => stats.rejected_proposals += 1,
                ProposalStatus::Executed => stats.executed_proposals += 1,
                ProposalStatus::Cancelled => stats.cancelled_proposals += 1,
            }
        }

        for votes in self.votes.values() {
            stats.total_votes_cast += votes.len();
        }

        stats
    }
}

impl Default for GovernanceState {
    fn default() -> Self {
        Self::new()
    }
}

/// Governance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStats {
    pub total_proposals: usize,
    pub active_proposals: usize,
    pub passed_proposals: usize,
    pub rejected_proposals: usize,
    pub executed_proposals: usize,
    pub cancelled_proposals: usize,
    pub total_votes_cast: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProposalType;
    use opensyria_core::crypto::KeyPair;

    #[test]
    fn test_add_proposal() {
        let mut state = GovernanceState::new();
        let proposer = KeyPair::generate();

        let proposal = Proposal::new(
            state.next_proposal_id(),
            proposer.public_key(),
            ProposalType::TextProposal {
                description: "Test".to_string(),
            },
            "Test Proposal".to_string(),
            "Description".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );

        let id = state.add_proposal(proposal);
        assert_eq!(id, 1);
        assert_eq!(state.next_proposal_id(), 2);
        assert!(state.get_proposal(id).is_some());
    }

    #[test]
    fn test_record_vote() {
        let mut state = GovernanceState::new();
        let proposer = KeyPair::generate();
        let voter = KeyPair::generate();

        let proposal = Proposal::new(
            state.next_proposal_id(),
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

        let id = state.add_proposal(proposal);

        let vote_record = VoteRecord {
            voter: voter.public_key(),
            vote: Vote::Yes,
            voting_power: 100_000,
            timestamp: 150,
        };

        state.record_vote(id, vote_record).unwrap();

        let retrieved_vote = state.get_vote(id, &voter.public_key());
        assert!(retrieved_vote.is_some());
        assert_eq!(retrieved_vote.unwrap().vote, Vote::Yes);

        let proposal = state.get_proposal(id).unwrap();
        assert_eq!(proposal.votes_yes, 100_000);
    }

    #[test]
    fn test_cannot_vote_twice() {
        let mut state = GovernanceState::new();
        let proposer = KeyPair::generate();
        let voter = KeyPair::generate();

        let proposal = Proposal::new(
            state.next_proposal_id(),
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

        let id = state.add_proposal(proposal);

        let vote1 = VoteRecord {
            voter: voter.public_key(),
            vote: Vote::Yes,
            voting_power: 100_000,
            timestamp: 150,
        };

        state.record_vote(id, vote1).unwrap();

        let vote2 = VoteRecord {
            voter: voter.public_key(),
            vote: Vote::No,
            voting_power: 100_000,
            timestamp: 160,
        };

        let result = state.record_vote(id, vote2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GovernanceError::AlreadyVoted));
    }

    #[test]
    fn test_finalize_proposals() {
        let mut state = GovernanceState::new();
        let proposer = KeyPair::generate();

        let proposal = Proposal::new(
            state.next_proposal_id(),
            proposer.public_key(),
            ProposalType::MinimumFee { new_fee: 200 },
            "Test".to_string(),
            "Desc".to_string(),
            100,
            1000,
            100,
            1_000_000,
        );

        let id = state.add_proposal(proposal);

        // Add votes
        let voter1 = KeyPair::generate();
        state
            .record_vote(
                id,
                VoteRecord {
                    voter: voter1.public_key(),
                    vote: Vote::Yes,
                    voting_power: 350_000,
                    timestamp: 150,
                },
            )
            .unwrap();

        let voter2 = KeyPair::generate();
        state
            .record_vote(
                id,
                VoteRecord {
                    voter: voter2.public_key(),
                    vote: Vote::No,
                    voting_power: 50_000,
                    timestamp: 160,
                },
            )
            .unwrap();

        // Finalize after voting ends
        state.finalize_proposals(1100);

        let proposal = state.get_proposal(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Passed);
        assert!(state.active_proposals.is_empty());
        assert_eq!(state.pending_execution.len(), 1);
    }

    #[test]
    fn test_cancel_proposal() {
        let mut state = GovernanceState::new();
        let proposer = KeyPair::generate();
        let other = KeyPair::generate();

        let proposal = Proposal::new(
            state.next_proposal_id(),
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

        let id = state.add_proposal(proposal);

        // Other user cannot cancel
        let result = state.cancel_proposal(id, &other.public_key());
        assert!(result.is_err());

        // Proposer can cancel
        state.cancel_proposal(id, &proposer.public_key()).unwrap();

        let proposal = state.get_proposal(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Cancelled);
    }

    #[test]
    fn test_governance_statistics() {
        let mut state = GovernanceState::new();
        let proposer = KeyPair::generate();

        // Add multiple proposals
        for i in 0..5 {
            let proposal = Proposal::new(
                state.next_proposal_id(),
                proposer.public_key(),
                ProposalType::TextProposal {
                    description: format!("Test {}", i),
                },
                format!("Proposal {}", i),
                "Desc".to_string(),
                100 + i,
                1000,
                100,
                1_000_000,
            );
            state.add_proposal(proposal);
        }

        let stats = state.get_statistics();
        assert_eq!(stats.total_proposals, 5);
        assert_eq!(stats.active_proposals, 5);
    }
}
