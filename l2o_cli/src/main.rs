mod error;
mod subcommand;

use clap::Parser;
use error::Result;

use crate::subcommand::indexer;
use crate::subcommand::indexer_poc;
use crate::subcommand::Cli;
use crate::subcommand::Commands;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    l2o_utils::setup_logger();

    let cli = Cli::parse();
    match cli.command {
        Commands::Indexer(args) => indexer::run(&args).await?,
        Commands::IndexerPoc(args) => indexer_poc::run(&args).await?,
    }

    Ok(())
}
