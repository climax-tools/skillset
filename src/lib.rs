pub mod cli;
pub mod config;
pub mod conventions;
pub mod error;
pub mod registry;
pub mod skill;
pub mod sources;

use cli::Cli;
use error::Result;

pub async fn run(cli: Cli) -> Result<()> {
    cli::handle_command(cli).await
}