use clap::Parser;
use colored::*;
use opensyria_consensus::{MiningStats, ProofOfWork};
use opensyria_core::Block;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "miner")]
#[command(about = "OpenSyria PoW Miner | منقّب الليرة الرقمية", long_about = None)]
struct Cli {
    /// Mining difficulty (number of leading zero bits)
    #[arg(short, long, default_value = "16")]
    difficulty: u32,

    /// Number of blocks to mine
    #[arg(short, long, default_value = "1")]
    blocks: u32,

    /// Show detailed progress
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    println!("{}", "═".repeat(60).cyan());
    println!(
        "{}",
        "  OpenSyria Proof-of-Work Miner | منقّب البرهان على العمل  "
            .cyan()
            .bold()
    );
    println!("{}", "═".repeat(60).cyan());
    println!();

    println!("{}: {}", "Difficulty | الصعوبة".yellow(), cli.difficulty);
    println!("{}: {}", "Target Blocks | عدد الكتل".yellow(), cli.blocks);
    println!();

    let pow = ProofOfWork::new(cli.difficulty);
    let mut previous_hash = [0u8; 32]; // Genesis
    let mut total_time = Duration::ZERO;
    let mut total_hashes = 0u64;

    for block_num in 1..=cli.blocks {
        println!(
            "{} {}",
            "▶ Mining Block | تعدين الكتلة".green().bold(),
            block_num
        );

        let block = Block::new(previous_hash, vec![], cli.difficulty);

        let (mined_block, stats) = if cli.verbose {
            pow.mine_with_callback(block, |hashes, rate| {
                print!(
                    "\r  {} {} | {} {:.2} H/s",
                    "Hashes | الهاشات:".dimmed(),
                    format!("{:>12}", hashes).white(),
                    "Rate | المعدل:".dimmed(),
                    rate
                );
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            })
        } else {
            pow.mine(block)
        };

        if cli.verbose {
            println!(); // New line after progress
        }

        total_time += stats.duration;
        total_hashes += stats.hashes_computed;
        previous_hash = mined_block.hash();

        print_mining_result(block_num, &mined_block, &stats);
        println!();
    }

    // Final summary
    println!("{}", "═".repeat(60).cyan());
    println!("{}", "  Mining Summary | ملخص التعدين  ".cyan().bold());
    println!("{}", "═".repeat(60).cyan());
    println!();
    println!(
        "{}: {} blocks",
        "Total Mined | إجمالي الكتل".yellow(),
        cli.blocks
    );
    println!(
        "{}: {:.2}s",
        "Total Time | الوقت الكلي".yellow(),
        total_time.as_secs_f64()
    );
    println!(
        "{}: {}",
        "Total Hashes | إجمالي الهاشات".yellow(),
        total_hashes
    );
    println!(
        "{}: {:.2} H/s",
        "Average Rate | المعدل المتوسط".yellow(),
        total_hashes as f64 / total_time.as_secs_f64()
    );
    println!(
        "{}: {:.2}s",
        "Avg Time/Block | متوسط الوقت للكتلة".yellow(),
        total_time.as_secs_f64() / cli.blocks as f64
    );
    println!();
}

fn print_mining_result(block_num: u32, block: &Block, stats: &MiningStats) {
    println!(
        "  {} {}",
        "✓ Block Mined | تم تعدين الكتلة".green(),
        format!("#{}", block_num).bold()
    );
    println!(
        "    {}: {}",
        "Hash | الهاش".dimmed(),
        hex::encode(block.hash()).white()
    );
    println!(
        "    {}: {}",
        "Nonce | النونس".dimmed(),
        format!("{}", stats.nonce_found).white()
    );
    println!(
        "    {}: {}",
        "Hashes | الهاشات".dimmed(),
        format!("{}", stats.hashes_computed).white()
    );
    println!(
        "    {}: {:.2}s",
        "Time | الوقت".dimmed(),
        stats.duration.as_secs_f64()
    );
    println!(
        "    {}: {:.2} H/s",
        "Hash Rate | معدل الهاش".dimmed(),
        stats.hash_rate
    );
}
