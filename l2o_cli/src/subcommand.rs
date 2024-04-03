use clap::command;
use clap::Args;
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
    Indexer(IndexerArgs),
    IndexerOrdHook(IndexerOrdHookArgs),
}

#[derive(Clone, Args)]
pub struct IndexerArgs {}

#[derive(Clone, Args)]
pub struct IndexerOrdHookArgs {}
