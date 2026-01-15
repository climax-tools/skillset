use clap::Parser;
use skillset::cli::Cli;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    skillset::run(cli)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}
