#[cfg(debug_assertions)]
mod circuits;
mod error;
mod subcommand;

use clap::Parser;
use error::Result;

use crate::subcommand::indexer_ordhook;
#[cfg(debug_assertions)]
use crate::subcommand::initializer;
#[cfg(debug_assertions)]
use crate::subcommand::sequencer;
use crate::subcommand::Cli;
use crate::subcommand::Commands;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    l2o_common::setup_logger();

    let cli = Cli::parse();
    match cli.command {
        Commands::IndexerOrdHook(args) => indexer_ordhook::run(&args).await?,
        #[cfg(debug_assertions)]
        Commands::Sequencer(args) => sequencer::run(&args).await?,
        #[cfg(debug_assertions)]
        Commands::Initializer(args) => {
            initializer::run(&args).await?;
        }
    }

    Ok(())
}
