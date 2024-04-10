mod circuits;
mod error;
mod subcommand;

use clap::Parser;
use error::Result;

use crate::subcommand::indexer;
use crate::subcommand::indexer_ordhook;
use crate::subcommand::initializer;
use crate::subcommand::sequencer;
use crate::subcommand::Cli;
use crate::subcommand::Commands;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    l2o_common::setup_logger();

    let cli = Cli::parse();
    match cli.command {
        Commands::Indexer(args) => indexer::run(&args).await?,
        Commands::IndexerOrdHook(args) => indexer_ordhook::run(&args).await?,
        Commands::Sequencer(args) => sequencer::run(&args).await?,
        Commands::Initializer(args) => initializer::run(&args).await?,
    }

    Ok(())
}
