use l2o_common::IndexerOrdHookArgs;

use crate::error::Result;

pub async fn run(args: &IndexerOrdHookArgs) -> Result<()> {
    l2o_indexer_ordhook::listen(args).await?;
    Ok(())
}
