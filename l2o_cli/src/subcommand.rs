use clap::command;
use clap::Parser;
use clap::Subcommand;

pub mod indexer;
pub mod indexer_ordhook;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Indexer(l2o_common::IndexerArgs),
    IndexerOrdHook(l2o_common::IndexerOrdHookArgs),
}
