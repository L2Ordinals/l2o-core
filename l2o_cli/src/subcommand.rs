use clap::command;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

pub mod indexer;
pub mod indexer_poc;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Indexer(IndexerArgs),
    IndexerPoc(IndexerPocArgs),
}

#[derive(Clone, Args)]
pub struct IndexerArgs {}

#[derive(Clone, Args)]
pub struct IndexerPocArgs {}
