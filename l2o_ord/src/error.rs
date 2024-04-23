use redb::TableError;
use serde::Deserialize;
use serde::Serialize;

use crate::inscription::inscription_id::InscriptionId;

#[derive(Debug, Clone, PartialEq, thiserror::Error, Deserialize, Serialize)]
pub enum BRC20Error {
    #[error("invalid number: {0}")]
    InvalidNum(String),

    #[error("tick invalid supply {0}")]
    InvalidSupply(String),

    #[error("tick: {0} has been existed")]
    DuplicateTick(String),

    #[error("tick: {0} not found")]
    TickNotFound(String),

    #[error("illegal tick length '{0}'")]
    InvalidTickLen(String),

    #[error("decimals {0} too large")]
    DecimalsTooLarge(u8),

    #[error("tick: {0} has been minted")]
    TickMinted(String),

    #[error("tick: {0} mint limit out of range {0}")]
    MintLimitOutOfRange(String, String),

    #[error("zero amount not allowed")]
    InvalidZeroAmount,

    #[error("amount overflow: {0}")]
    AmountOverflow(String),

    #[error("insufficient balance: {0} {1}")]
    InsufficientBalance(String, String),

    #[error("amount exceed limit: {0}")]
    AmountExceedLimit(String),

    #[error("transferable inscriptionId not found: {0}")]
    TransferableNotFound(InscriptionId),

    #[error("invalid inscribe to coinbase")]
    InscribeToCoinbase,

    #[error("transferable owner not match {0}")]
    TransferableOwnerNotMatch(InscriptionId),

    #[error("self issuance not activated")]
    SelfIssuanceNotActivated,

    #[error("'self_mint' must be set to 'true', when deploying 5 bytes tick")]
    SelfIssuanceCheckedFailed,

    #[error("self mint permission denied")]
    SelfMintPermissionDenied,

    /// an InternalError is an error that happens exceed our expect
    /// and should not happen under normal circumstances
    #[error("internal error: {0}")]
    InternalError(String),

    // num error
    #[error("{op} overflow: original: {org}, other: {other}")]
    Overflow {
        op: String,
        org: String,
        other: String,
    },

    #[error("invalid integer {0}")]
    InvalidInteger(String),
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("brc20 error: {0}")]
    BRC20Error(BRC20Error),

    #[error("ledger error: {0}")]
    LedgerError(anyhow::Error),

    #[error("table error: {0}")]
    TableError(TableError),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
    #[error("invalid content type")]
    InvalidContentType,

    #[error("unsupport content type")]
    UnSupportContentType,

    #[error("invalid json string")]
    InvalidJson,

    #[error("not brc20 json")]
    NotBRC20Json,

    #[error("parse operation json error: {0}")]
    ParseOperationJsonError(String),
}

impl From<BRC20Error> for Error {
    fn from(e: BRC20Error) -> Self {
        Self::BRC20Error(e)
    }
}

impl From<TableError> for Error {
    fn from(error: TableError) -> Self {
        Self::TableError(error)
    }
}
