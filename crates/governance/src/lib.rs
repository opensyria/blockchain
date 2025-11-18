// Governance system for on-chain proposals and voting

pub mod manager;
pub mod state;
pub mod storage;
pub mod types;

pub use manager::{GovernanceManager, GovernanceSnapshot};
pub use state::{GovernanceError, GovernanceState, GovernanceStats};
pub use storage::{GovernanceStorage, StorageError};
pub use types::{
    GovernanceConfig, Proposal, ProposalId, ProposalStatus, ProposalType, Vote, VoteRecord,
};
