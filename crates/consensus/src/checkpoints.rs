/// Checkpoint for preventing long-range attacks
/// نقطة فحص لمنع الهجمات طويلة المدى
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    pub height: u64,
    pub hash: [u8; 32],
}

/// Mainnet checkpoints (hardcoded after blocks are mined)
/// نقاط فحص الشبكة الرئيسية (مشفرة بعد تعدين الكتل)
pub const MAINNET_CHECKPOINTS: &[Checkpoint] = &[
    // Genesis block checkpoint
    Checkpoint {
        height: 0,
        hash: [0u8; 32], // Will be replaced with actual genesis hash
    },
    // Additional checkpoints will be added as mainnet progresses
    // Every ~10,000 blocks recommended
];

/// Testnet checkpoints
/// نقاط فحص شبكة الاختبار
pub const TESTNET_CHECKPOINTS: &[Checkpoint] = &[
    Checkpoint {
        height: 0,
        hash: [0u8; 32],
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointError {
    Mismatch {
        height: u64,
        expected: [u8; 32],
        got: [u8; 32],
    },
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckpointError::Mismatch { height, expected, got } => {
                write!(
                    f,
                    "Checkpoint mismatch at height {}: expected {:x?}, got {:x?}",
                    height,
                    &expected[..4],
                    &got[..4]
                )
            }
        }
    }
}

impl std::error::Error for CheckpointError {}

/// Verify block hash matches checkpoint at given height
/// التحقق من أن تجزئة الكتلة تطابق نقطة الفحص عند الارتفاع المحدد
pub fn verify_checkpoint(
    height: u64,
    hash: &[u8; 32],
    use_testnet: bool,
) -> Result<(), CheckpointError> {
    let checkpoints = if use_testnet {
        TESTNET_CHECKPOINTS
    } else {
        MAINNET_CHECKPOINTS
    };

    for checkpoint in checkpoints {
        if checkpoint.height == height {
            if checkpoint.hash != *hash {
                return Err(CheckpointError::Mismatch {
                    height,
                    expected: checkpoint.hash,
                    got: *hash,
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_checkpoint_verification() {
        let genesis_hash = [0u8; 32];
        
        // Should pass for genesis block
        assert!(verify_checkpoint(0, &genesis_hash, false).is_ok());
        assert!(verify_checkpoint(0, &genesis_hash, true).is_ok());
    }

    #[test]
    fn test_checkpoint_mismatch() {
        let wrong_hash = [1u8; 32];
        
        // Should fail for wrong hash at checkpoint height
        let result = verify_checkpoint(0, &wrong_hash, false);
        assert!(result.is_err());
        
        match result {
            Err(CheckpointError::Mismatch { height, .. }) => {
                assert_eq!(height, 0);
            }
            _ => panic!("Expected CheckpointError::Mismatch"),
        }
    }

    #[test]
    fn test_non_checkpoint_height() {
        let any_hash = [42u8; 32];
        
        // Should pass for non-checkpoint heights (no validation)
        assert!(verify_checkpoint(5, &any_hash, false).is_ok());
        assert!(verify_checkpoint(100, &any_hash, false).is_ok());
    }
}
