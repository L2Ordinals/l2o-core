use std::io::Error as IoError;

use anyhow::Error as AnyhowError;
use serde_json::error::Error as SerializeJsonError;
use thiserror::Error as ThisError;

#[non_exhaustive]
#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Anyhow Error: `{0:?}`")]
    AnyhowError(#[from] AnyhowError),
    #[error("Io Error: `{0:?}`")]
    IoError(#[from] IoError),
    #[error("Actix-Web Error: `{0:?}`")]
    ActixWebError(String),
    #[error("Serialize Json Error: `{0:?}`")]
    SerializeJsonError(#[from] SerializeJsonError),
}

pub type Result<T> = std::result::Result<T, Error>;
