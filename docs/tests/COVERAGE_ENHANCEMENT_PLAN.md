# Advanced Test Coverage Enhancement
## Target: 80% Code Coverage for Mainnet Launch

**Current Coverage:** ~65%  
**Target Coverage:** 80%  
**Gap:** 15% (critical paths in consensus, network, governance)  
**Deadline:** Week 11 (before Phase 3 completion)

---

## Coverage Gaps by Module

### 1. Consensus Module - Current: 72% → Target: 90%

#### Missing Coverage

**File: `crates/consensus/src/consensus.rs`**
- Line 245-267: `validate_block_pow()` edge cases
- Line 312-334: `adjust_difficulty()` overflow scenarios
- Line 401-425: `check_reorg_depth()` boundary conditions

**Tests to Add:**

```rust
// tests/consensus_coverage.rs

#[test]
fn test_pow_validation_edge_cases() {
    // Test 1: Block with maximum valid nonce
    let block = create_test_block(nonce: u64::MAX);
    assert!(validate_block_pow(&block, difficulty: 16));
    
    // Test 2: Block with hash just above difficulty threshold
    let block = create_block_with_hash_value(difficulty + 1);
    assert!(!validate_block_pow(&block, difficulty));
    
    // Test 3: Block with hash exactly at difficulty threshold
    let block = create_block_with_hash_value(difficulty);
    assert!(validate_block_pow(&block, difficulty));
}

#[test]
fn test_difficulty_adjustment_overflow() {
    // Test 1: Extremely fast block times (1 second per block)
    let fast_chain = create_chain_with_block_times(vec![1; 100]);
    let new_diff = adjust_difficulty(&fast_chain);
    assert!(new_diff <= initial_difficulty * 125 / 100); // Max 25% increase
    
    // Test 2: Extremely slow block times (1 hour per block)
    let slow_chain = create_chain_with_block_times(vec![3600; 100]);
    let new_diff = adjust_difficulty(&slow_chain);
    assert!(new_diff >= initial_difficulty * 75 / 100); // Max 25% decrease
    
    // Test 3: Integer overflow in time calculation
    let overflow_chain = create_chain_with_timestamps(vec![0, u64::MAX]);
    let new_diff = adjust_difficulty(&overflow_chain);
    assert!(new_diff > 0); // Should not crash or return 0
}

#[test]
fn test_reorg_depth_boundaries() {
    // Test 1: Reorg exactly at MAX_REORG_DEPTH (100 blocks)
    let chain_a = create_chain(height: 200);
    let chain_b = create_fork_at(chain_a, fork_height: 100);
    assert!(check_reorg_depth(chain_a, chain_b)); // Should be allowed
    
    // Test 2: Reorg beyond MAX_REORG_DEPTH (101 blocks)
    let chain_c = create_fork_at(chain_a, fork_height: 99);
    assert!(!check_reorg_depth(chain_a, chain_c)); // Should be rejected
    
    // Test 3: Reorg at genesis (maximum depth)
    let chain_d = create_fork_at(chain_a, fork_height: 0);
    assert!(!check_reorg_depth(chain_a, chain_d)); // Should be rejected
}

#[test]
fn test_median_time_past_calculation() {
    // Test 1: Exactly 11 blocks (normal case)
    let chain = create_chain_with_timestamps(vec![100, 200, 150, 300, 250, 350, 400, 500, 450, 550, 600]);
    assert_eq!(median_time_past(&chain), 350); // Median of 11 values
    
    // Test 2: Less than 11 blocks (genesis case)
    let short_chain = create_chain_with_timestamps(vec![100, 200, 300]);
    assert_eq!(median_time_past(&short_chain), 200); // Median of 3 values
    
    // Test 3: Unsorted timestamps (out-of-order but still valid)
    let unsorted = create_chain_with_timestamps(vec![500, 100, 300, 200, 400]);
    assert_eq!(median_time_past(&unsorted), 300); // Median after sorting
}

#[test]
fn test_timestamp_validation_mtp() {
    let chain = create_test_chain(height: 20);
    let mtp = median_time_past(&chain);
    
    // Test 1: Timestamp equal to MTP (should fail)
    let block_eq = create_block(timestamp: mtp);
    assert!(!validate_timestamp(&block_eq, &chain));
    
    // Test 2: Timestamp one second after MTP (should pass)
    let block_after = create_block(timestamp: mtp + 1);
    assert!(validate_timestamp(&block_after, &chain));
    
    // Test 3: Timestamp far in future (beyond MAX_FUTURE_DRIFT)
    let current_time = get_current_time();
    let block_future = create_block(timestamp: current_time + 61);
    assert!(!validate_timestamp(&block_future, &chain)); // MAX_FUTURE_DRIFT = 60
}
```

---

### 2. Network Module - Current: 58% → Target: 75%

#### Missing Coverage

**File: `crates/network/src/p2p.rs`**
- Line 189-215: Peer eviction logic
- Line 267-289: ASN diversity enforcement
- Line 334-356: Message deduplication

**Tests to Add:**

```rust
// tests/network_coverage.rs

#[test]
async fn test_peer_eviction_strategy() {
    let mut network = create_test_network();
    
    // Fill to max capacity (50 inbound peers)
    for i in 0..50 {
        network.connect_peer(create_peer(id: i)).await;
    }
    
    // Test 1: New peer with higher reputation evicts lowest reputation peer
    let low_rep_peer = network.peers[0].clone(); // Assume sorted by reputation
    let high_rep_peer = create_peer(id: 100, reputation: 100);
    
    network.connect_peer(high_rep_peer).await;
    assert!(!network.has_peer(low_rep_peer.id));
    assert!(network.has_peer(high_rep_peer.id));
    
    // Test 2: New peer with lower reputation is rejected
    let new_low_rep = create_peer(id: 101, reputation: -50);
    network.connect_peer(new_low_rep).await;
    assert!(!network.has_peer(new_low_rep.id));
}

#[test]
async fn test_asn_diversity_enforcement() {
    let mut network = create_test_network();
    
    // Test 1: Accept up to 10% peers from same ASN
    for i in 0..5 {
        let peer = create_peer_with_asn(asn: 12345); // 5 peers = 10% of 50
        assert!(network.connect_peer(peer).await.is_ok());
    }
    
    // Test 2: Reject 6th peer from same ASN (exceeds 10%)
    let excess_peer = create_peer_with_asn(asn: 12345);
    assert!(network.connect_peer(excess_peer).await.is_err());
    
    // Test 3: Accept peer from different ASN
    let diverse_peer = create_peer_with_asn(asn: 67890);
    assert!(network.connect_peer(diverse_peer).await.is_ok());
}

#[test]
async fn test_message_deduplication() {
    let mut network = create_test_network();
    let block = create_test_block();
    
    // Test 1: First message is processed
    let msg1 = create_block_message(block.clone());
    assert!(network.handle_message(msg1).await.is_ok());
    
    // Test 2: Duplicate message is ignored (within 5 minute window)
    let msg2 = create_block_message(block.clone());
    assert!(network.handle_message(msg2).await.is_err()); // Should be deduplicated
    
    // Test 3: After 5 minutes, same message can be reprocessed
    network.advance_time(Duration::from_secs(301));
    let msg3 = create_block_message(block.clone());
    assert!(network.handle_message(msg3).await.is_ok());
}

#[test]
async fn test_rate_limiting() {
    let mut network = create_test_network();
    let peer = create_peer(id: 42);
    network.connect_peer(peer.clone()).await;
    
    // Test 1: 100 messages/sec allowed
    for i in 0..100 {
        let msg = create_test_message();
        assert!(network.handle_peer_message(peer.id, msg).await.is_ok());
    }
    
    // Test 2: 101st message within same second is rate-limited
    let excess_msg = create_test_message();
    assert!(network.handle_peer_message(peer.id, excess_msg).await.is_err());
    
    // Test 3: After 1 second, rate limit resets
    network.advance_time(Duration::from_secs(1));
    let new_msg = create_test_message();
    assert!(network.handle_peer_message(peer.id, new_msg).await.is_ok());
}
```

---

### 3. Governance Module - Current: 82% → Target: 90%

#### Missing Coverage

**File: `crates/governance/src/governance.rs`**
- Line 145-167: Proposal parameter validation
- Line 223-245: Emergency governance pause
- Line 312-334: Quadratic voting overflow

**Tests to Add:**

```rust
// tests/governance_coverage.rs

#[test]
fn test_proposal_parameter_validation() {
    let gov = create_governance_instance();
    
    // Test 1: Title too long (max 200 chars)
    let long_title = "A".repeat(201);
    let proposal = create_proposal(title: long_title);
    assert!(gov.validate_proposal(&proposal).is_err());
    
    // Test 2: Description too long (max 10KB)
    let long_desc = "B".repeat(10_001);
    let proposal = create_proposal(description: long_desc);
    assert!(gov.validate_proposal(&proposal).is_err());
    
    // Test 3: Invalid voting period (must be 7-90 days)
    let short_period = create_proposal(voting_period: Duration::from_secs(6 * 24 * 3600));
    assert!(gov.validate_proposal(&short_period).is_err());
    
    let long_period = create_proposal(voting_period: Duration::from_secs(91 * 24 * 3600));
    assert!(gov.validate_proposal(&long_period).is_err());
    
    // Test 4: Invalid quorum (must be 1-100%)
    let invalid_quorum = create_proposal(quorum: 101);
    assert!(gov.validate_proposal(&invalid_quorum).is_err());
    
    // Test 5: Zero or negative deposit
    let zero_deposit = create_proposal(deposit: 0);
    assert!(gov.validate_proposal(&zero_deposit).is_err());
}

#[test]
fn test_emergency_governance_pause() {
    let mut gov = create_governance_instance();
    
    // Test 1: Admin can pause governance
    gov.emergency_pause(admin_signature).unwrap();
    assert!(gov.is_paused());
    
    // Test 2: Cannot create proposals while paused
    let proposal = create_test_proposal();
    assert!(gov.create_proposal(proposal).is_err());
    
    // Test 3: Cannot vote while paused
    assert!(gov.vote(proposal_id: 1, choice: "YES").is_err());
    
    // Test 4: Can unpause and resume normal operations
    gov.emergency_unpause(admin_signature).unwrap();
    assert!(!gov.is_paused());
    assert!(gov.create_proposal(create_test_proposal()).is_ok());
}

#[test]
fn test_quadratic_voting_overflow() {
    let gov = create_governance_instance();
    
    // Test 1: Normal case (sqrt of 10,000 = 100 votes)
    let vote_power = gov.calculate_vote_power(amount: 10_000);
    assert_eq!(vote_power, 100);
    
    // Test 2: Maximum safe value (sqrt of u64::MAX should not overflow)
    let max_vote = gov.calculate_vote_power(amount: u64::MAX);
    assert!(max_vote > 0); // Should not panic or return 0
    assert!(max_vote < u32::MAX as u64); // Result should fit in reasonable range
    
    // Test 3: Edge case (amount = 1 should give 1 vote)
    let min_vote = gov.calculate_vote_power(amount: 1);
    assert_eq!(min_vote, 1);
    
    // Test 4: Zero amount should give zero votes
    let zero_vote = gov.calculate_vote_power(amount: 0);
    assert_eq!(zero_vote, 0);
}
```

---

### 4. Storage Module - Current: 64% → Target: 80%

#### Missing Coverage

**File: `crates/storage/src/rocksdb_storage.rs`**
- Line 178-201: Database compaction
- Line 256-278: Merkle proof generation
- Line 334-356: Backup/restore

**Tests to Add:**

```rust
// tests/storage_coverage.rs

#[test]
fn test_database_compaction() {
    let storage = create_test_storage();
    
    // Test 1: Write 10K blocks, trigger compaction
    for i in 0..10_000 {
        storage.store_block(create_test_block(height: i)).unwrap();
    }
    
    let size_before = storage.get_db_size();
    storage.compact_range(start: 0, end: 10_000).unwrap();
    let size_after = storage.get_db_size();
    
    // Compaction should reduce size (deleted/overwritten keys removed)
    assert!(size_after <= size_before);
    
    // Test 2: All blocks still readable after compaction
    for i in 0..10_000 {
        assert!(storage.get_block(i).is_some());
    }
}

#[test]
fn test_merkle_proof_generation() {
    let storage = create_test_storage();
    let transactions = vec![create_tx(1), create_tx(2), create_tx(3), create_tx(4)];
    let merkle_root = storage.build_merkle_tree(&transactions);
    
    // Test 1: Generate proof for transaction at index 2
    let proof = storage.generate_merkle_proof(&transactions, index: 2).unwrap();
    
    // Test 2: Verify proof is valid
    assert!(storage.verify_merkle_proof(
        leaf: transactions[2].hash(),
        proof: &proof,
        root: merkle_root
    ));
    
    // Test 3: Proof for wrong transaction should fail
    assert!(!storage.verify_merkle_proof(
        leaf: transactions[0].hash(),
        proof: &proof,  // Proof for tx[2], not tx[0]
        root: merkle_root
    ));
    
    // Test 4: Tampered proof should fail
    let mut tampered_proof = proof.clone();
    tampered_proof[0] = [0u8; 32]; // Corrupt first hash in proof
    assert!(!storage.verify_merkle_proof(
        leaf: transactions[2].hash(),
        proof: &tampered_proof,
        root: merkle_root
    ));
}

#[test]
fn test_database_backup_restore() {
    let storage = create_test_storage();
    
    // Test 1: Store data
    for i in 0..100 {
        storage.store_block(create_test_block(height: i)).unwrap();
    }
    
    // Test 2: Create backup
    let backup_path = "/tmp/opensyria_backup";
    storage.create_backup(backup_path).unwrap();
    assert!(Path::new(backup_path).exists());
    
    // Test 3: Restore from backup to new storage instance
    let restored_storage = Storage::restore_from_backup(backup_path).unwrap();
    
    // Test 4: Verify all data restored correctly
    for i in 0..100 {
        let original = storage.get_block(i).unwrap();
        let restored = restored_storage.get_block(i).unwrap();
        assert_eq!(original, restored);
    }
}
```

---

## Integration Tests (Cross-Module Coverage)

**File: `tests/integration_coverage.rs`**

```rust
#[tokio::test]
async fn test_end_to_end_transaction_flow() {
    // Setup: 3-node network
    let network = create_test_network(nodes: 3);
    
    // Step 1: Create transaction on node 1
    let tx = create_transaction(
        from: "syl1qalice...",
        to: "syl1qbob...",
        amount: 1000
    );
    network.nodes[0].submit_transaction(tx.clone()).await.unwrap();
    
    // Step 2: Transaction propagates to all nodes
    tokio::time::sleep(Duration::from_secs(1)).await;
    for node in &network.nodes {
        assert!(node.mempool.contains(&tx.hash()));
    }
    
    // Step 3: Node 2 mines block with transaction
    let block = network.nodes[1].mine_block().await.unwrap();
    assert!(block.transactions.contains(&tx));
    
    // Step 4: Block propagates to all nodes
    tokio::time::sleep(Duration::from_secs(1)).await;
    for node in &network.nodes {
        assert_eq!(node.blockchain.get_block(block.height).unwrap(), block);
    }
    
    // Step 5: Balance updated on all nodes
    for node in &network.nodes {
        assert_eq!(node.get_balance("syl1qbob..."), 1000);
    }
}

#[tokio::test]
async fn test_network_partition_recovery() {
    let network = create_test_network(nodes: 6);
    
    // Step 1: Partition network (nodes 0-2 vs nodes 3-5)
    network.partition_nodes(group_a: [0, 1, 2], group_b: [3, 4, 5]);
    
    // Step 2: Both partitions mine separate chains
    for i in 0..10 {
        network.nodes[0].mine_block().await.unwrap(); // Chain A
        network.nodes[3].mine_block().await.unwrap(); // Chain B
    }
    
    // Step 3: Verify chains diverged
    let chain_a_tip = network.nodes[0].blockchain.get_tip();
    let chain_b_tip = network.nodes[3].blockchain.get_tip();
    assert_ne!(chain_a_tip.hash, chain_b_tip.hash);
    
    // Step 4: Heal partition
    network.heal_partition();
    tokio::time::sleep(Duration::from_secs(5)).await; // Allow sync
    
    // Step 5: All nodes converge to longest chain
    let final_tip = network.nodes[0].blockchain.get_tip();
    for node in &network.nodes {
        assert_eq!(node.blockchain.get_tip().hash, final_tip.hash);
    }
}
```

---

## Property-Based Testing (QuickCheck/Proptest)

**File: `tests/property_based_coverage.rs`**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_transaction_serialization_roundtrip(
        from in any::<[u8; 32]>(),
        to in any::<[u8; 32]>(),
        amount in 1u64..1_000_000_000,
        nonce in any::<u64>()
    ) {
        let tx = Transaction { from, to, amount, nonce, ..default() };
        let serialized = tx.serialize();
        let deserialized = Transaction::deserialize(&serialized).unwrap();
        prop_assert_eq!(tx, deserialized);
    }
    
    #[test]
    fn test_difficulty_adjustment_bounds(
        block_times in proptest::collection::vec(1u64..7200, 100)
    ) {
        let chain = create_chain_with_block_times(block_times);
        let new_diff = adjust_difficulty(&chain);
        
        // Difficulty should never be 0
        prop_assert!(new_diff > 0);
        
        // Difficulty change should be ≤25%
        let old_diff = chain.last().unwrap().difficulty;
        prop_assert!(new_diff >= old_diff * 75 / 100);
        prop_assert!(new_diff <= old_diff * 125 / 100);
    }
}
```

---

## Fuzzing Tests

**File: `fuzz/fuzz_targets/transaction_parser.rs`**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use opensyria_core::Transaction;

fuzz_target!(|data: &[u8]| {
    // Should never panic on any input
    let _ = Transaction::deserialize(data);
});
```

**Run Fuzzing:**
```bash
cargo install cargo-fuzz
cargo fuzz run transaction_parser -- -max_total_time=3600  # 1 hour
```

---

## Coverage Measurement

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage/ --all-features --workspace

# View report
open coverage/index.html
```

### CI/CD Integration

**File: `.github/workflows/coverage.yml`**

```yaml
name: Code Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests with coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage/
      
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
          fail_ci_if_error: true
      
      - name: Check coverage threshold
        run: |
          COVERAGE=$(grep -oP 'line-rate="\K[^"]+' coverage/cobertura.xml | head -1)
          echo "Coverage: $(echo "$COVERAGE * 100" | bc)%"
          if (( $(echo "$COVERAGE < 0.80" | bc -l) )); then
            echo "Coverage below 80% threshold!"
            exit 1
          fi
```

---

## Timeline & Milestones

**Week 1:**
- ✅ Consensus module tests (72% → 90%)
- ✅ Generate coverage report baseline

**Week 2:**
- ✅ Network module tests (58% → 75%)
- ✅ Governance module tests (82% → 90%)

**Week 3:**
- ✅ Storage module tests (64% → 80%)
- ✅ Integration tests (cross-module)

**Week 4:**
- ✅ Property-based tests (QuickCheck)
- ✅ Fuzzing tests (1M+ inputs)
- ✅ Final coverage report (verify 80%+)

---

## Success Criteria

- ✅ Overall coverage ≥80%
- ✅ Critical modules (consensus, wallet, network) ≥85%
- ✅ Zero uncovered critical paths (security-sensitive code)
- ✅ All property-based tests pass (10K+ random inputs)
- ✅ Fuzzing finds no crashes (1M+ inputs)
- ✅ CI/CD enforces 80% threshold on all PRs

---

**Last Updated:** November 19, 2025  
**Next Review:** Week 11 (before Phase 3 completion)

*"Test coverage is not just a number—it's confidence in correctness."*  
*"تغطية الاختبار ليست مجرد رقم - إنها الثقة في الصحة"*
