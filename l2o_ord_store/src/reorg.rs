use core::fmt;

#[derive(Debug, PartialEq)]
pub enum ReorgError {
    Recoverable { height: u32, depth: u32 },
    Unrecoverable,
}

impl fmt::Display for ReorgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReorgError::Recoverable { height, depth } => {
                write!(f, "{depth} block deep reorg detected at height {height}")
            }
            ReorgError::Unrecoverable => write!(f, "unrecoverable reorg detected"),
        }
    }
}

impl std::error::Error for ReorgError {}

pub const MAX_SAVEPOINTS: u32 = 2;
pub const SAVEPOINT_INTERVAL: u32 = 10;
pub const CHAIN_TIP_DISTANCE: u32 = 21;
