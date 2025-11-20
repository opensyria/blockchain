// Governance system for on-chain proposals and voting

pub mod manager;
pub mod state;
pub mod storage;
pub mod treasury;
pub mod types;
pub mod validation;

pub use manager::{GovernanceManager, GovernanceSnapshot};
pub use state::{GovernanceError, GovernanceState, GovernanceStats};
pub use storage::{GovernanceStorage, StorageError};
pub use treasury::{Treasury, TreasuryError, TreasurySpending, TreasuryStats};
pub use types::{
    GovernanceConfig, Proposal, ProposalId, ProposalStatus, ProposalType, Vote, VoteRecord,
};
pub use validation::{ProposalValidator, ProposalValidationError};
