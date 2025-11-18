use clap::Parser;
use std::path::PathBuf;
use opensyria_node_cli::Node;
use opensyria_wallet_api::{AppState, start_server};

#[derive(Parser)]
#[command(name = "opensyria-wallet-api")]
#[command(about = "Open Syria Wallet API Server | خادم واجهة برمجة التطبيقات للمحفظة")]
struct Cli {
    /// Node data directory
    #[arg(short, long, default_value = "~/.opensyria/node")]
    data_dir: String,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind to
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Expand tilde in data_dir
    let data_dir = if cli.data_dir.starts_with("~") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(cli.data_dir.replacen("~", &home, 1))
    } else {
        PathBuf::from(cli.data_dir)
    };

    // Open node
    println!("📂 Opening node at: {}", data_dir.display());
    let node = Node::open(data_dir)?;
    let chain_height = node.get_blockchain().get_chain_height()
        .map_err(|e| anyhow::anyhow!("Failed to get chain height: {}", e))?;
    println!("✅ Node opened successfully");
    println!("   Chain height: {}", chain_height);

    // Create app state
    let state = AppState::new(node);

    // Start server
    start_server(state, &cli.host, cli.port).await?;

    Ok(())
}
