use std::io::Error as IoError;

use anyhow::Error as AnyhowError;
use hex::FromHexError;
use k256::schnorr::signature::Error as SchnorrSignatureError;
use musig2::errors::VerifyError;
use musig2::secp::errors::InvalidPointBytes;
use serde_json::error::Error as SerializeJsonError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("Anyhow Error: `{0:?}`")]
    AnyhowError(#[from] AnyhowError),
    #[error("Io Error: `{0:?}`")]
    IoError(#[from] IoError),
    #[error("Serialize Json Error: `{0:?}`")]
    SerializeJsonError(#[from] SerializeJsonError),
    #[error("Musig2 Verify Error: `{0:?}`")]
    Musig2VerifyError(#[from] VerifyError),
    #[error("Musig2 InvalidPointBytes Error: `{0:?}`")]
    Musig2InvalidPointBytes(#[from] InvalidPointBytes),
    #[error("FromHexError: `{0:?}`")]
    FromHexError(#[from] FromHexError),
    #[error("SchnorrSignatureError: `{0:?}`")]
    SchnorrSignatureError(#[from] SchnorrSignatureError),
}

pub type Result<T> = std::result::Result<T, Error>;
