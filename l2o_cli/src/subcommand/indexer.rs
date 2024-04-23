use l2o_common::IndexerArgs;

use crate::error::Result;

pub async fn run(args: &IndexerArgs) -> Result<()> {
    l2o_indexer::listen(args).await?;
    Ok(())
}
