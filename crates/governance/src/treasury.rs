use opensyria_core::crypto::PublicKey;
use serde::{Deserialize, Serialize};

/// Treasury management for governance spending proposals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treasury {
    /// Current treasury balance
    balance: u64,

    /// Transaction fee portion that goes to treasury (percentage)
    fee_percentage: u8,

    /// Total collected from fees
    total_collected: u64,

    /// Total spent via proposals
    total_spent: u64,

    /// Spending history
    spending_history: Vec<TreasurySpending>,
}

/// Record of a treasury spending transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasurySpending {
    pub proposal_id: u64,
    pub recipient: PublicKey,
    pub amount: u64,
    pub description: String,
    pub executed_at: u64, // Block height
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreasuryError {
    InsufficientFunds { requested: u64, available: u64 },
    InvalidFeePercentage,
    InvalidAmount,
}

impl std::fmt::Display for TreasuryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsufficientFunds { requested, available } => {
                write!(
                    f,
                    "Insufficient treasury funds: requested {} but only {} available",
                    requested, available
                )
            }
            Self::InvalidFeePercentage => write!(f, "Fee percentage must be 0-100"),
            Self::InvalidAmount => write!(f, "Amount must be greater than zero"),
        }
    }
}

impl std::error::Error for TreasuryError {}

impl Treasury {
    /// Create a new treasury with initial configuration
    pub fn new(fee_percentage: u8) -> Result<Self, TreasuryError> {
        if fee_percentage > 100 {
            return Err(TreasuryError::InvalidFeePercentage);
        }

        Ok(Self {
            balance: 0,
            fee_percentage,
            total_collected: 0,
            total_spent: 0,
            spending_history: Vec::new(),
        })
    }

    /// Add transaction fees to treasury
    pub fn add_fees(&mut self, total_fees: u64) {
        let treasury_portion = (total_fees as u128 * self.fee_percentage as u128 / 100) as u64;
        self.balance += treasury_portion;
        self.total_collected += treasury_portion;
    }

    /// Execute a treasury spending proposal
    pub fn spend(
        &mut self,
        proposal_id: u64,
        recipient: PublicKey,
        amount: u64,
        description: String,
        block_height: u64,
    ) -> Result<(), TreasuryError> {
        if amount == 0 {
            return Err(TreasuryError::InvalidAmount);
        }

        if amount > self.balance {
            return Err(TreasuryError::InsufficientFunds {
                requested: amount,
                available: self.balance,
            });
        }

        self.balance -= amount;
        self.total_spent += amount;

        self.spending_history.push(TreasurySpending {
            proposal_id,
            recipient,
            amount,
            description,
            executed_at: block_height,
        });

        Ok(())
    }

    /// Get current balance
    pub fn balance(&self) -> u64 {
        self.balance
    }

    /// Get fee percentage
    pub fn fee_percentage(&self) -> u8 {
        self.fee_percentage
    }

    /// Update fee percentage (via governance)
    pub fn set_fee_percentage(&mut self, new_percentage: u8) -> Result<(), TreasuryError> {
        if new_percentage > 100 {
            return Err(TreasuryError::InvalidFeePercentage);
        }
        self.fee_percentage = new_percentage;
        Ok(())
    }

    /// Get spending history
    pub fn spending_history(&self) -> &[TreasurySpending] {
        &self.spending_history
    }

    /// Get statistics
    pub fn statistics(&self) -> TreasuryStats {
        TreasuryStats {
            current_balance: self.balance,
            total_collected: self.total_collected,
            total_spent: self.total_spent,
            fee_percentage: self.fee_percentage,
            spending_count: self.spending_history.len(),
        }
    }
}

impl Default for Treasury {
    fn default() -> Self {
        Self::new(10).unwrap() // Default 10% of fees to treasury
    }
}

/// Treasury statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryStats {
    pub current_balance: u64,
    pub total_collected: u64,
    pub total_spent: u64,
    pub fee_percentage: u8,
    pub spending_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_treasury_creation() {
        let treasury = Treasury::new(10).unwrap();
        assert_eq!(treasury.balance(), 0);
        assert_eq!(treasury.fee_percentage(), 10);
    }

    #[test]
    fn test_treasury_invalid_fee_percentage() {
        assert!(Treasury::new(101).is_err());
    }

    #[test]
    fn test_add_fees() {
        let mut treasury = Treasury::new(10).unwrap();
        treasury.add_fees(1000);
        assert_eq!(treasury.balance(), 100); // 10% of 1000
        assert_eq!(treasury.statistics().total_collected, 100);
    }

    #[test]
    fn test_spending() {
        let mut treasury = Treasury::new(10).unwrap();
        treasury.add_fees(10000); // Adds 1000 to treasury

        let recipient = PublicKey([1u8; 32]);
        treasury
            .spend(1, recipient, 500, "Test spending".to_string(), 100)
            .unwrap();

        assert_eq!(treasury.balance(), 500);
        assert_eq!(treasury.statistics().total_spent, 500);
        assert_eq!(treasury.spending_history().len(), 1);
    }

    #[test]
    fn test_insufficient_funds() {
        let mut treasury = Treasury::new(10).unwrap();
        treasury.add_fees(1000); // Adds 100 to treasury

        let recipient = PublicKey([1u8; 32]);
        let result = treasury.spend(1, recipient, 200, "Too much".to_string(), 100);

        assert!(result.is_err());
        match result.unwrap_err() {
            TreasuryError::InsufficientFunds { requested, available } => {
                assert_eq!(requested, 200);
                assert_eq!(available, 100);
            }
            _ => panic!("Expected InsufficientFunds error"),
        }
    }

    #[test]
    fn test_update_fee_percentage() {
        let mut treasury = Treasury::new(10).unwrap();
        treasury.set_fee_percentage(20).unwrap();
        assert_eq!(treasury.fee_percentage(), 20);

        assert!(treasury.set_fee_percentage(101).is_err());
    }
}
