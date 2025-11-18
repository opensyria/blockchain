// Governance system for on-chain proposals and voting

pub mod types;
pub mod state;
pub mod manager;
pub mod storage;

pub use types::{
    Proposal, ProposalId, ProposalType, ProposalStatus, Vote, VoteRecord, GovernanceConfig,
};
pub use state::{GovernanceError, GovernanceState, GovernanceStats};
pub use manager::{GovernanceManager, GovernanceSnapshot};
pub use storage::{GovernanceStorage, StorageError};
