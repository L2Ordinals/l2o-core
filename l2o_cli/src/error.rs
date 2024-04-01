use anyhow::Error as AnyhowError;
use l2o_indexer::error::Error as L2OIndexError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Anyhow Error: `{0:?}`")]
    AnyhowError(#[from] AnyhowError),
    #[error("L2O Index Error: `{0:?}`")]
    L2OIndexError(#[from] L2OIndexError),
}

pub type Result<T> = std::result::Result<T, Error>;
