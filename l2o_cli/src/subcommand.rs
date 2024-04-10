use clap::command;
use clap::Parser;
use clap::Subcommand;

pub mod indexer;
pub mod indexer_ordhook;
pub mod initializer;
pub mod sequencer;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Indexer(l2o_common::IndexerArgs),
    IndexerOrdHook(l2o_common::IndexerOrdHookArgs),
    Sequencer(l2o_common::SequencerArgs),
    Initializer(l2o_common::InitializerArgs),
}
