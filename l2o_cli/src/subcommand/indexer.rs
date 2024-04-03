use crate::error::Result;
use crate::subcommand::IndexerArgs;

pub async fn run(_args: &IndexerArgs) -> Result<()> {
    l2o_indexer::listen().await?;
    Ok(())
}
