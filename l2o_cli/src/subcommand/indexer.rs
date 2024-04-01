use crate::error::Result;
use crate::subcommand::IndexerArgs;

pub async fn run(args: &IndexerArgs) -> Result<()> {
    l2o_indexer::listen().await?;
    Ok(())
}
