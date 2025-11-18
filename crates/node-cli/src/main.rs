mod node;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use node::Node;
use opensyria_core::crypto::PublicKey;
use ed25519_dalek::Signer;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opensyria-node")]
#[command(about = "Open Syria Blockchain Node | عقدة بلوكتشين سوريا المفتوحة", long_about = None)]
struct Cli {
    /// Node data directory
    #[arg(short, long, default_value = "~/.opensyria/node")]
    data_dir: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new blockchain node | تهيئة عقدة جديدة
    Init {
        /// Mining difficulty for genesis block
        #[arg(short, long, default_value = "16")]
        difficulty: u32,
    },

    /// Start mining blocks | بدء التعدين
    Mine {
        /// Number of blocks to mine (0 = continuous)
        #[arg(short, long, default_value = "0")]
        blocks: u32,

        /// Mining difficulty
        #[arg(short, long, default_value = "16")]
        difficulty: u32,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show blockchain info | عرض معلومات البلوكتشين
    Info,

    /// Show block details | عرض تفاصيل كتلة
    Block {
        /// Block height or "latest"
        height: String,
    },

    /// Show account balance | عرض رصيد حساب
    Balance {
        /// Account address (hex public key)
        address: String,
    },

    /// Process a transaction | معالجة معاملة
    ProcessTx {
        /// Path to signed transaction JSON
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Export blockchain data | تصدير بيانات البلوكتشين
    Export {
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Start height
        #[arg(long, default_value = "1")]
        start: u64,

        /// End height (0 = all)
        #[arg(long, default_value = "0")]
        end: u64,
    },

    /// Network operations | عمليات الشبكة
    Network {
        #[command(subcommand)]
        command: NetworkCommands,
    },

    /// Governance operations | عمليات الحوكمة
    Governance {
        #[command(subcommand)]
        command: GovernanceCommands,
    },

    /// Multi-signature accounts | حسابات متعددة التوقيع
    Multisig {
        #[command(subcommand)]
        command: MultisigCommands,
    },

    /// Mining pool operations | عمليات تجمع التعدين
    Pool {
        #[command(subcommand)]
        command: PoolCommands,
    },
}

#[derive(Subcommand)]
enum GovernanceCommands {
    /// Create a new proposal | إنشاء اقتراح جديد
    Propose {
        /// Proposal title
        #[arg(long)]
        title: String,

        /// Proposal description
        #[arg(long)]
        description: String,

        /// Proposal type (text, min-fee, etc.)
        #[arg(short = 't', long)]
        proposal_type: String,

        /// Proposer address (hex public key)
        #[arg(short, long)]
        proposer: String,
    },

    /// Vote on a proposal | التصويت على اقتراح
    Vote {
        /// Proposal ID
        proposal_id: u64,

        /// Vote choice (yes, no, abstain)
        #[arg(short, long)]
        choice: String,

        /// Voter address (hex public key)
        #[arg(short = 'a', long)]
        voter: String,
    },

    /// List all proposals | عرض جميع الاقتراحات
    List {
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Show proposal details | عرض تفاصيل اقتراح
    Show {
        /// Proposal ID
        proposal_id: u64,
    },

    /// Show governance statistics | عرض إحصائيات الحوكمة
    Stats,

    /// Process proposals (finalize ended voting periods)
    Process,
}

#[derive(Subcommand)]
enum MultisigCommands {
    /// Create a new multisig account | إنشاء حساب متعدد التوقيع
    Create {
        /// Signer public keys (hex, can be repeated)
        #[arg(short, long, required = true)]
        signer: Vec<String>,

        /// Minimum signatures required (M-of-N)
        #[arg(short = 't', long)]
        threshold: u8,

        /// Initial balance to fund account
        #[arg(short, long, default_value = "0")]
        balance: u64,
    },

    /// Show multisig account details | عرض تفاصيل الحساب
    Info {
        /// Multisig account address (hex)
        address: String,
    },

    /// Create a multisig transaction | إنشاء معاملة متعددة التوقيع
    CreateTx {
        /// Multisig account address (hex)
        #[arg(long)]
        from: String,

        /// Recipient address (hex)
        #[arg(long)]
        to: String,

        /// Amount to send
        #[arg(long)]
        amount: u64,

        /// Transaction fee
        #[arg(long)]
        fee: u64,

        /// Output file for unsigned transaction
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Sign a multisig transaction | توقيع معاملة
    Sign {
        /// Transaction file to sign
        #[arg(long)]
        tx_file: PathBuf,

        /// Signer's private key (hex)
        #[arg(long)]
        private_key: String,

        /// Output file for signed transaction
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Submit a fully-signed multisig transaction | إرسال معاملة مكتملة
    Submit {
        /// Signed transaction file
        #[arg(long)]
        tx_file: PathBuf,
    },
}

#[derive(Subcommand)]
enum PoolCommands {
    /// Initialize a new mining pool | تهيئة تجمع تعدين جديد
    Init {
        /// Pool operator public key (hex)
        #[arg(long)]
        operator: String,

        /// Pool fee percentage (0-100)
        #[arg(long, default_value = "2")]
        fee: u8,

        /// Share difficulty
        #[arg(long, default_value = "12")]
        share_difficulty: u32,

        /// Reward method (proportional, pps, pplns)
        #[arg(long, default_value = "proportional")]
        method: String,
    },

    /// Show pool statistics | عرض إحصائيات التجمع
    Stats,

    /// List all miners | عرض جميع المعدنين
    Miners,

    /// Show miner statistics | عرض إحصائيات معدن
    Miner {
        /// Miner public key (hex)
        address: String,
    },

    /// Register as a miner | التسجيل كمعدن
    Register {
        /// Miner public key (hex)
        address: String,
    },

    /// Process payouts | معالجة المدفوعات
    Payout {
        /// Miner public key (hex, optional - all miners if not specified)
        #[arg(long)]
        miner: Option<String>,
    },
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Start P2P network node | تشغيل عقدة الشبكة
    Start {
        /// Listen address for P2P connections
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/9000")]
        listen: String,

        /// Bootstrap peer addresses (can be repeated)
        #[arg(short, long)]
        bootstrap: Vec<String>,

        /// Enable mDNS for local peer discovery
        #[arg(long, default_value = "true")]
        mdns: bool,
    },

    /// List connected peers | عرض الأقران المتصلين
    Peers,

    /// Connect to a peer | الاتصال بنظير
    Dial {
        /// Peer multiaddr (e.g., /ip4/192.168.1.100/tcp/9000/p2p/12D3...)
        address: String,
    },

    /// Synchronize blockchain from network | مزامنة البلوكتشين
    Sync,

    /// Broadcast a block | بث كتلة
    BroadcastBlock {
        /// Block height to broadcast
        height: u64,
    },

    /// Show network status | عرض حالة الشبكة
    Status,

    /// Run as persistent daemon | تشغيل كخدمة دائمة
    Daemon {
        /// Listen address for P2P connections
        #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/9000")]
        listen: String,

        /// Bootstrap peer addresses (can be repeated)
        #[arg(short, long)]
        bootstrap: Vec<String>,

        /// Enable mDNS for local peer discovery
        #[arg(long, default_value = "true")]
        mdns: bool,

        /// Sync interval in seconds
        #[arg(long, default_value = "30")]
        sync_interval: u64,

        /// Enable auto-mining when daemon
        #[arg(long)]
        mine: bool,

        /// Mining difficulty (if --mine enabled)
        #[arg(long, default_value = "16")]
        difficulty: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    // Expand tilde in data_dir
    let data_dir = shellexpand::tilde(&cli.data_dir).to_string();
    let data_dir = PathBuf::from(data_dir);

    match cli.command {
        Commands::Init { difficulty } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Initializing Open Syria Node  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            let node = Node::init(data_dir.clone(), difficulty)?;

            println!("{}", "✓ Node initialized successfully".green());
            println!();
            println!("{}: {}", "Data directory".cyan(), data_dir.display());
            println!("{}: {}", "Genesis difficulty".cyan(), difficulty);
            println!("{}: {}", "Chain height".cyan(), node.get_height()?);
            println!();
        }

        Commands::Mine {
            blocks,
            difficulty,
            verbose,
        } => {
            let mut node = Node::open(data_dir)?;
            node.start_mining(blocks, difficulty, verbose)?;
        }

        Commands::Info => {
            let node = Node::open(data_dir)?;
            let height = node.get_height()?;
            let tip = node.get_tip()?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Blockchain Information  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Chain Height".yellow(), height);
            
            if let Some(tip_block) = tip {
                println!("{}: {}", "Latest Block".yellow(), hex::encode(tip_block.hash()));
                println!("{}: {}", "Timestamp".yellow(), tip_block.header.timestamp);
                println!("{}: {}", "Difficulty".yellow(), tip_block.header.difficulty);
                println!("{}: {}", "Transactions".yellow(), tip_block.transactions.len());
            }
            
            println!();
        }

        Commands::Block { height } => {
            let node = Node::open(data_dir)?;
            
            let block = if height == "latest" {
                node.get_tip()?
            } else {
                let h: u64 = height.parse()?;
                node.get_block_by_height(h)?
            };

            if let Some(block) = block {
                println!("{}", "═".repeat(60).cyan());
                println!("{}", format!("  Block Details  ").cyan().bold());
                println!("{}", "═".repeat(60).cyan());
                println!();
                println!("{}: {}", "Hash".yellow(), hex::encode(block.hash()));
                println!("{}: {}", "Previous Hash".yellow(), hex::encode(block.header.previous_hash));
                println!("{}: {}", "Merkle Root".yellow(), hex::encode(block.header.merkle_root));
                println!("{}: {}", "Timestamp".yellow(), block.header.timestamp);
                println!("{}: {}", "Difficulty".yellow(), block.header.difficulty);
                println!("{}: {}", "Nonce".yellow(), block.header.nonce);
                println!("{}: {}", "Transactions".yellow(), block.transactions.len());
                println!();

                if !block.transactions.is_empty() {
                    println!("{}", "Transactions:".cyan());
                    for (i, tx) in block.transactions.iter().enumerate() {
                        println!("  {}. {} SYL", i + 1, tx.amount as f64 / 1_000_000.0);
                        println!("     From: {}...", hex::encode(&tx.from.0[..8]));
                        println!("     To:   {}...", hex::encode(&tx.to.0[..8]));
                    }
                    println!();
                }
            } else {
                println!("{}", "Block not found".red());
            }
        }

        Commands::Balance { address } => {
            let node = Node::open(data_dir)?;
            let pk = opensyria_core::crypto::PublicKey::from_hex(&address)?;
            let balance = node.get_balance(&pk)?;

            println!();
            println!("{}: {}...", "Address".cyan(), &address[..16]);
            println!("{}: {} SYL", "Balance".yellow().bold(), balance as f64 / 1_000_000.0);
            println!();
        }

        Commands::ProcessTx { file } => {
            let mut node = Node::open(data_dir)?;
            let json = std::fs::read_to_string(&file)?;
            let tx: opensyria_core::Transaction = serde_json::from_str(&json)?;

            println!("{}", "Processing transaction...".dimmed());
            node.process_transaction(tx)?;
            println!("{}", "✓ Transaction processed successfully".green());
        }

        Commands::Export { output, start, end } => {
            let node = Node::open(data_dir)?;
            let height = node.get_height()?;
            let end = if end == 0 { height } else { end };

            println!("{}", format!("Exporting blocks {} to {}...", start, end).dimmed());

            let blocks = node.get_block_range(start, end)?;
            let json = serde_json::to_string_pretty(&blocks)?;
            std::fs::write(&output, json)?;

            println!(
                "{}",
                format!("✓ Exported {} blocks to {}", blocks.len(), output.display()).green()
            );
        }

        Commands::Network { command } => {
            handle_network_command(command, data_dir).await?;
        }

        Commands::Governance { command } => {
            handle_governance(data_dir, command).await?;
        }

        Commands::Multisig { command } => {
            handle_multisig_command(command, data_dir)?;
        }

        Commands::Pool { command } => {
            handle_pool_command(command, data_dir)?;
        }
    }

    Ok(())
}

async fn handle_network_command(command: NetworkCommands, data_dir: PathBuf) -> Result<()> {
    use opensyria_network::{NetworkNode, NodeConfig, NetworkEvent};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    match command {
        NetworkCommands::Start { listen, bootstrap, mdns } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Starting P2P Network Node  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            // Parse multiaddrs
            let listen_addr = listen.parse()
                .map_err(|e| anyhow::anyhow!("Invalid listen address: {}", e))?;
            
            let bootstrap_peers: Result<Vec<_>> = bootstrap
                .iter()
                .map(|addr| addr.parse().map_err(|e| anyhow::anyhow!("Invalid bootstrap address {}: {}", addr, e)))
                .collect();
            let bootstrap_peers = bootstrap_peers?;

            // Configure network node
            let network_dir = data_dir.join("network");
            let config = NodeConfig {
                listen_addr,
                bootstrap_peers: bootstrap_peers.clone(),
                data_dir: network_dir,
                enable_mdns: mdns,
            };

            println!("{}: {}", "Listen address".cyan(), listen);
            println!("{}: {}", "mDNS discovery".cyan(), if mdns { "enabled" } else { "disabled" });
            if !bootstrap.is_empty() {
                println!("{}: {} peers", "Bootstrap".cyan(), bootstrap.len());
                for peer in &bootstrap {
                    println!("  - {}", peer.dimmed());
                }
            }
            println!();

            // Create network node
            println!("{}", "Initializing network node...".dimmed());
            let (mut node, mut events) = NetworkNode::new(config).await?;

            println!("{}: {}", "Peer ID".yellow(), node.local_peer_id());
            println!();

            // Start listening
            println!("{}", "Starting network listener...".dimmed());
            node.listen(listen.parse()?).await?;
            println!("{}", "✓ Network node started".green());
            println!();

            // Handle events
            let event_handler = tokio::spawn(async move {
                while let Some(event) = events.recv().await {
                    match event {
                        NetworkEvent::PeerConnected(peer_id) => {
                            println!("{} {}", "→ Peer connected:".green(), peer_id);
                        }
                        NetworkEvent::PeerDisconnected(peer_id) => {
                            println!("{} {}", "← Peer disconnected:".yellow(), peer_id);
                        }
                        NetworkEvent::NewBlock(block) => {
                            println!(
                                "{} {} (hash: {}...)",
                                "📦 New block received:".cyan(),
                                "block",
                                hex::encode(&block.hash()[..8])
                            );
                        }
                        NetworkEvent::NewTransaction(tx) => {
                            println!(
                                "{} {} SYL",
                                "💸 New transaction:".cyan(),
                                tx.amount as f64 / 1_000_000.0
                            );
                        }
                        NetworkEvent::ChainTipUpdated { height, hash } => {
                            println!(
                                "{} height={}, hash={}...",
                                "⛓️  Chain tip updated:".yellow(),
                                height,
                                hex::encode(&hash[..8])
                            );
                        }
                        NetworkEvent::SyncProgress { current, target } => {
                            println!(
                                "{} {}/{}",
                                "🔄 Syncing:".dimmed(),
                                current,
                                target
                            );
                        }
                    }
                }
            });

            println!("{}", "Press Ctrl+C to stop".dimmed());
            
            // Run network node
            tokio::select! {
                result = node.run() => {
                    result?;
                }
                _ = tokio::signal::ctrl_c() => {
                    println!();
                    println!("{}", "Shutting down...".yellow());
                }
            }

            event_handler.abort();
            println!("{}", "✓ Network node stopped".green());
        }

        NetworkCommands::Peers => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Connected Peers  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}", "Not implemented: requires persistent node".yellow());
            println!("{}", "Use 'network start' to run a network node".dimmed());
        }

        NetworkCommands::Dial { address } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Connecting to Peer  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}", "Not implemented: requires persistent node".yellow());
            println!("{}", "Use 'network start --bootstrap <addr>' instead".dimmed());
        }

        NetworkCommands::Sync => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Blockchain Synchronization  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}", "Not implemented: requires persistent node".yellow());
            println!("{}", "Use 'network start' to automatically sync".dimmed());
        }

        NetworkCommands::BroadcastBlock { height } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Broadcasting Block  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}", format!("Block height: {}", height).cyan());
            println!();
            println!("{}", "Not implemented: requires persistent node".yellow());
        }

        NetworkCommands::Status => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Network Status  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}", "Not implemented: requires persistent node".yellow());
            println!("{}", "Use 'network start' to run a network node".dimmed());
        }

        NetworkCommands::Daemon {
            listen,
            bootstrap,
            mdns,
            sync_interval,
            mine,
            difficulty,
        } => {
            handle_daemon(data_dir, listen, bootstrap, mdns, sync_interval, mine, difficulty).await?;
        }
    }

    Ok(())
}

async fn handle_daemon(
    data_dir: PathBuf,
    listen: String,
    bootstrap: Vec<String>,
    mdns: bool,
    sync_interval: u64,
    enable_mining: bool,
    difficulty: u32,
) -> Result<()> {
    use tokio::time::{interval, Duration};
    use tokio::signal;

    println!("{}", "═".repeat(60).cyan().bold());
    println!("{}", "  Open Syria Network Daemon  ".cyan().bold());
    println!("{}", "═".repeat(60).cyan().bold());
    println!();

    // Open node
    let mut node = Node::open(data_dir.clone())?;
    let mut chain_height = node.get_blockchain().get_chain_height()?;
    
    println!("{} {}", "📂 Node directory:".bold(), data_dir.display());
    println!("{} {}", "📊 Chain height:".bold(), chain_height.to_string().cyan());
    println!("{} {}", "🌐 Listen address:".bold(), listen.cyan());
    println!("{} {} seconds", "🔄 Sync interval:".bold(), sync_interval.to_string().cyan());
    
    if enable_mining {
        println!("{} {}", "⛏️  Mining:".bold(), "enabled".green());
        println!("{} {}", "💎 Difficulty:".bold(), difficulty.to_string().cyan());
    }
    
    if !bootstrap.is_empty() {
        println!("{} {}", "🔗 Bootstrap peers:".bold(), bootstrap.len().to_string().cyan());
        for peer in &bootstrap {
            println!("   {}", peer.dimmed());
        }
    }
    
    if mdns {
        println!("{} {}", "📡 mDNS:".bold(), "enabled".green());
    }
    
    println!();
    println!("{}", "🚀 Daemon running... (Press Ctrl+C to stop)".green().bold());
    println!();
    println!("{}", "Note: Full P2P networking requires NetworkNode integration".dimmed());
    println!("{}", "This daemon demonstrates auto-mining and periodic checks".dimmed());
    println!();

    // Setup periodic tasks
    let mut status_timer = interval(Duration::from_secs(sync_interval));
    let mut mine_timer = if enable_mining {
        Some(interval(Duration::from_secs(15))) // Mine every 15 seconds
    } else {
        None
    };

    // Main daemon loop
    loop {
        tokio::select! {
            // Show status periodically
            _ = status_timer.tick() => {
                let current_height = node.get_blockchain().get_chain_height()?;
                if current_height != chain_height {
                    println!("{} Chain height: {} → {}", 
                        "📊".cyan(),
                        chain_height.to_string().dimmed(),
                        current_height.to_string().green()
                    );
                    chain_height = current_height;
                } else {
                    println!("{} Status check - height: {}, pending txs: {}", 
                        "💫".dimmed(),
                        chain_height.to_string().cyan(),
                        node.get_pending_transactions().len().to_string().dimmed()
                    );
                }
            }

            // Mine blocks if enabled
            _ = async {
                if let Some(ref mut timer) = mine_timer {
                    timer.tick().await
                } else {
                    std::future::pending().await
                }
            } => {
                match mine_block(&mut node, difficulty).await {
                    Ok(Some(block)) => {
                        let height = node.get_blockchain().get_chain_height()?;
                        println!("{} Mined block at height {} with {} tx(s) ({})", 
                            "⛏️ ".green(), 
                            height.to_string().cyan(),
                            block.transactions.len().to_string().yellow(),
                            hex::encode(&block.hash()[..8]).dimmed()
                        );
                        chain_height = height;
                    }
                    Ok(None) => {
                        println!("{} No transactions to mine", "⛏️ ".dimmed());
                    }
                    Err(e) => {
                        println!("{} Mining error: {}", "⚠️ ".yellow(), e);
                    }
                }
            }

            // Handle Ctrl+C
            _ = signal::ctrl_c() => {
                println!();
                println!("{}", "Shutting down daemon...".yellow());
                break;
            }
        }
    }

    let final_height = node.get_blockchain().get_chain_height()?;
    println!();
    println!("{} {}", "Final height:".bold(), final_height.to_string().green());
    println!("{}", "✓ Daemon stopped".green());
    Ok(())
}

async fn mine_block(node: &mut Node, difficulty: u32) -> Result<Option<opensyria_core::Block>> {
    use opensyria_consensus::ProofOfWork;
    use opensyria_core::Block;
    
    // Get pending transactions
    let transactions = node.get_pending_transactions();
    if transactions.is_empty() {
        return Ok(None); // Nothing to mine
    }
    
    // Take up to 100 transactions
    let txs: Vec<_> = transactions.into_iter().take(100).collect();
    
    let blockchain = node.get_blockchain();
    let tip_hash = blockchain.get_chain_tip()?.ok_or_else(|| anyhow::anyhow!("No chain tip"))?;
    
    // Create block
    let block = Block::new(tip_hash, txs, difficulty);
    
    // Mine block (use low difficulty for daemon to avoid blocking too long)
    let pow = ProofOfWork::new(difficulty.min(16)); // Cap at 16 for daemon
    let (mined_block, _stats) = pow.mine(block);
    
    // Append block
    blockchain.append_block(&mined_block)?;
    
    // Update state for block transactions
    for tx in &mined_block.transactions {
        let _ = node.get_state().transfer(&tx.from, &tx.to, tx.amount);
        let _ = node.get_state().sub_balance(&tx.from, tx.fee);
        let _ = node.get_state().increment_nonce(&tx.from);
    }
    
    Ok(Some(mined_block))
}

async fn handle_governance(data_dir: PathBuf, command: GovernanceCommands) -> Result<()> {
    use opensyria_governance::{ProposalStatus, ProposalType, Vote};

    match command {
        GovernanceCommands::Propose {
            title,
            description,
            proposal_type,
            proposer,
        } => {
            let node = Node::open(data_dir)?;

            // Parse proposer public key
            let proposer_key = PublicKey::from_hex(&proposer)
                .map_err(|e| anyhow::anyhow!("Invalid proposer address: {:?}", e))?;

            // Parse proposal type
            let prop_type = match proposal_type.as_str() {
                "text" => ProposalType::TextProposal {
                    description: description.clone(),
                },
                "min-fee" => ProposalType::MinimumFee { new_fee: 200 },
                _ => {
                    anyhow::bail!("Unknown proposal type: {}\nAvailable types: text, min-fee", proposal_type);
                }
            };

            let proposal_id = node.create_proposal(proposer_key, prop_type, title.clone(), description)?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Proposal Created  ".green().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Proposal ID".yellow(), proposal_id);
            println!("{}: {}", "Title".yellow(), title);
            println!("{}: {}", "Proposer".yellow(), proposer);
            println!();
        }

        GovernanceCommands::Vote {
            proposal_id,
            choice,
            voter,
        } => {
            let node = Node::open(data_dir)?;

            // Parse voter public key
            let voter_key = PublicKey::from_hex(&voter)
                .map_err(|e| anyhow::anyhow!("Invalid voter address: {:?}", e))?;

            // Parse vote choice
            let vote = match choice.to_lowercase().as_str() {
                "yes" | "y" => Vote::Yes,
                "no" | "n" => Vote::No,
                "abstain" | "a" => Vote::Abstain,
                _ => anyhow::bail!("Invalid vote choice: {}\nValid choices: yes, no, abstain", choice),
            };

            node.vote_on_proposal(proposal_id, voter_key, vote)?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Vote Recorded  ".green().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Proposal ID".yellow(), proposal_id);
            println!("{}: {:?}", "Vote".yellow(), vote);
            println!("{}: {}", "Voter".yellow(), voter);
            println!();
        }

        GovernanceCommands::List { status } => {
            let node = Node::open(data_dir)?;
            let manager = node.load_governance()?;
            let proposals = manager.get_all_proposals();

            if proposals.is_empty() {
                println!("{}", "No proposals found.".yellow());
                return Ok(());
            }

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Governance Proposals  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            for proposal in proposals {
                // Filter by status if specified
                if let Some(ref status_filter) = status {
                    let matches = match status_filter.to_lowercase().as_str() {
                        "active" => proposal.status == ProposalStatus::Active,
                        "passed" => proposal.status == ProposalStatus::Passed,
                        "rejected" => proposal.status == ProposalStatus::Rejected,
                        "executed" => proposal.status == ProposalStatus::Executed,
                        "cancelled" => proposal.status == ProposalStatus::Cancelled,
                        _ => true,
                    };

                    if !matches {
                        continue;
                    }
                }

                println!("{}: {}", "ID".bold(), proposal.id);
                println!("{}: {}", "Title".bold(), proposal.title);
                println!("{}: {:?}", "Status".bold(), proposal.status);
                println!("{}: {}%", "Participation".bold(), proposal.participation_rate());
                println!("{}: {}%", "Yes Votes".bold(), proposal.yes_percentage());
                println!("{}", "-".repeat(60));
            }
            println!();
        }

        GovernanceCommands::Show { proposal_id } => {
            let node = Node::open(data_dir)?;
            let manager = node.load_governance()?;

            let proposal = manager
                .get_proposal(proposal_id)
                .ok_or_else(|| anyhow::anyhow!("Proposal {} not found", proposal_id))?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Proposal Details  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "ID".bold(), proposal.id);
            println!("{}: {}", "Title".bold(), proposal.title);
            println!("{}: {}", "Description".bold(), proposal.description);
            println!("{}: {:?}", "Type".bold(), proposal.proposal_type);
            println!("{}: {:?}", "Status".bold(), proposal.status);
            println!();
            println!("{}: {}", "Created".bold(), proposal.created_at);
            println!("{}: {}", "Voting Start".bold(), proposal.voting_start);
            println!("{}: {}", "Voting End".bold(), proposal.voting_end);
            println!("{}: {} blocks", "Execution Delay".bold(), proposal.execution_delay);
            println!();
            println!("{}: {}%", "Required Quorum".bold(), proposal.required_quorum);
            println!("{}: {}%", "Required Threshold".bold(), proposal.required_threshold);
            println!();
            println!("{}: {}", "Yes Votes".bold(), proposal.votes_yes);
            println!("{}: {}", "No Votes".bold(), proposal.votes_no);
            println!("{}: {}", "Abstain Votes".bold(), proposal.votes_abstain);
            println!("{}: {}%", "Participation".bold(), proposal.participation_rate());
            println!("{}: {}%", "Yes Percentage".bold(), proposal.yes_percentage());
            println!();
            println!(
                "{}: {}",
                "Meets Quorum".bold(),
                if proposal.meets_quorum() { "Yes".green() } else { "No".red() }
            );
            println!(
                "{}: {}",
                "Meets Threshold".bold(),
                if proposal.meets_threshold() { "Yes".green() } else { "No".red() }
            );
            println!();

            let votes = manager.get_proposal_votes(proposal_id);
            if !votes.is_empty() {
                println!("{}", "Votes Cast:".cyan().bold());
                println!("{}", "-".repeat(60));
                for vote in votes.iter().take(10) {
                    println!("  {:?} - Power: {} - Block: {}", vote.vote, vote.voting_power, vote.timestamp);
                }
                if votes.len() > 10 {
                    println!("  ... and {} more votes", votes.len() - 10);
                }
                println!();
            }
        }

        GovernanceCommands::Stats => {
            let node = Node::open(data_dir)?;
            let manager = node.load_governance()?;
            let stats = manager.get_statistics();

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Governance Statistics  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Total Proposals".bold(), stats.total_proposals);
            println!("{}: {}", "Active Proposals".bold(), stats.active_proposals);
            println!("{}: {}", "Passed Proposals".bold(), stats.passed_proposals);
            println!("{}: {}", "Rejected Proposals".bold(), stats.rejected_proposals);
            println!("{}: {}", "Executed Proposals".bold(), stats.executed_proposals);
            println!("{}: {}", "Cancelled Proposals".bold(), stats.cancelled_proposals);
            println!("{}: {}", "Total Votes Cast".bold(), stats.total_votes_cast);
            println!();

            let config = manager.config();
            println!("{}", "Configuration:".cyan().bold());
            println!("{}", "-".repeat(60));
            println!("{}: {} Lira", "Min Proposal Stake".bold(), config.min_proposal_stake / 1_000_000);
            println!("{}: {} blocks", "Default Voting Period".bold(), config.default_voting_period);
            println!("{}: {} blocks", "Default Execution Delay".bold(), config.default_execution_delay);
            println!("{}: {}", "Enabled".bold(), if config.enabled { "Yes" } else { "No" });
            println!();
        }

        GovernanceCommands::Process => {
            let node = Node::open(data_dir)?;
            let finalized = node.process_proposals()?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Proposals Processed  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Finalized proposals".yellow(), finalized);
            println!();
        }
    }

    Ok(())
}

fn handle_multisig_command(command: MultisigCommands, data_dir: PathBuf) -> Result<()> {
    use opensyria_core::multisig::{MultisigAccount, MultisigTransaction};
    use opensyria_core::crypto::{PublicKey, KeyPair};
    use std::fs;

    match command {
        MultisigCommands::Create { signer, threshold, balance } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Creating Multisig Account  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            // Parse signer public keys
            let signers: Result<Vec<PublicKey>> = signer
                .iter()
                .map(|s| PublicKey::from_hex(s).map_err(|e| anyhow::anyhow!("Invalid signer key: {}", e)))
                .collect();
            let signers = signers?;

            // Create multisig account
            let account = MultisigAccount::new(signers.clone(), threshold)?;
            let address = account.address();

            // Store account configuration
            let node = Node::open(data_dir)?;
            node.get_state().store_multisig_account(&account)?;

            // Fund account if balance specified
            if balance > 0 {
                node.get_state().set_balance(&address, balance)?;
                println!("{}: {} Lira", "Initial balance".green(), balance as f64 / 1_000_000.0);
            }

            println!("{}: {}", "Address".bold(), hex::encode(address.0));
            println!("{}: {}", "Signers".bold(), signers.len());
            println!("{}: {}", "Threshold".bold(), threshold);
            println!();
            println!("{}", "Signer addresses:".cyan());
            for (i, signer) in signers.iter().enumerate() {
                println!("  {}. {}", i + 1, hex::encode(signer.0));
            }
            println!();
            println!("{}", "✓ Multisig account created successfully".green());
            println!();
        }

        MultisigCommands::Info { address } => {
            let node = Node::open(data_dir)?;
            let addr = PublicKey::from_hex(&address)?;

            match node.get_state().get_multisig_account(&addr)? {
                Some(account) => {
                    let balance = node.get_state().get_balance(&addr)?;
                    let nonce = node.get_state().get_nonce(&addr)?;

                    println!("{}", "═".repeat(60).cyan());
                    println!("{}", "  Multisig Account Details  ".cyan().bold());
                    println!("{}", "═".repeat(60).cyan());
                    println!();
                    println!("{}: {}", "Address".bold(), hex::encode(addr.0));
                    println!("{}: {} Lira", "Balance".bold(), balance as f64 / 1_000_000.0);
                    println!("{}: {}", "Nonce".bold(), nonce);
                    println!("{}: {}-of-{}", "Threshold".bold(), account.threshold, account.num_signers());
                    println!();
                    println!("{}", "Authorized signers:".cyan());
                    for (i, signer) in account.signers.iter().enumerate() {
                        println!("  {}. {}", i + 1, hex::encode(signer.0));
                    }
                    println!();
                }
                None => {
                    println!("{}", "Error: Not a multisig account".red());
                }
            }
        }

        MultisigCommands::CreateTx { from, to, amount, fee, output } => {
            let node = Node::open(data_dir)?;
            let from_addr = PublicKey::from_hex(&from)?;
            let to_addr = PublicKey::from_hex(&to)?;

            // Load multisig account
            let account = node.get_state().get_multisig_account(&from_addr)?
                .ok_or_else(|| anyhow::anyhow!("Not a multisig account"))?;

            // Get nonce
            let nonce = node.get_state().get_nonce(&from_addr)?;

            // Create unsigned transaction
            let tx = MultisigTransaction::new(account, to_addr, amount, fee, nonce);

            // Serialize to file
            let json = serde_json::to_string_pretty(&tx)?;
            fs::write(&output, json)?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Multisig Transaction Created  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "From".bold(), hex::encode(from_addr.0));
            println!("{}: {}", "To".bold(), hex::encode(to_addr.0));
            println!("{}: {} Lira", "Amount".bold(), amount as f64 / 1_000_000.0);
            println!("{}: {} Lira", "Fee".bold(), fee as f64 / 1_000_000.0);
            println!("{}: {}", "Nonce".bold(), nonce);
            println!();
            println!("{}: {}", "Output file".green(), output.display());
            println!();
            println!("{}", "Next steps:".yellow());
            println!("  1. Send this file to signers for signing");
            println!("  2. Each signer runs: multisig sign --tx-file {} --private-key <key> --output <signed>", output.display());
            println!("  3. Submit when threshold met: multisig submit --tx-file <signed>");
            println!();
        }

        MultisigCommands::Sign { tx_file, private_key, output } => {
            // Load transaction
            let json = fs::read_to_string(&tx_file)?;
            let mut tx: MultisigTransaction = serde_json::from_str(&json)?;

            // Parse private key and derive public key
            let key_bytes = hex::decode(&private_key)?;
            if key_bytes.len() != 32 {
                anyhow::bail!("Invalid private key length");
            }
            let mut key_array = [0u8; 32];
            key_array.copy_from_slice(&key_bytes);
            
            // Create keypair (this is a simplified approach - in production, use proper key derivation)
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_array);
            let verifying_key = signing_key.verifying_key();
            let public_key = PublicKey(verifying_key.to_bytes());

            // Sign transaction
            let msg = tx.signing_hash();
            let signature = signing_key.sign(&msg).to_bytes().to_vec();
            
            tx.add_signature(public_key, signature)?;

            // Save signed transaction
            let json = serde_json::to_string_pretty(&tx)?;
            fs::write(&output, json)?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Transaction Signed  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Signer".bold(), hex::encode(public_key.0));
            println!("{}: {}/{}", "Signatures".bold(), tx.signatures.len(), tx.account.threshold);
            println!("{}: {}", "Ready to submit".bold(), if tx.is_ready() { "Yes ✓".green() } else { "No (need more signatures)".yellow() });
            println!();
            println!("{}: {}", "Output file".green(), output.display());
            println!();
        }

        MultisigCommands::Submit { tx_file } => {
            let node = Node::open(data_dir)?;
            
            // Load transaction
            let json = fs::read_to_string(&tx_file)?;
            let tx: MultisigTransaction = serde_json::from_str(&json)?;

            // Verify transaction
            tx.verify()?;

            // Check balance
            let from = tx.from();
            let balance = node.get_state().get_balance(&from)?;
            let required = tx.amount + tx.fee;
            
            if balance < required {
                anyhow::bail!("Insufficient balance: {} required, {} available", 
                    required as f64 / 1_000_000.0,
                    balance as f64 / 1_000_000.0
                );
            }

            // Execute transaction
            node.get_state().transfer(&from, &tx.to, required)?;
            node.get_state().increment_nonce(&from)?;

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Multisig Transaction Submitted  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Transaction hash".bold(), hex::encode(tx.hash()));
            println!("{}: {}", "From".bold(), hex::encode(from.0));
            println!("{}: {}", "To".bold(), hex::encode(tx.to.0));
            println!("{}: {} Lira", "Amount".bold(), tx.amount as f64 / 1_000_000.0);
            println!("{}: {} Lira", "Fee".bold(), tx.fee as f64 / 1_000_000.0);
            println!("{}: {}", "Signatures".bold(), tx.signatures.len());
            println!();
            println!("{}", "✓ Transaction executed successfully".green());
            println!();
        }
    }

    Ok(())
}

fn handle_pool_command(command: PoolCommands, data_dir: PathBuf) -> Result<()> {
    use opensyria_mining_pool::{MiningPool, PoolConfig, RewardMethod, Share};
    use opensyria_core::crypto::PublicKey;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let pool_file = data_dir.join("mining_pool.json");

    match command {
        PoolCommands::Init { operator, fee, share_difficulty, method } => {
            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Initializing Mining Pool  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            let operator_key = PublicKey::from_hex(&operator)?;
            
            let reward_method = match method.as_str() {
                "proportional" => RewardMethod::Proportional,
                "pps" => RewardMethod::PPS,
                "pplns" => RewardMethod::PPLNS { window: 100 },
                _ => anyhow::bail!("Invalid reward method. Use: proportional, pps, or pplns"),
            };

            let config = PoolConfig {
                operator: operator_key,
                fee_percent: fee,
                min_payout: 1_000_000, // 1 Lira
                share_difficulty,
                reward_method,
                server_address: "0.0.0.0:3333".to_string(),
            };

            let pool = MiningPool::new(config.clone());

            // Save pool configuration
            let json = serde_json::to_string_pretty(&config)?;
            fs::write(&pool_file, json)?;

            println!("{}: {}", "Operator".bold(), hex::encode(operator_key.0));
            println!("{}: {}%", "Pool Fee".bold(), fee);
            println!("{}: {}", "Share Difficulty".bold(), share_difficulty);
            println!("{}: {:?}", "Reward Method".bold(), reward_method);
            println!();
            println!("{}", "✓ Mining pool initialized successfully".green());
            println!("{}: {}", "Config saved to".green(), pool_file.display());
            println!();
        }

        PoolCommands::Stats => {
            if !pool_file.exists() {
                anyhow::bail!("Mining pool not initialized. Run: pool init");
            }

            let json = fs::read_to_string(&pool_file)?;
            let config: PoolConfig = serde_json::from_str(&json)?;
            let pool = MiningPool::new(config);
            let stats = pool.get_stats();

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Mining Pool Statistics  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Active Miners".bold(), stats.active_miners);
            println!("{}: {:.2} MH/s", "Pool Hashrate".bold(), stats.pool_hashrate / 1_000_000.0);
            println!("{}: {}", "Blocks Mined".bold(), stats.blocks_mined);
            println!("{}: {}", "Current Difficulty".bold(), stats.current_difficulty);
            println!("{}: {}", "Current Round Shares".bold(), stats.current_round_shares);
            println!("{}: {}%", "Pool Fee".bold(), stats.pool_fee);
            println!();
        }

        PoolCommands::Miners => {
            if !pool_file.exists() {
                anyhow::bail!("Mining pool not initialized. Run: pool init");
            }

            let json = fs::read_to_string(&pool_file)?;
            let config: PoolConfig = serde_json::from_str(&json)?;
            let pool = MiningPool::new(config);
            let miners = pool.get_all_miners();

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Pool Miners  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            if miners.is_empty() {
                println!("{}", "No miners registered yet".yellow());
                println!();
                return Ok(());
            }

            for (i, miner) in miners.iter().enumerate() {
                println!("{}. {}", i + 1, hex::encode(miner.miner.0));
                println!("   Valid Shares: {}", miner.valid_shares);
                println!("   Invalid Shares: {}", miner.invalid_shares);
                println!("   Hashrate: {:.2} MH/s", miner.hashrate / 1_000_000.0);
                println!("   Total Rewards: {} Lira", miner.total_rewards as f64 / 1_000_000.0);
                println!("   Pending: {} Lira", miner.pending_rewards as f64 / 1_000_000.0);
                println!();
            }
        }

        PoolCommands::Miner { address } => {
            if !pool_file.exists() {
                anyhow::bail!("Mining pool not initialized. Run: pool init");
            }

            let json = fs::read_to_string(&pool_file)?;
            let config: PoolConfig = serde_json::from_str(&json)?;
            let pool = MiningPool::new(config);
            
            let miner_key = PublicKey::from_hex(&address)?;
            
            match pool.get_miner_stats(&miner_key) {
                Some(stats) => {
                    println!("{}", "═".repeat(60).cyan());
                    println!("{}", "  Miner Statistics  ".cyan().bold());
                    println!("{}", "═".repeat(60).cyan());
                    println!();
                    println!("{}: {}", "Address".bold(), hex::encode(stats.miner.0));
                    println!("{}: {}", "Total Shares".bold(), stats.total_shares);
                    println!("{}: {}", "Valid Shares".bold(), stats.valid_shares);
                    println!("{}: {}", "Invalid Shares".bold(), stats.invalid_shares);
                    println!("{}: {:.2} MH/s", "Hashrate".bold(), stats.hashrate / 1_000_000.0);
                    println!("{}: {} Lira", "Total Rewards".bold(), stats.total_rewards as f64 / 1_000_000.0);
                    println!("{}: {} Lira", "Pending Rewards".bold(), stats.pending_rewards as f64 / 1_000_000.0);
                    println!();
                }
                None => {
                    println!("{}", "Miner not found in pool".red());
                }
            }
        }

        PoolCommands::Register { address } => {
            if !pool_file.exists() {
                anyhow::bail!("Mining pool not initialized. Run: pool init");
            }

            let json = fs::read_to_string(&pool_file)?;
            let config: PoolConfig = serde_json::from_str(&json)?;
            let mut pool = MiningPool::new(config.clone());
            
            let miner_key = PublicKey::from_hex(&address)?;
            pool.register_miner(miner_key);

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Miner Registered  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();
            println!("{}: {}", "Address".bold(), hex::encode(miner_key.0));
            println!();
            println!("{}", "✓ Miner registered successfully".green());
            println!();
            println!("{}", "Next steps:".yellow());
            println!("  1. Start your miner pointing to pool server");
            println!("  2. Monitor stats: pool miner {}", hex::encode(miner_key.0));
            println!();
        }

        PoolCommands::Payout { miner } => {
            if !pool_file.exists() {
                anyhow::bail!("Mining pool not initialized. Run: pool init");
            }

            let json = fs::read_to_string(&pool_file)?;
            let config: PoolConfig = serde_json::from_str(&json)?;
            let mut pool = MiningPool::new(config);

            println!("{}", "═".repeat(60).cyan());
            println!("{}", "  Processing Payouts  ".cyan().bold());
            println!("{}", "═".repeat(60).cyan());
            println!();

            if let Some(addr) = miner {
                let miner_key = PublicKey::from_hex(&addr)?;
                
                match pool.process_payout(&miner_key) {
                    Ok(amount) => {
                        println!("{}: {}", "Miner".bold(), hex::encode(miner_key.0));
                        println!("{}: {} Lira", "Payout".bold(), amount as f64 / 1_000_000.0);
                        println!();
                        println!("{}", "✓ Payout processed".green());
                    }
                    Err(e) => {
                        println!("{}: {}", "Error".red(), e);
                    }
                }
            } else {
                // Process all miners
                let miner_keys: Vec<PublicKey> = pool.get_all_miners()
                    .into_iter()
                    .map(|m| m.miner)
                    .collect();
                    
                let mut total_paid = 0u64;
                let mut count = 0;

                for miner_key in miner_keys {
                    if let Ok(amount) = pool.process_payout(&miner_key) {
                        println!("{}: {} Lira", 
                            hex::encode(&miner_key.0[..8]), 
                            amount as f64 / 1_000_000.0
                        );
                        total_paid += amount;
                        count += 1;
                    }
                }

                println!();
                println!("{}: {}", "Miners Paid".bold(), count);
                println!("{}: {} Lira", "Total Paid".bold(), total_paid as f64 / 1_000_000.0);
                println!();
            }
        }
    }

    Ok(())
}
