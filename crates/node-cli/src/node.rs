use anyhow::{Context, Result};
use colored::*;
use opensyria_consensus::{MiningStats, ProofOfWork};
use opensyria_core::{crypto::PublicKey, Block, Transaction};
use opensyria_governance::{
    GovernanceConfig, GovernanceManager, GovernanceStorage, ProposalType, Vote,
};
use opensyria_storage::Storage;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

/// Blockchain node with mining and transaction processing
pub struct Node {
    storage: Storage,
    governance_storage: GovernanceStorage,
    pending_transactions: HashMap<[u8; 32], Transaction>,
    #[allow(dead_code)]
    data_dir: PathBuf,
}

impl Node {
    /// Initialize a new blockchain with genesis block
    pub fn init(data_dir: PathBuf, difficulty: u32) -> Result<Self> {
        std::fs::create_dir_all(&data_dir).context("Failed to create data directory")?;

        let storage = Storage::open(data_dir.clone()).context("Failed to open storage")?;

        // Initialize governance
        let gov_dir = data_dir.join("governance");
        let governance_storage =
            GovernanceStorage::open(&gov_dir).context("Failed to open governance storage")?;

        if !governance_storage.has_snapshot()? {
            let config = GovernanceConfig::default();
            let manager = GovernanceManager::new(config);
            let snapshot = manager.create_snapshot();
            governance_storage.save_snapshot(&snapshot)?;
            tracing::info!("Governance system initialized");
        }

        // Create and store genesis block
        let genesis = Block::genesis();
        storage
            .blockchain
            .append_block(&genesis)
            .context("Failed to append genesis block")?;

        tracing::info!(
            "Genesis block created with hash: {}",
            hex::encode(genesis.hash())
        );

        Ok(Self {
            storage,
            governance_storage,
            pending_transactions: HashMap::new(),
            data_dir,
        })
    }

    /// Open existing blockchain node
    pub fn open(data_dir: PathBuf) -> Result<Self> {
        let storage = Storage::open(data_dir.clone()).context("Failed to open storage")?;

        let gov_dir = data_dir.join("governance");
        let governance_storage =
            GovernanceStorage::open(&gov_dir).context("Failed to open governance storage")?;

        let height = storage
            .blockchain
            .get_chain_height()
            .context("Failed to get chain height")?;

        if height == 0 {
            anyhow::bail!("Node not initialized. Run 'init' first.");
        }

        tracing::info!("Opened blockchain at height {}", height);

        Ok(Self {
            storage,
            governance_storage,
            pending_transactions: HashMap::new(),
            data_dir,
        })
    }

    /// Get current blockchain height
    pub fn get_height(&self) -> Result<u64> {
        self.storage
            .blockchain
            .get_chain_height()
            .context("Failed to get chain height")
    }

    /// Get chain tip (latest block)
    pub fn get_tip(&self) -> Result<Option<Block>> {
        let tip_hash = self.storage.blockchain.get_chain_tip()?;

        if let Some(hash) = tip_hash {
            self.storage
                .blockchain
                .get_block(&hash)
                .context("Failed to get tip block")
        } else {
            Ok(None)
        }
    }

    /// Get block by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>> {
        self.storage
            .blockchain
            .get_block_by_height(height)
            .context("Failed to get block")
    }

    /// Get block range
    pub fn get_block_range(&self, start: u64, end: u64) -> Result<Vec<Block>> {
        self.storage
            .blockchain
            .get_block_range(start, end)
            .context("Failed to get block range")
    }

    /// Get account balance
    pub fn get_balance(&self, address: &PublicKey) -> Result<u64> {
        self.storage
            .state
            .get_balance(address)
            .context("Failed to get balance")
    }

    /// Get miner address (temporary: generates new address each time)
    /// TODO: Load from wallet configuration
    fn get_miner_address(&self) -> Result<PublicKey> {
        use opensyria_core::crypto::KeyPair;
        
        // For now, generate a deterministic address based on node data
        // In production, this should load from wallet
        let keypair = KeyPair::generate();
        Ok(keypair.public_key())
    }

    /// Process and apply a transaction to state
    pub fn process_transaction(&mut self, tx: Transaction) -> Result<()> {
        // Verify signature
        tx.verify().context("Transaction verification failed")?;

        // Check nonce
        let expected_nonce = self.storage.state.get_nonce(&tx.from)?;
        if tx.nonce != expected_nonce {
            anyhow::bail!(
                "Invalid nonce: expected {}, got {}",
                expected_nonce,
                tx.nonce
            );
        }

        // Process transfer
        let total = tx.amount + tx.fee;
        self.storage
            .state
            .transfer(&tx.from, &tx.to, total)
            .context("Transfer failed")?;

        // Increment nonce
        self.storage.state.increment_nonce(&tx.from)?;

        tracing::info!(
            "Processed transaction: {} SYL from {}... to {}...",
            tx.amount as f64 / 1_000_000.0,
            hex::encode(&tx.from.0[..8]),
            hex::encode(&tx.to.0[..8])
        );

        Ok(())
    }

    /// Start mining blocks
    pub fn start_mining(&mut self, block_count: u32, difficulty: u32, verbose: bool) -> Result<()> {
        println!("{}", "═".repeat(60).cyan());
        println!("{}", "  OpenSyria Mining Node  ".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();

        let pow = ProofOfWork::new(difficulty);
        let current_height = self.get_height()?;

        println!("{}: {}", "Starting height".yellow(), current_height);
        println!("{}: {}", "Difficulty".yellow(), difficulty);
        println!(
            "{}: {}",
            "Target blocks".yellow(),
            if block_count == 0 {
                "continuous".to_string()
            } else {
                block_count.to_string()
            }
        );
        println!();

        let mut mined_count = 0u32;
        let mining_start = Instant::now();

        loop {
            // Get current tip
            let tip = self.get_tip()?.context("No tip block found")?;
            let previous_hash = tip.hash();
            let new_height = current_height + mined_count as u64 + 1;

            // Get miner address (use first wallet address or generate one)
            let miner_address = self.get_miner_address()?;

            // Calculate total fees from pending transactions
            let total_fees: u64 = 0; // TODO: sum fees from pending transactions when mempool integrated

            // Create coinbase transaction
            let coinbase = Transaction::coinbase(
                opensyria_core::CHAIN_ID_MAINNET,
                miner_address,
                new_height,
                total_fees,
            )
            .context("Failed to create coinbase transaction")?;

            // Create transactions vector with coinbase first
            let mut transactions = vec![coinbase];
            // TODO: Add pending transactions from mempool

            // Create new block with coinbase
            let block = Block::new(previous_hash, transactions, difficulty);

            if verbose {
                println!(
                    "{} {}",
                    "▶ Mining Block".green().bold(),
                    current_height + mined_count as u64 + 1
                );
            }

            // Mine the block
            let (mined_block, stats) = if verbose {
                pow.mine_with_callback(block, |hashes, rate| {
                    print!(
                        "\r  {} {} | {} {:.2} H/s",
                        "Hashes:".dimmed(),
                        format!("{:>12}", hashes).white(),
                        "Rate:".dimmed(),
                        rate
                    );
                    use std::io::Write;
                    std::io::stdout().flush().unwrap();
                })
            } else {
                pow.mine(block)
            };

            if verbose {
                println!(); // New line after progress
            }

            // Append to blockchain
            self.storage
                .blockchain
                .append_block(&mined_block)
                .context("Failed to append mined block")?;

            mined_count += 1;
            let new_height = current_height + mined_count as u64;

            // Print mining result
            self.print_mining_result(new_height, &mined_block, &stats);

            // Check if we should stop
            if block_count > 0 && mined_count >= block_count {
                break;
            }
        }

        // Final summary
        let total_time = mining_start.elapsed();
        println!();
        println!("{}", "═".repeat(60).cyan());
        println!("{}", "  Mining Summary  ".cyan().bold());
        println!("{}", "═".repeat(60).cyan());
        println!();
        println!("{}: {} blocks", "Mined".yellow(), mined_count);
        println!(
            "{}: {:.2}s",
            "Total time".yellow(),
            total_time.as_secs_f64()
        );
        println!(
            "{}: {:.2}s",
            "Avg time/block".yellow(),
            total_time.as_secs_f64() / mined_count as f64
        );
        println!("{}: {}", "Final height".yellow(), self.get_height()?);
        println!();

        Ok(())
    }

    // ===== Governance Methods =====

    /// Load governance manager from storage
    pub fn load_governance(&self) -> Result<GovernanceManager> {
        if !self.governance_storage.has_snapshot()? {
            let config = GovernanceConfig::default();
            return Ok(GovernanceManager::new(config));
        }

        let snapshot = self.governance_storage.load_snapshot()?;
        Ok(GovernanceManager::from_snapshot(snapshot))
    }

    /// Save governance manager to storage
    pub fn save_governance(&self, manager: &GovernanceManager) -> Result<()> {
        let snapshot = manager.create_snapshot();
        self.governance_storage.save_snapshot(&snapshot)?;
        Ok(())
    }

    /// Create a new governance proposal
    pub fn create_proposal(
        &self,
        proposer: PublicKey,
        proposal_type: ProposalType,
        title: String,
        description: String,
    ) -> Result<u64> {
        let mut manager = self.load_governance()?;

        // Get proposer stake from state
        let proposer_balance = self.storage.state.get_balance(&proposer)?;

        // Get current block height
        let current_height = self.storage.blockchain.get_chain_height()?;

        // Calculate total voting power (sum of all balances)
        // For simplicity, we'll use a fixed large number
        // TODO: Implement efficient total balance calculation
        let total_voting_power = 1_000_000_000_000;

        let proposal_id = manager.create_proposal(
            proposer,
            proposer_balance,
            proposal_type,
            title,
            description,
            current_height,
            total_voting_power,
        )?;

        self.save_governance(&manager)?;

        Ok(proposal_id)
    }

    /// Cast a vote on a proposal
    pub fn vote_on_proposal(&self, proposal_id: u64, voter: PublicKey, vote: Vote) -> Result<()> {
        let mut manager = self.load_governance()?;

        // Get voter's voting power from their balance
        let voting_power = self.storage.state.get_balance(&voter)?;

        // Get current block height
        let current_height = self.storage.blockchain.get_chain_height()?;

        manager.vote(proposal_id, voter, vote, voting_power, current_height)?;

        self.save_governance(&manager)?;

        Ok(())
    }

    /// Process proposals (finalize ended voting periods)
    pub fn process_proposals(&self) -> Result<usize> {
        let mut manager = self.load_governance()?;
        let current_height = self.storage.blockchain.get_chain_height()?;

        let before_stats = manager.get_statistics();
        manager.process_proposals(current_height);
        let after_stats = manager.get_statistics();

        let newly_finalized = (after_stats.passed_proposals + after_stats.rejected_proposals)
            - (before_stats.passed_proposals + before_stats.rejected_proposals);

        // Execute ready proposals
        let ready_ids: Vec<u64> = manager
            .get_ready_for_execution(current_height)
            .into_iter()
            .map(|p| p.id)
            .collect();

        for proposal_id in ready_ids {
            self.execute_proposal(&mut manager, proposal_id)?;
        }

        self.save_governance(&manager)?;

        Ok(newly_finalized)
    }

    /// Execute a passed proposal
    fn execute_proposal(&self, manager: &mut GovernanceManager, proposal_id: u64) -> Result<()> {
        let proposal = manager
            .get_proposal(proposal_id)
            .ok_or_else(|| anyhow::anyhow!("Proposal not found"))?;

        tracing::info!("Executing proposal {}: {}", proposal_id, proposal.title);

        match &proposal.proposal_type {
            ProposalType::MinimumFee { new_fee } => {
                tracing::info!("Setting minimum fee to: {}", new_fee);
                // TODO: Apply to mempool configuration
            }
            ProposalType::BlockSizeLimit { new_limit } => {
                tracing::info!("Setting block size limit to: {} bytes", new_limit);
                // TODO: Apply to consensus configuration
            }
            ProposalType::DifficultyAdjustment {
                target_block_time,
                adjustment_interval,
            } => {
                tracing::info!(
                    "Setting difficulty adjustment: target={}s, interval={} blocks",
                    target_block_time,
                    adjustment_interval
                );
                // TODO: Apply to consensus configuration
            }
            ProposalType::TextProposal { .. } => {
                // Non-binding, just log
                tracing::info!("Text proposal (non-binding)");
            }
            _ => {
                tracing::warn!("Unimplemented proposal type execution");
            }
        }

        manager.mark_proposal_executed(proposal_id)?;

        Ok(())
    }

    fn print_mining_result(&self, height: u64, block: &Block, stats: &MiningStats) {
        println!(
            "  {} {}",
            "✓ Block Mined".green(),
            format!("#{}", height).bold()
        );
        println!(
            "    {}: {}",
            "Hash".dimmed(),
            hex::encode(block.hash()).white()
        );
        println!(
            "    {}: {}",
            "Nonce".dimmed(),
            format!("{}", stats.nonce_found).white()
        );
        println!(
            "    {}: {} ({:.2}s)",
            "Hashes".dimmed(),
            format!("{}", stats.hashes_computed).white(),
            stats.duration.as_secs_f64()
        );
        println!("    {}: {:.2} H/s", "Hash Rate".dimmed(), stats.hash_rate);
        println!();
    }

    // API accessors for wallet API

    /// Get reference to blockchain
    pub fn get_blockchain(&self) -> &opensyria_storage::BlockchainStorage {
        &self.storage.blockchain
    }

    /// Get reference to state storage
    pub fn get_state(&self) -> &opensyria_storage::StateStorage {
        &self.storage.state
    }

    /// Get pending transactions
    pub fn get_pending_transactions(&self) -> Vec<Transaction> {
        self.pending_transactions.values().cloned().collect()
    }

    /// Add transaction to pending pool
    #[allow(dead_code)]
    pub fn add_transaction_to_mempool(&mut self, transaction: Transaction) -> Result<()> {
        // Verify transaction
        transaction
            .verify()
            .context("Invalid transaction signature")?;

        // Check sender has sufficient balance
        let balance = self.storage.state.get_balance(&transaction.from)?;
        let nonce = self.storage.state.get_nonce(&transaction.from)?;

        let total_cost = transaction.amount + transaction.fee;
        if balance < total_cost {
            anyhow::bail!("Insufficient balance");
        }

        // Check nonce
        if transaction.nonce != nonce {
            anyhow::bail!(
                "Invalid nonce. Expected {}, got {}",
                nonce,
                transaction.nonce
            );
        }

        // Add to pending pool
        let tx_hash = transaction.hash();
        self.pending_transactions.insert(tx_hash, transaction);

        Ok(())
    }
}
