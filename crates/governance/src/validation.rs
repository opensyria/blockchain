//! Proposal Validation
//!
//! GOV-P2-005: Add proposal parameter validation
//!
//! Validates governance proposals to prevent:
//! - Malformed parameters (negative values, overflows)
//! - Unrealistic settings (0 block time, infinite fees)
//! - DoS attacks (extremely long descriptions, malicious values)
//! - Economic attacks (draining treasury, extreme fee changes)

use crate::types::{Proposal, ProposalType};

/// Maximum allowed lengths for text fields
const MAX_TITLE_LENGTH: usize = 200;
const MAX_DESCRIPTION_LENGTH: usize = 10_000;

/// Protocol parameter bounds
const MIN_BLOCK_TIME_SECS: u64 = 10; // 10 seconds
const MAX_BLOCK_TIME_SECS: u64 = 3600; // 1 hour
const MIN_ADJUSTMENT_INTERVAL: u32 = 10; // 10 blocks
const MAX_ADJUSTMENT_INTERVAL: u32 = 10_000; // 10,000 blocks
const MIN_TRANSACTION_FEE: u64 = 1; // 1 unit (0.000001 SYL)
const MAX_TRANSACTION_FEE: u64 = 1_000_000_000; // 1000 SYL
const MIN_BLOCK_SIZE: usize = 1_024; // 1KB
const MAX_BLOCK_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MIN_BLOCK_REWARD: u64 = 0; // Allow zero (PoS transition)
const MAX_BLOCK_REWARD: u64 = 100_000_000_000; // 100,000 SYL
const MAX_TREASURY_SPENDING: u64 = 10_000_000_000; // 10,000 SYL per proposal
const MIN_VOTING_PERIOD: u64 = 100; // At least 100 blocks (~1.6 hours)
const MAX_VOTING_PERIOD: u64 = 100_000; // At most 100k blocks (~70 days)
const MIN_EXECUTION_DELAY: u64 = 10; // At least 10 blocks
const MAX_EXECUTION_DELAY: u64 = 50_000; // At most 50k blocks (~35 days)

/// Proposal validation error
#[derive(Debug, thiserror::Error)]
pub enum ProposalValidationError {
    #[error("Title too long: {0} chars (max {MAX_TITLE_LENGTH})")]
    TitleTooLong(usize),

    #[error("Description too long: {0} chars (max {MAX_DESCRIPTION_LENGTH})")]
    DescriptionTooLong(usize),

    #[error("Title is empty")]
    EmptyTitle,

    #[error("Description is empty")]
    EmptyDescription,

    #[error("Block time {0}s out of range ({MIN_BLOCK_TIME_SECS}-{MAX_BLOCK_TIME_SECS}s)")]
    InvalidBlockTime(u64),

    #[error("Adjustment interval {0} blocks out of range ({MIN_ADJUSTMENT_INTERVAL}-{MAX_ADJUSTMENT_INTERVAL})")]
    InvalidAdjustmentInterval(u32),

    #[error("Transaction fee {0} out of range ({MIN_TRANSACTION_FEE}-{MAX_TRANSACTION_FEE})")]
    InvalidTransactionFee(u64),

    #[error("Block size {0} bytes out of range ({MIN_BLOCK_SIZE}-{MAX_BLOCK_SIZE})")]
    InvalidBlockSize(usize),

    #[error("Block reward {0} out of range ({MIN_BLOCK_REWARD}-{MAX_BLOCK_REWARD})")]
    InvalidBlockReward(u64),

    #[error("Treasury spending {0} exceeds maximum {MAX_TREASURY_SPENDING}")]
    TreasurySpendingTooHigh(u64),

    #[error("Treasury spending amount is zero")]
    ZeroTreasurySpending,

    #[error("Voting period {0} blocks out of range ({MIN_VOTING_PERIOD}-{MAX_VOTING_PERIOD})")]
    InvalidVotingPeriod(u64),

    #[error("Execution delay {0} blocks out of range ({MIN_EXECUTION_DELAY}-{MAX_EXECUTION_DELAY})")]
    InvalidExecutionDelay(u64),

    #[error("Protocol version cannot be zero")]
    ZeroProtocolVersion,

    #[error("Activation height {0} must be in the future (current: {1})")]
    ActivationHeightInPast(u64, u64),

    #[error("Activation height {0} too far in future (max {1} blocks ahead)")]
    ActivationHeightTooFar(u64, u64),
}

/// Proposal validator
pub struct ProposalValidator {
    current_height: u64,
}

impl ProposalValidator {
    /// Create new validator at current blockchain height
    pub fn new(current_height: u64) -> Self {
        Self { current_height }
    }

    /// Validate proposal before submission
    ///
    /// Checks:
    /// - Text field lengths (prevent bloat)
    /// - Parameter bounds (prevent extreme values)
    /// - Economic constraints (prevent treasury drain)
    /// - Temporal constraints (reasonable voting periods)
    pub fn validate(&self, proposal: &Proposal) -> Result<(), ProposalValidationError> {
        // Validate text fields
        self.validate_title(&proposal.title)?;
        self.validate_description(&proposal.description)?;

        // Validate voting period
        let voting_period = proposal.voting_end.saturating_sub(proposal.voting_start);
        if voting_period < MIN_VOTING_PERIOD || voting_period > MAX_VOTING_PERIOD {
            return Err(ProposalValidationError::InvalidVotingPeriod(voting_period));
        }

        // Validate execution delay
        if proposal.execution_delay < MIN_EXECUTION_DELAY
            || proposal.execution_delay > MAX_EXECUTION_DELAY
        {
            return Err(ProposalValidationError::InvalidExecutionDelay(
                proposal.execution_delay,
            ));
        }

        // Validate proposal type-specific parameters
        self.validate_proposal_type(&proposal.proposal_type)?;

        Ok(())
    }

    /// Validate proposal title
    fn validate_title(&self, title: &str) -> Result<(), ProposalValidationError> {
        if title.is_empty() {
            return Err(ProposalValidationError::EmptyTitle);
        }

        if title.len() > MAX_TITLE_LENGTH {
            return Err(ProposalValidationError::TitleTooLong(title.len()));
        }

        Ok(())
    }

    /// Validate proposal description
    fn validate_description(&self, description: &str) -> Result<(), ProposalValidationError> {
        if description.is_empty() {
            return Err(ProposalValidationError::EmptyDescription);
        }

        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(ProposalValidationError::DescriptionTooLong(
                description.len(),
            ));
        }

        Ok(())
    }

    /// Validate proposal type-specific parameters
    fn validate_proposal_type(
        &self,
        proposal_type: &ProposalType,
    ) -> Result<(), ProposalValidationError> {
        match proposal_type {
            ProposalType::DifficultyAdjustment {
                target_block_time,
                adjustment_interval,
            } => {
                if *target_block_time < MIN_BLOCK_TIME_SECS
                    || *target_block_time > MAX_BLOCK_TIME_SECS
                {
                    return Err(ProposalValidationError::InvalidBlockTime(*target_block_time));
                }

                if *adjustment_interval < MIN_ADJUSTMENT_INTERVAL
                    || *adjustment_interval > MAX_ADJUSTMENT_INTERVAL
                {
                    return Err(ProposalValidationError::InvalidAdjustmentInterval(
                        *adjustment_interval,
                    ));
                }

                Ok(())
            }

            ProposalType::MinimumFee { new_fee } => {
                if *new_fee < MIN_TRANSACTION_FEE || *new_fee > MAX_TRANSACTION_FEE {
                    return Err(ProposalValidationError::InvalidTransactionFee(*new_fee));
                }
                Ok(())
            }

            ProposalType::BlockSizeLimit { new_limit } => {
                if *new_limit < MIN_BLOCK_SIZE || *new_limit > MAX_BLOCK_SIZE {
                    return Err(ProposalValidationError::InvalidBlockSize(*new_limit));
                }
                Ok(())
            }

            ProposalType::BlockReward { new_reward } => {
                if *new_reward < MIN_BLOCK_REWARD || *new_reward > MAX_BLOCK_REWARD {
                    return Err(ProposalValidationError::InvalidBlockReward(*new_reward));
                }
                Ok(())
            }

            ProposalType::TreasurySpending {
                recipient: _,
                amount,
                description,
            } => {
                if *amount == 0 {
                    return Err(ProposalValidationError::ZeroTreasurySpending);
                }

                if *amount > MAX_TREASURY_SPENDING {
                    return Err(ProposalValidationError::TreasurySpendingTooHigh(*amount));
                }

                // Description already validated at proposal level
                // But we can add specific checks for treasury proposals
                if description.len() < 50 {
                    // Require detailed justification for spending
                    return Err(ProposalValidationError::EmptyDescription);
                }

                Ok(())
            }

            ProposalType::ProtocolUpgrade {
                version,
                activation_height,
                description: _,
            } => {
                if *version == 0 {
                    return Err(ProposalValidationError::ZeroProtocolVersion);
                }

                // Activation height must be in future
                if *activation_height <= self.current_height {
                    return Err(ProposalValidationError::ActivationHeightInPast(
                        *activation_height,
                        self.current_height,
                    ));
                }

                // But not too far in future (1 year max)
                let max_future_height = self.current_height + 525_600; // ~1 year at 1 min blocks
                if *activation_height > max_future_height {
                    return Err(ProposalValidationError::ActivationHeightTooFar(
                        *activation_height,
                        max_future_height,
                    ));
                }

                Ok(())
            }

            ProposalType::TextProposal { description: _ } => {
                // Text proposals have minimal validation (already covered by general description check)
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;

    fn create_test_proposal(proposal_type: ProposalType) -> Proposal {
        let proposer = KeyPair::generate();
        Proposal::new(
            1,
            proposer.public_key(),
            proposal_type,
            "Test Proposal".to_string(),
            "This is a test proposal with sufficient description length".to_string(),
            1000,
            1000, // voting period
            100,  // execution delay
            1_000_000,
        )
    }

    #[test]
    fn test_valid_difficulty_adjustment_proposal() {
        let validator = ProposalValidator::new(1000);
        let proposal = create_test_proposal(ProposalType::DifficultyAdjustment {
            target_block_time: 60,
            adjustment_interval: 100,
        });

        assert!(validator.validate(&proposal).is_ok());
    }

    #[test]
    fn test_invalid_block_time() {
        let validator = ProposalValidator::new(1000);
        let proposal = create_test_proposal(ProposalType::DifficultyAdjustment {
            target_block_time: 5, // Too low
            adjustment_interval: 100,
        });

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::InvalidBlockTime(5))
        ));
    }

    #[test]
    fn test_title_too_long() {
        let validator = ProposalValidator::new(1000);
        let mut proposal = create_test_proposal(ProposalType::TextProposal {
            description: "Test".to_string(),
        });
        proposal.title = "a".repeat(MAX_TITLE_LENGTH + 1);

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::TitleTooLong(_))
        ));
    }

    #[test]
    fn test_description_too_long() {
        let validator = ProposalValidator::new(1000);
        let mut proposal = create_test_proposal(ProposalType::TextProposal {
            description: "Test".to_string(),
        });
        proposal.description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::DescriptionTooLong(_))
        ));
    }

    #[test]
    fn test_empty_title() {
        let validator = ProposalValidator::new(1000);
        let mut proposal = create_test_proposal(ProposalType::TextProposal {
            description: "Test".to_string(),
        });
        proposal.title = "".to_string();

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::EmptyTitle)
        ));
    }

    #[test]
    fn test_treasury_spending_too_high() {
        let validator = ProposalValidator::new(1000);
        let proposer = KeyPair::generate();
        let recipient = KeyPair::generate();

        let proposal = create_test_proposal(ProposalType::TreasurySpending {
            recipient: recipient.public_key(),
            amount: MAX_TREASURY_SPENDING + 1,
            description: "This is a long enough description for treasury spending validation".to_string(),
        });

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::TreasurySpendingTooHigh(_))
        ));
    }

    #[test]
    fn test_protocol_upgrade_activation_in_past() {
        let current_height = 1000;
        let validator = ProposalValidator::new(current_height);

        let proposal = create_test_proposal(ProposalType::ProtocolUpgrade {
            version: 2,
            activation_height: 500, // In the past
            description: "Upgrade to v2".to_string(),
        });

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::ActivationHeightInPast(_, _))
        ));
    }

    #[test]
    fn test_valid_treasury_proposal() {
        let validator = ProposalValidator::new(1000);
        let recipient = KeyPair::generate();

        let proposal = create_test_proposal(ProposalType::TreasurySpending {
            recipient: recipient.public_key(),
            amount: 1_000_000_000, // 1000 SYL
            description: "This is a detailed justification for treasury spending with more than 50 characters".to_string(),
        });

        assert!(validator.validate(&proposal).is_ok());
    }

    #[test]
    fn test_voting_period_too_short() {
        let validator = ProposalValidator::new(1000);
        let mut proposal = create_test_proposal(ProposalType::TextProposal {
            description: "Test".to_string(),
        });
        proposal.voting_start = 1000;
        proposal.voting_end = 1050; // Only 50 blocks

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::InvalidVotingPeriod(_))
        ));
    }

    #[test]
    fn test_execution_delay_too_short() {
        let validator = ProposalValidator::new(1000);
        let mut proposal = create_test_proposal(ProposalType::TextProposal {
            description: "Test".to_string(),
        });
        proposal.execution_delay = 5; // Too short

        assert!(matches!(
            validator.validate(&proposal),
            Err(ProposalValidationError::InvalidExecutionDelay(5))
        ));
    }
}
