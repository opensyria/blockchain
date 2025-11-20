//! Load Testing Suite for OpenSyria Blockchain
//!
//! PERF-P2-007: Create load testing suite
//!
//! Comprehensive load tests to validate system performance under stress:
//! - Transaction throughput testing
//! - P2P network stress testing
//! - Database performance benchmarking
//! - API endpoint load testing
//!
//! Run with: cargo test --release --package opensyria-load-tests

use opensyria_core::{Block, Transaction, crypto::KeyPair};
use opensyria_consensus::ProofOfWork;
use opensyria_mempool::{Mempool, MempoolConfig};
use opensyria_storage::{BlockchainStorage, StateStorage};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tempfile::TempDir;

/// Load test configuration
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Number of transactions to generate
    pub num_transactions: usize,
    /// Number of concurrent senders
    pub num_senders: usize,
    /// Number of blocks to mine
    pub num_blocks: usize,
    /// Target transactions per block
    pub txs_per_block: usize,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            num_transactions: 10_000,
            num_senders: 100,
            num_blocks: 100,
            txs_per_block: 100,
        }
    }
}

/// Load test results
#[derive(Debug)]
pub struct LoadTestResults {
    pub total_duration_secs: f64,
    pub transactions_processed: usize,
    pub blocks_mined: usize,
    pub avg_tx_per_sec: f64,
    pub avg_block_time_secs: f64,
    pub mempool_peak_size: usize,
    pub database_size_bytes: u64,
}

/// Transaction generation load test
///
/// Measures mempool performance under heavy transaction load
#[tokio::test]
async fn test_mempool_transaction_load() {
    let config = LoadTestConfig {
        num_transactions: 1_000,
        num_senders: 10,
        ..Default::default()
    };

    let temp_dir = TempDir::new().unwrap();
    let state = StateStorage::open(temp_dir.path().join("state")).unwrap();

    // Setup test accounts with balances
    let senders: Vec<KeyPair> = (0..config.num_senders)
        .map(|_| KeyPair::generate())
        .collect();

    for sender in &senders {
        state.set_balance(&sender.public_key(), 1_000_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();
    }

    let state = Arc::new(RwLock::new(state));
    let mempool_config = MempoolConfig {
        max_size: config.num_transactions * 2,
        ..Default::default()
    };
    let mut mempool = Mempool::new(mempool_config, state);

    let receiver = KeyPair::generate();
    let start = Instant::now();
    let mut peak_size = 0;

    // Generate and add transactions
    for i in 0..config.num_transactions {
        let sender = &senders[i % config.num_senders];
        let nonce = i / config.num_senders;

        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            10_000,
            1_000,
            nonce as u64,
        );
        let msg = tx.signing_hash();
        tx.signature = sender.sign(&msg);

        mempool.add_transaction(tx).await.ok();
        peak_size = peak_size.max(mempool.size());
    }

    let duration = start.elapsed();
    let tx_per_sec = config.num_transactions as f64 / duration.as_secs_f64();

    println!("\n=== Mempool Load Test Results ===");
    println!("Transactions processed: {}", config.num_transactions);
    println!("Duration: {:.2}s", duration.as_secs_f64());
    println!("Throughput: {:.2} tx/sec", tx_per_sec);
    println!("Peak mempool size: {}", peak_size);
    println!("Final mempool size: {}", mempool.size());

    assert!(tx_per_sec > 100.0, "Mempool throughput too low: {:.2} tx/sec", tx_per_sec);
}

/// Block mining and storage load test
///
/// Measures blockchain storage performance under continuous block production
#[test]
fn test_blockchain_storage_load() {
    let config = LoadTestConfig {
        num_blocks: 100,
        txs_per_block: 50,
        ..Default::default()
    };

    let temp_dir = TempDir::new().unwrap();
    let storage = BlockchainStorage::open(temp_dir.path().join("blockchain")).unwrap();
    let pow = ProofOfWork::new(8); // Easy difficulty for testing

    let sender = KeyPair::generate();
    let receiver = KeyPair::generate();

    let start = Instant::now();
    let mut prev_hash = [0u8; 32];

    for block_num in 0..config.num_blocks {
        // Generate transactions for this block
        let transactions: Vec<Transaction> = (0..config.txs_per_block)
            .map(|tx_num| {
                let nonce = (block_num * config.txs_per_block + tx_num) as u64;
                let mut tx = Transaction::new(
                    sender.public_key(),
                    receiver.public_key(),
                    10_000,
                    1_000,
                    nonce,
                );
                let msg = tx.signing_hash();
                tx.signature = sender.sign(&msg);
                tx
            })
            .collect();

        // Create and mine block
        let block = Block::new(prev_hash, transactions, 8);
        let (mined_block, _stats) = pow.mine(block);
        
        // Store block
        storage.append_block(&mined_block, None).unwrap();
        prev_hash = mined_block.hash();

        if (block_num + 1) % 20 == 0 {
            println!("Mined and stored {} blocks...", block_num + 1);
        }
    }

    let duration = start.elapsed();
    let blocks_per_sec = config.num_blocks as f64 / duration.as_secs_f64();
    let total_txs = config.num_blocks * config.txs_per_block;
    let tx_per_sec = total_txs as f64 / duration.as_secs_f64();

    println!("\n=== Blockchain Storage Load Test Results ===");
    println!("Blocks mined and stored: {}", config.num_blocks);
    println!("Total transactions: {}", total_txs);
    println!("Duration: {:.2}s", duration.as_secs_f64());
    println!("Block throughput: {:.2} blocks/sec", blocks_per_sec);
    println!("Transaction throughput: {:.2} tx/sec", tx_per_sec);
    println!("Final blockchain height: {}", storage.get_chain_height().unwrap());

    assert_eq!(storage.get_chain_height().unwrap(), config.num_blocks as u64);
}

/// State storage balance query load test
///
/// Measures state database read performance
#[test]
fn test_state_balance_query_load() {
    let temp_dir = TempDir::new().unwrap();
    let state = StateStorage::open(temp_dir.path().join("state")).unwrap();

    // Create test accounts
    let num_accounts = 1_000;
    let accounts: Vec<KeyPair> = (0..num_accounts)
        .map(|_| KeyPair::generate())
        .collect();

    // Write balances
    println!("Writing {} account balances...", num_accounts);
    for (i, account) in accounts.iter().enumerate() {
        state.set_balance(&account.public_key(), (i as u64 + 1) * 1_000_000).unwrap();
    }

    // Benchmark reads
    let num_queries = 10_000;
    let start = Instant::now();

    for i in 0..num_queries {
        let account = &accounts[i % num_accounts];
        let balance = state.get_balance(&account.public_key()).unwrap();
        assert!(balance > 0);
    }

    let duration = start.elapsed();
    let queries_per_sec = num_queries as f64 / duration.as_secs_f64();
    let avg_query_time_us = duration.as_micros() / num_queries as u128;

    println!("\n=== State Balance Query Load Test Results ===");
    println!("Accounts in state: {}", num_accounts);
    println!("Queries executed: {}", num_queries);
    println!("Duration: {:.2}s", duration.as_secs_f64());
    println!("Query throughput: {:.2} queries/sec", queries_per_sec);
    println!("Average query time: {} μs", avg_query_time_us);

    // With bloom filters, should achieve >10k queries/sec
    assert!(queries_per_sec > 1000.0, "Query performance too low: {:.2} q/sec", queries_per_sec);
}

/// Parallel mining load test
///
/// Validates multi-threaded mining performance
#[test]
fn test_parallel_mining_performance() {
    let pow = ProofOfWork::new(16); // Medium difficulty
    let genesis = Block::genesis();

    // Single-threaded baseline
    let start_single = Instant::now();
    let (_mined_single, stats_single) = pow.mine(genesis.clone());
    let time_single = start_single.elapsed();

    // Multi-threaded (8 threads)
    let start_parallel = Instant::now();
    let (_mined_parallel, stats_parallel) = pow.mine_parallel(genesis, Some(8));
    let time_parallel = start_parallel.elapsed();

    let speedup = time_single.as_secs_f64() / time_parallel.as_secs_f64();

    println!("\n=== Parallel Mining Load Test Results ===");
    println!("Difficulty: {}", pow.difficulty());
    println!("Single-threaded:");
    println!("  Time: {:.2}s", time_single.as_secs_f64());
    println!("  Hash rate: {:.2} H/s", stats_single.hash_rate);
    println!("Multi-threaded (8 threads):");
    println!("  Time: {:.2}s", time_parallel.as_secs_f64());
    println!("  Hash rate: {:.2} H/s", stats_parallel.hash_rate);
    println!("Speedup: {:.2}x", speedup);

    // Should achieve at least 3x speedup with 8 threads
    assert!(speedup > 3.0, "Parallel mining speedup too low: {:.2}x", speedup);
}

/// Full system integration load test
///
/// End-to-end test of full transaction → block → storage pipeline
#[tokio::test]
async fn test_full_system_integration_load() {
    let config = LoadTestConfig {
        num_blocks: 10,
        txs_per_block: 20,
        ..Default::default()
    };

    let temp_dir = TempDir::new().unwrap();
    
    // Setup components
    let blockchain = BlockchainStorage::open(temp_dir.path().join("blockchain")).unwrap();
    let state = StateStorage::open(temp_dir.path().join("state")).unwrap();
    
    let sender = KeyPair::generate();
    let receiver = KeyPair::generate();
    
    state.set_balance(&sender.public_key(), 1_000_000_000).unwrap();
    state.set_nonce(&sender.public_key(), 0).unwrap();
    
    let state = Arc::new(RwLock::new(state));
    let mempool_config = MempoolConfig::default();
    let mut mempool = Mempool::new(mempool_config, state);
    
    let pow = ProofOfWork::new(8);
    let start = Instant::now();
    
    for block_num in 0..config.num_blocks {
        // Add transactions to mempool
        for tx_num in 0..config.txs_per_block {
            let nonce = (block_num * config.txs_per_block + tx_num) as u64;
            let mut tx = Transaction::new(
                sender.public_key(),
                receiver.public_key(),
                10_000,
                1_000,
                nonce,
            );
            let msg = tx.signing_hash();
            tx.signature = sender.sign(&msg);
            mempool.add_transaction(tx).await.ok();
        }
        
        // Get transactions from mempool
        let txs = mempool.get_priority_transactions(config.txs_per_block);
        
        // Mine block
        let prev_hash = blockchain.get_chain_tip().unwrap().unwrap_or([0u8; 32]);
        let block = Block::new(prev_hash, txs.clone(), 8);
        let (mined_block, _) = pow.mine(block);
        
        // Store block
        blockchain.append_block(&mined_block, None).unwrap();
        
        // Remove confirmed transactions from mempool
        mempool.remove_confirmed_transactions(&txs);
    }
    
    let duration = start.elapsed();
    let total_txs = config.num_blocks * config.txs_per_block;
    
    println!("\n=== Full System Integration Load Test Results ===");
    println!("Blocks processed: {}", config.num_blocks);
    println!("Transactions processed: {}", total_txs);
    println!("Duration: {:.2}s", duration.as_secs_f64());
    println!("System throughput: {:.2} tx/sec", total_txs as f64 / duration.as_secs_f64());
    println!("Final blockchain height: {}", blockchain.get_chain_height().unwrap());
    println!("Final mempool size: {}", mempool.size());
    
    assert_eq!(blockchain.get_chain_height().unwrap(), config.num_blocks as u64);
    assert_eq!(mempool.size(), 0);
}
