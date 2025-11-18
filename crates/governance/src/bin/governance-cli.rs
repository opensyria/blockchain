use clap::{Parser, Subcommand};
use colored::Colorize;
use opensyria_core::crypto::KeyPair;
use opensyria_governance::{
    GovernanceConfig, GovernanceManager, GovernanceStorage, ProposalType, Vote,
};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "governance-cli")]
#[command(about = "Open Syria Governance CLI | إدارة الحوكمة", long_about = None)]
struct Cli {
    /// Data directory
    #[arg(short, long, default_value = "data/governance")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize governance system
    Init,

    /// Create a new proposal
    Propose {
        /// Proposal title
        #[arg(long)]
        title: String,

        /// Proposal description
        #[arg(long)]
        description: String,

        /// Proposal type
        #[arg(short = 't', long)]
        proposal_type: String,
    },

    /// List all proposals
    List {
        /// Filter by status: active, passed, rejected, executed, cancelled
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Show proposal details
    Show {
        /// Proposal ID
        proposal_id: u64,
    },

    /// Vote on a proposal
    Vote {
        /// Proposal ID
        proposal_id: u64,

        /// Vote choice: yes, no, abstain
        #[arg(short, long)]
        choice: String,
    },

    /// Show governance statistics
    Stats,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init_governance(&cli.data_dir),
        Commands::Propose {
            title,
            description,
            proposal_type,
        } => create_proposal(&cli.data_dir, title, description, proposal_type),
        Commands::List { status } => list_proposals(&cli.data_dir, status),
        Commands::Show { proposal_id } => show_proposal(&cli.data_dir, proposal_id),
        Commands::Vote {
            proposal_id,
            choice,
        } => vote_on_proposal(&cli.data_dir, proposal_id, choice),
        Commands::Stats => show_stats(&cli.data_dir),
    }
}

fn init_governance(data_dir: &PathBuf) {
    println!("{}", "Initializing governance system...".cyan());

    let storage = match GovernanceStorage::open(data_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} {}", "Error opening storage:".red(), e);
            return;
        }
    };

    let config = GovernanceConfig::default();
    let manager = GovernanceManager::new(config);
    let snapshot = manager.create_snapshot();

    if let Err(e) = storage.save_snapshot(&snapshot) {
        eprintln!("{} {}", "Error saving snapshot:".red(), e);
        return;
    }

    println!("{}", "✓ Governance system initialized".green());
    println!("Data directory: {}", data_dir.display());
}

fn load_manager(data_dir: &PathBuf) -> Result<GovernanceManager, String> {
    let storage = GovernanceStorage::open(data_dir).map_err(|e| format!("Storage error: {}", e))?;

    if !storage.has_snapshot().unwrap_or(false) {
        return Err("Governance not initialized. Run 'init' first.".to_string());
    }

    let snapshot = storage
        .load_snapshot()
        .map_err(|e| format!("Failed to load snapshot: {}", e))?;

    Ok(GovernanceManager::from_snapshot(snapshot))
}

fn create_proposal(data_dir: &PathBuf, title: String, description: String, type_str: String) {
    let mut manager = match load_manager(data_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            return;
        }
    };

    // Parse proposal type
    let proposal_type = match type_str.as_str() {
        "text" => ProposalType::TextProposal {
            description: description.clone(),
        },
        "min-fee" => ProposalType::MinimumFee { new_fee: 200 },
        _ => {
            eprintln!("{} Unknown proposal type: {}", "Error:".red(), type_str);
            eprintln!("Available types: text, min-fee");
            return;
        }
    };

    let proposer = KeyPair::generate(); // In real use, load from wallet
    let current_height = 1000; // In real use, get from blockchain
    let total_voting_power = 100_000_000_000; // In real use, get from state

    match manager.create_proposal(
        proposer.public_key(),
        2_000_000_000, // Proposer stake
        proposal_type,
        title.clone(),
        description,
        current_height,
        total_voting_power,
    ) {
        Ok(id) => {
            println!("{}", "✓ Proposal created successfully".green());
            println!("Proposal ID: {}", id);
            println!("Title: {}", title);

            // Save state
            let storage = GovernanceStorage::open(data_dir).unwrap();
            let snapshot = manager.create_snapshot();
            let _ = storage.save_snapshot(&snapshot);
        }
        Err(e) => {
            eprintln!("{} {}", "Error creating proposal:".red(), e);
        }
    }
}

fn list_proposals(data_dir: &PathBuf, status_filter: Option<String>) {
    let manager = match load_manager(data_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            return;
        }
    };

    let proposals = manager.get_all_proposals();

    if proposals.is_empty() {
        println!("{}", "No proposals found.".yellow());
        return;
    }

    println!("\n{}", "Governance Proposals".cyan().bold());
    println!("{}", "=".repeat(80));

    for proposal in proposals {
        // Filter by status if specified
        if let Some(ref status) = status_filter {
            let matches = match status.to_lowercase().as_str() {
                "active" => proposal.status == opensyria_governance::ProposalStatus::Active,
                "passed" => proposal.status == opensyria_governance::ProposalStatus::Passed,
                "rejected" => proposal.status == opensyria_governance::ProposalStatus::Rejected,
                "executed" => proposal.status == opensyria_governance::ProposalStatus::Executed,
                "cancelled" => proposal.status == opensyria_governance::ProposalStatus::Cancelled,
                _ => true,
            };

            if !matches {
                continue;
            }
        }

        println!("\n{} {}", "ID:".bold(), proposal.id);
        println!("{} {}", "Title:".bold(), proposal.title);
        println!("{} {:?}", "Status:".bold(), proposal.status);
        println!(
            "{} {}%",
            "Participation:".bold(),
            proposal.participation_rate()
        );
        println!(
            "{} {}% (need {}%)",
            "Yes votes:".bold(),
            proposal.yes_percentage(),
            proposal.required_threshold
        );
        println!("{}", "-".repeat(80));
    }
}

fn show_proposal(data_dir: &PathBuf, proposal_id: u64) {
    let manager = match load_manager(data_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            return;
        }
    };

    let proposal = match manager.get_proposal(proposal_id) {
        Some(p) => p,
        None => {
            eprintln!("{} Proposal {} not found", "Error:".red(), proposal_id);
            return;
        }
    };

    println!("\n{}", "Proposal Details".cyan().bold());
    println!("{}", "=".repeat(80));
    println!("{} {}", "ID:".bold(), proposal.id);
    println!("{} {}", "Title:".bold(), proposal.title);
    println!("{} {}", "Description:".bold(), proposal.description);
    println!("{} {:?}", "Type:".bold(), proposal.proposal_type);
    println!("{} {:?}", "Status:".bold(), proposal.status);
    println!();
    println!("{} {}", "Created at block:".bold(), proposal.created_at);
    println!("{} {}", "Voting starts:".bold(), proposal.voting_start);
    println!("{} {}", "Voting ends:".bold(), proposal.voting_end);
    println!(
        "{} {} blocks",
        "Execution delay:".bold(),
        proposal.execution_delay
    );
    println!();
    println!(
        "{} {}%",
        "Required quorum:".bold(),
        proposal.required_quorum
    );
    println!(
        "{} {}%",
        "Required threshold:".bold(),
        proposal.required_threshold
    );
    println!();
    println!("{} {}", "Yes votes:".bold(), proposal.votes_yes);
    println!("{} {}", "No votes:".bold(), proposal.votes_no);
    println!("{} {}", "Abstain votes:".bold(), proposal.votes_abstain);
    println!(
        "{} {}%",
        "Participation:".bold(),
        proposal.participation_rate()
    );
    println!(
        "{} {}%",
        "Yes percentage:".bold(),
        proposal.yes_percentage()
    );
    println!();
    println!(
        "{} {}",
        "Meets quorum:".bold(),
        if proposal.meets_quorum() {
            "Yes".green()
        } else {
            "No".red()
        }
    );
    println!(
        "{} {}",
        "Meets threshold:".bold(),
        if proposal.meets_threshold() {
            "Yes".green()
        } else {
            "No".red()
        }
    );

    let votes = manager.get_proposal_votes(proposal_id);
    if !votes.is_empty() {
        println!("\n{}", "Votes Cast".cyan().bold());
        println!("{}", "-".repeat(80));
        for vote in votes {
            println!(
                "  {:?} - Power: {} - Block: {}",
                vote.vote, vote.voting_power, vote.timestamp
            );
        }
    }
}

fn vote_on_proposal(data_dir: &PathBuf, proposal_id: u64, choice: String) {
    let mut manager = match load_manager(data_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            return;
        }
    };

    let vote = match choice.to_lowercase().as_str() {
        "yes" | "y" => Vote::Yes,
        "no" | "n" => Vote::No,
        "abstain" | "a" => Vote::Abstain,
        _ => {
            eprintln!("{} Invalid vote choice: {}", "Error:".red(), choice);
            eprintln!("Valid choices: yes, no, abstain");
            return;
        }
    };

    let voter = KeyPair::generate(); // In real use, load from wallet
    let voting_power = 1_000_000; // In real use, get from state
    let current_height = 1500; // In real use, get from blockchain

    match manager.vote(
        proposal_id,
        voter.public_key(),
        vote,
        voting_power,
        current_height,
    ) {
        Ok(_) => {
            println!("{}", "✓ Vote recorded successfully".green());
            println!("Proposal ID: {}", proposal_id);
            println!("Vote: {:?}", vote);
            println!("Voting power: {}", voting_power);

            // Save state
            let storage = GovernanceStorage::open(data_dir).unwrap();
            let snapshot = manager.create_snapshot();
            let _ = storage.save_snapshot(&snapshot);
        }
        Err(e) => {
            eprintln!("{} {}", "Error voting:".red(), e);
        }
    }
}

fn show_stats(data_dir: &PathBuf) {
    let manager = match load_manager(data_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            return;
        }
    };

    let stats = manager.get_statistics();

    println!("\n{}", "Governance Statistics".cyan().bold());
    println!("{}", "=".repeat(80));
    println!("{} {}", "Total proposals:".bold(), stats.total_proposals);
    println!("{} {}", "Active proposals:".bold(), stats.active_proposals);
    println!("{} {}", "Passed proposals:".bold(), stats.passed_proposals);
    println!(
        "{} {}",
        "Rejected proposals:".bold(),
        stats.rejected_proposals
    );
    println!(
        "{} {}",
        "Executed proposals:".bold(),
        stats.executed_proposals
    );
    println!(
        "{} {}",
        "Cancelled proposals:".bold(),
        stats.cancelled_proposals
    );
    println!("{} {}", "Total votes cast:".bold(), stats.total_votes_cast);
    println!();

    let config = manager.config();
    println!("{}", "Configuration".cyan().bold());
    println!("{}", "-".repeat(80));
    println!(
        "{} {} Lira",
        "Min proposal stake:".bold(),
        config.min_proposal_stake / 1_000_000
    );
    println!(
        "{} {} blocks",
        "Default voting period:".bold(),
        config.default_voting_period
    );
    println!(
        "{} {} blocks",
        "Default execution delay:".bold(),
        config.default_execution_delay
    );
    println!(
        "{} {}",
        "Governance enabled:".bold(),
        if config.enabled { "Yes" } else { "No" }
    );
}
