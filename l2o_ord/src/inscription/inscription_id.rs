use core::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

use bitcoin::hashes::Hash;
use bitcoin::Txid;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use crate::serde_helpers::DeserializeFromStr;

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub struct InscriptionId {
    pub txid: Txid,
    pub index: u32,
}

impl Default for InscriptionId {
    fn default() -> Self {
        Self {
            txid: Txid::all_zeros(),
            index: 0,
        }
    }
}

impl InscriptionId {
    pub(crate) fn value(self) -> Vec<u8> {
        let index = self.index.to_le_bytes();
        let mut index_slice = index.as_slice();

        while index_slice.last().copied() == Some(0) {
            index_slice = &index_slice[0..index_slice.len() - 1];
        }

        self.txid
            .to_byte_array()
            .iter()
            .chain(index_slice)
            .copied()
            .collect()
    }
}

impl<'de> Deserialize<'de> for InscriptionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(DeserializeFromStr::deserialize(deserializer)?.0)
    }
}

impl Serialize for InscriptionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl Display for InscriptionId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}i{}", self.txid, self.index)
    }
}

#[derive(Debug)]
pub enum ParseError {
    Character(char),
    Length(usize),
    Separator(char),
    Txid(bitcoin::hashes::hex::HexToArrayError),
    Index(std::num::ParseIntError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Character(c) => write!(f, "invalid character: '{c}'"),
            Self::Length(len) => write!(f, "invalid length: {len}"),
            Self::Separator(c) => write!(f, "invalid separator: `{c}`"),
            Self::Txid(err) => write!(f, "invalid txid: {err}"),
            Self::Index(err) => write!(f, "invalid index: {err}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl FromStr for InscriptionId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(char) = s.chars().find(|char| !char.is_ascii()) {
            return Err(ParseError::Character(char));
        }

        const TXID_LEN: usize = 64;
        const MIN_LEN: usize = TXID_LEN + 2;

        if s.len() < MIN_LEN {
            return Err(ParseError::Length(s.len()));
        }

        let txid = &s[..TXID_LEN];

        let separator = s.chars().nth(TXID_LEN).unwrap();

        if separator != 'i' {
            return Err(ParseError::Separator(separator));
        }

        let vout = &s[TXID_LEN + 1..];

        Ok(Self {
            txid: txid.parse().map_err(ParseError::Txid)?,
            index: vout.parse().map_err(ParseError::Index)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_matches;
    use crate::inscription::inscription_id::InscriptionId;
    use crate::inscription::inscription_id::ParseError;
    use crate::test_helpers::inscription_id;
    use crate::test_helpers::txid;

    #[test]
    fn display() {
        assert_eq!(
            inscription_id(1).to_string(),
            "1111111111111111111111111111111111111111111111111111111111111111i1",
        );
        assert_eq!(
            InscriptionId {
                txid: crate::test_helpers::txid(1),
                index: 0,
            }
            .to_string(),
            "1111111111111111111111111111111111111111111111111111111111111111i0",
        );
        assert_eq!(
            InscriptionId {
                txid: txid(1),
                index: 0xFFFFFFFF,
            }
            .to_string(),
            "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
        );
    }

    #[test]
    fn from_str() {
        assert_eq!(
            "1111111111111111111111111111111111111111111111111111111111111111i1"
                .parse::<InscriptionId>()
                .unwrap(),
            inscription_id(1),
        );
        assert_eq!(
            "1111111111111111111111111111111111111111111111111111111111111111i4294967295"
                .parse::<InscriptionId>()
                .unwrap(),
            InscriptionId {
                txid: txid(1),
                index: 0xFFFFFFFF,
            },
        );
        assert_eq!(
            "1111111111111111111111111111111111111111111111111111111111111111i4294967295"
                .parse::<InscriptionId>()
                .unwrap(),
            InscriptionId {
                txid: txid(1),
                index: 0xFFFFFFFF,
            },
        );
    }

    #[test]
    fn from_str_bad_character() {
        assert_matches!(
            "→".parse::<InscriptionId>(),
            Err(ParseError::Character('→')),
        );
    }

    #[test]
    fn from_str_bad_length() {
        assert_matches!("foo".parse::<InscriptionId>(), Err(ParseError::Length(3)));
    }

    #[test]
    fn from_str_bad_separator() {
        assert_matches!(
            "0000000000000000000000000000000000000000000000000000000000000000x0"
                .parse::<InscriptionId>(),
            Err(ParseError::Separator('x')),
        );
    }

    #[test]
    fn from_str_bad_index() {
        assert_matches!(
            "0000000000000000000000000000000000000000000000000000000000000000ifoo"
                .parse::<InscriptionId>(),
            Err(ParseError::Index(_)),
        );
    }

    #[test]
    fn from_str_bad_txid() {
        assert_matches!(
            "x000000000000000000000000000000000000000000000000000000000000000i0"
                .parse::<InscriptionId>(),
            Err(ParseError::Txid(_)),
        );
    }
}
