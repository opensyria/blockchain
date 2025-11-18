use opensyria_explorer_backend::ExplorerServer;
use std::net::SocketAddr;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments (simple version)
    let args: Vec<String> = std::env::args().collect();
    
    let data_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("data")
    };

    let port: u16 = if args.len() > 2 {
        args[2].parse().unwrap_or(3000)
    } else {
        3000
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Get static directory path
    let static_dir = std::env::current_exe()?
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("crates/explorer-backend/static"))
        .unwrap_or_else(|| PathBuf::from("crates/explorer-backend/static"));

    println!("🚀 Open Syria Block Explorer");
    println!("📂 Data directory: {}", data_dir.display());
    println!("🌐 Server address: http://{}", addr);
    println!("📁 Static files: {}", static_dir.display());
    println!();
    println!("Open your browser to: http://localhost:{}", port);
    println!();

    let server = ExplorerServer::new(data_dir, addr)?
        .with_static_dir(static_dir);

    server.run().await?;

    Ok(())
}
