use crate::error::Result;
use crate::subcommand::IndexerOrdHookArgs;

pub async fn run(_args: &IndexerOrdHookArgs) -> Result<()> {
    l2o_indexer_ordhook::listen().await.unwrap();
    Ok(())
}
