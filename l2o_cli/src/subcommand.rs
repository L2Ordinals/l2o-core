use clap::command;
use clap::Parser;
use clap::Subcommand;

pub mod indexer;
#[cfg(debug_assertions)]
pub mod initializer;
#[cfg(debug_assertions)]
pub mod sequencer;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Indexer(l2o_common::IndexerArgs),
    #[cfg(debug_assertions)]
    Sequencer(l2o_common::SequencerArgs),
    #[cfg(debug_assertions)]
    Initializer(l2o_common::InitializerArgs),
}
