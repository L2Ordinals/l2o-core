use std::fmt;
use std::str::FromStr;

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// A block number or tag.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum BlockNumber {
    /// Latest block
    #[default]
    Latest,
    /// Finalized block accepted as canonical
    Finalized,
    /// Block by number from canon chain
    Number(u32),
}

impl BlockNumber {
    /// Returns the numeric block number if explicitly set
    pub fn as_number(&self) -> Option<u32> {
        match *self {
            BlockNumber::Number(num) => Some(num),
            _ => None,
        }
    }

    /// Returns `true` if a numeric block number is set
    pub fn is_number(&self) -> bool {
        matches!(self, BlockNumber::Number(_))
    }

    /// Returns `true` if it's "latest"
    pub fn is_latest(&self) -> bool {
        matches!(self, BlockNumber::Latest)
    }

    /// Returns `true` if it's "finalized"
    pub fn is_finalized(&self) -> bool {
        matches!(self, BlockNumber::Finalized)
    }
}

impl<T: Into<u32>> From<T> for BlockNumber {
    fn from(num: T) -> Self {
        BlockNumber::Number(num.into())
    }
}

impl Serialize for BlockNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            BlockNumber::Number(ref x) => serializer.serialize_str(&format!("0x{x:x}")),
            BlockNumber::Finalized => serializer.serialize_str("finalized"),
            BlockNumber::Latest => serializer.serialize_str("latest"),
        }
    }
}

impl<'de> Deserialize<'de> for BlockNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        s.parse().map_err(serde::de::Error::custom)
    }
}

impl FromStr for BlockNumber {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "latest" => Ok(Self::Latest),
            "finalized" => Ok(Self::Finalized),
            // hex
            n if n.starts_with("0x") => u32::from_str_radix(n.trim_start_matches("0x"), 16)
                .map(Self::Number)
                .map_err(|e| e.to_string()),
            // decimal
            n => n
                .parse::<u32>()
                .map(|n| Self::Number(n.into()))
                .map_err(|e| e.to_string()),
        }
    }
}

impl fmt::Display for BlockNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockNumber::Number(ref x) => format!("0x{x:x}").fmt(f),
            BlockNumber::Latest => f.write_str("latest"),
            BlockNumber::Finalized => f.write_str("finalized"),
        }
    }
}
