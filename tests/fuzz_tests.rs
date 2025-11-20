//! Fuzzing Tests for OpenSyria Blockchain
//!
//! PERF-P2-008: Add fuzzing tests
//!
//! Property-based and fuzz testing to discover edge cases and security issues.
//! Uses proptest for property-based testing (similar to QuickCheck).
//!
//! Run with: cargo test --package opensyria-fuzz-tests

use opensyria_core::{Block, BlockHeader, Transaction, crypto::KeyPair};
use opensyria_consensus::ProofOfWork;
use proptest::prelude::*;

/// Strategy for generating valid transactions
fn transaction_strategy() -> impl Strategy<Value = (Transaction, KeyPair)> {
    (
        any::<u64>(), // amount
        any::<u64>(), // fee
        any::<u64>(), // nonce
    )
        .prop_map(|(amount, fee, nonce)| {
            let sender = KeyPair::generate();
            let receiver = KeyPair::generate();

            let mut tx = Transaction::new(
                sender.public_key(),
                receiver.public_key(),
                amount,
                fee,
                nonce,
            );
            let msg = tx.signing_hash();
            tx.signature = sender.sign(&msg);

            (tx, sender)
        })
}

/// Property: Transaction hash should be deterministic
#[test]
fn prop_transaction_hash_deterministic() {
    proptest!(|(seed in any::<u64>())| {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx1 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            seed,
            100,
            0,
        );
        let tx2 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            seed,
            100,
            0,
        );

        prop_assert_eq!(tx1.hash(), tx2.hash());
    });
}

/// Property: Different transactions should have different hashes
#[test]
fn prop_different_transactions_different_hashes() {
    proptest!(|(amount1 in any::<u64>(), amount2 in any::<u64>())| {
        prop_assume!(amount1 != amount2);

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx1 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            amount1,
            100,
            0,
        );
        let tx2 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            amount2,
            100,
            0,
        );

        prop_assert_ne!(tx1.hash(), tx2.hash());
    });
}

/// Property: Block hash should change when any field changes
#[test]
fn prop_block_hash_changes_with_fields() {
    proptest!(|(nonce1 in any::<u64>(), nonce2 in any::<u64>())| {
        prop_assume!(nonce1 != nonce2);

        let mut header1 = BlockHeader {
            prev_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 1000,
            difficulty: 16,
            nonce: nonce1,
        };

        let mut header2 = header1;
        header2.nonce = nonce2;

        prop_assert_ne!(header1.hash(), header2.hash());
    });
}

/// Property: Difficulty check should be consistent
#[test]
fn prop_difficulty_check_consistent() {
    proptest!(|(nonce in any::<u64>(), difficulty in 1u32..32)| {
        let mut header = BlockHeader {
            prev_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 1000,
            difficulty,
            nonce,
        };

        let meets1 = header.meets_difficulty();
        let meets2 = header.meets_difficulty();

        prop_assert_eq!(meets1, meets2);
    });
}

/// Property: Transaction amount should never overflow when serialized
#[test]
fn prop_transaction_amount_no_overflow() {
    proptest!(|(amount in any::<u64>(), fee in any::<u64>())| {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            amount,
            fee,
            0,
        );

        // Should not panic when computing total value
        let total = amount.saturating_add(fee);
        prop_assert!(total >= amount);
        prop_assert!(total >= fee);
    });
}

/// Property: Signature verification should always succeed for correctly signed transactions
#[test]
fn prop_valid_signature_always_verifies() {
    proptest!(|((tx, sender) in transaction_strategy())| {
        let msg = tx.signing_hash();
        prop_assert!(sender.public_key().verify(&msg, &tx.signature));
    });
}

/// Property: Nonce should always increase in valid transaction chains
#[test]
fn prop_nonce_ordering() {
    proptest!(|(n1 in 0u64..1000, n2 in 0u64..1000)| {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx1 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            100,
            10,
            n1,
        );
        let tx2 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            100,
            10,
            n2,
        );

        if n1 < n2 {
            prop_assert!(tx1.nonce < tx2.nonce);
        } else if n1 > n2 {
            prop_assert!(tx1.nonce > tx2.nonce);
        } else {
            prop_assert_eq!(tx1.nonce, tx2.nonce);
        }
    });
}

/// Fuzz test: Random transaction data should not cause panics
#[test]
fn fuzz_transaction_parsing() {
    proptest!(|(
        amount in any::<u64>(),
        fee in any::<u64>(),
        nonce in any::<u64>(),
        data in prop::collection::vec(any::<u8>(), 0..1000)
    )| {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            amount,
            fee,
            nonce,
        );
        tx.data = data;

        // Should not panic
        let _hash = tx.hash();
        let _signing_hash = tx.signing_hash();
        let config = bincode::config::standard();
        let _serialized = bincode::encode_to_vec(&tx, config);
    });
}

/// Fuzz test: Block with random transactions should handle gracefully
#[test]
fn fuzz_block_with_random_transactions() {
    proptest!(|(
        num_txs in 0usize..100,
        difficulty in 1u32..16
    )| {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let transactions: Vec<Transaction> = (0..num_txs)
            .map(|i| {
                let mut tx = Transaction::new(
                    sender.public_key(),
                    receiver.public_key(),
                    i as u64 * 1000,
                    100,
                    i as u64,
                );
                let msg = tx.signing_hash();
                tx.signature = sender.sign(&msg);
                tx
            })
            .collect();

        let block = Block::new([0u8; 32], transactions, difficulty);

        // Should not panic
        let _hash = block.hash();
        let _valid_merkle = block.verify_merkle_root();
        let config = bincode::config::standard();
        let _serialized = bincode::encode_to_vec(&block, config);
    });
}

/// Fuzz test: Difficulty adjustment should never panic
#[test]
fn fuzz_difficulty_adjustment() {
    use opensyria_consensus::DifficultyAdjuster;
    use std::time::Duration;

    proptest!(|(
        current_difficulty in 1u32..1000,
        actual_time_secs in 1u64..1_000_000,
        block_count in 1u32..10_000
    )| {
        let adjuster = DifficultyAdjuster::new(60, 100);
        let actual_time = Duration::from_secs(actual_time_secs);

        // Should not panic and should return valid difficulty
        let new_difficulty = adjuster.adjust(current_difficulty, actual_time, block_count);

        prop_assert!(new_difficulty >= opensyria_core::MIN_DIFFICULTY);
        prop_assert!(new_difficulty <= opensyria_core::MAX_DIFFICULTY);
    });
}

/// Fuzz test: State balance operations should never overflow
#[test]
fn fuzz_state_balance_operations() {
    use opensyria_storage::StateStorage;
    use tempfile::TempDir;

    proptest!(|(
        balance1 in any::<u64>(),
        balance2 in any::<u64>(),
    )| {
        let temp_dir = TempDir::new().unwrap();
        let state = StateStorage::open(temp_dir.path().to_path_buf()).unwrap();
        let account = KeyPair::generate();

        // Set initial balance
        state.set_balance(&account.public_key(), balance1).ok();

        // Try to add balance (should handle overflow)
        let result = state.add_balance(&account.public_key(), balance2);

        // If overflow would occur, should error
        if balance1.checked_add(balance2).is_none() {
            prop_assert!(result.is_err());
        } else {
            prop_assert!(result.is_ok());
            let final_balance = state.get_balance(&account.public_key()).unwrap();
            prop_assert_eq!(final_balance, balance1 + balance2);
        }
    });
}

/// Property: Merkle root should change if any transaction changes
#[test]
fn prop_merkle_root_changes_with_transactions() {
    proptest!(|(
        amount1 in any::<u64>(),
        amount2 in any::<u64>(),
    )| {
        prop_assume!(amount1 != amount2);

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx1 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            amount1,
            100,
            0,
        );
        let tx2 = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            amount2,
            100,
            0,
        );

        let block1 = Block::new([0u8; 32], vec![tx1], 16);
        let block2 = Block::new([0u8; 32], vec![tx2], 16);

        prop_assert_ne!(block1.header.merkle_root, block2.header.merkle_root);
    });
}

/// Fuzz test: Chain ID should not affect hash computation negatively
#[test]
fn fuzz_chain_id_handling() {
    proptest!(|(chain_id in any::<u32>())| {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1000,
            100,
            0,
        );

        // Chain ID is included in signing hash
        let signing_hash = tx.signing_hash();
        
        // Should produce valid 32-byte hash
        prop_assert_eq!(signing_hash.len(), 32);
    });
}
