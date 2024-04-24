use once_cell::sync::Lazy;

use crate::decimal::Decimal;

pub mod chain;
pub mod decimal;
pub mod error;
pub mod inscription;
pub mod media;
pub mod operation;
pub mod sat_point;
pub mod serde_helpers;
pub mod tag;
pub mod test_helpers;

pub const ORIGINAL_TICK_LENGTH: usize = 4;
pub const SELF_ISSUANCE_TICK_LENGTH: usize = 5;
pub const MAX_TICK_BYTE_COUNT: usize = SELF_ISSUANCE_TICK_LENGTH;
pub const PROTOCOL_LITERAL: &str = "brc-20";
pub const MAX_DECIMAL_WIDTH: u8 = 18;

pub static MAXIMUM_SUPPLY: Lazy<Decimal> = Lazy::new(|| Decimal::from(u64::MAX));

pub static BIGDECIMAL_TEN: Lazy<Decimal> = Lazy::new(|| Decimal::from(10u64));

#[allow(dead_code)]
pub const fn default_decimals() -> u8 {
    MAX_DECIMAL_WIDTH
}
