use l2o_common::IndexerArgs;

use crate::error::Result;

pub async fn run(args: IndexerArgs) -> Result<()> {
    let indexer = l2o_indexer::Indexer::new(args)?;
    indexer.listen().await?;
    Ok(())
}
