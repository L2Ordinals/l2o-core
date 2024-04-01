use crate::error::Result;
use crate::subcommand::IndexerPocArgs;

pub async fn run(args: &IndexerPocArgs) -> Result<()> {
    l2o_indexer_poc::listen().await.unwrap();
    Ok(())
}
