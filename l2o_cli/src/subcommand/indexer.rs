use l2o_common::IndexerArgs;

use crate::error::Result;

pub async fn run(_args: &IndexerArgs) -> Result<()> {
    l2o_indexer::listen().await?;
    Ok(())
}
