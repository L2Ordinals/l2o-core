use std::{fmt::Display, str::FromStr};

use l2o_common::common::data::hash::Hash256;
use plonky2::{
    field::{
        goldilocks_field::GoldilocksField,
        types::{Field, Sample},
    },
    hash::hash_types::{HashOut, RichField},
    plonk::config::GenericHashOut,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Wrapper<T>(pub T);

impl<T> std::ops::Deref for Wrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for Wrapper<T> {
    fn from(value: T) -> Self {
        Wrapper(value)
    }
}

pub type WHashOut<F> = Wrapper<HashOut<F>>;
pub type GoldilocksHashOut = WHashOut<GoldilocksField>;
pub type GHashOut = HashOut<GoldilocksField>;

impl<F: Field> Default for WHashOut<F> {
    fn default() -> Self {
        Wrapper(HashOut::ZERO)
    }
}

impl<F: RichField> Display for WHashOut<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self)
            .map(|v| v.replace('\"', ""))
            .unwrap();

        write!(f, "{}", s)
    }
}

impl<F: RichField> FromStr for WHashOut<F> {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut r = hex::decode(s)?;
        if r.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }

        r.reverse();
        Ok(Self::from_bytes(&r))
    }
}

impl<F: RichField> Serialize for WHashOut<F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = self.0.to_bytes();
        bytes.reverse();
        hex::encode(&bytes).serialize(serializer)
    }
}

impl<'de, F: RichField> Deserialize<'de> for WHashOut<F> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let raw = String::deserialize(deserializer)?;
        let mut bytes = hex::decode(&raw).map_err(|_| Error::custom("invalid hex string"))?;
        if bytes.len() != 32 {
            return Err(Error::custom("hash hex strings must be 64 characters long"));
        }
        bytes.reverse();
        Ok(Wrapper(HashOut::from_bytes(&bytes)))
    }
}

impl<F: RichField> GenericHashOut<F> for WHashOut<F> {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        HashOut::from_bytes(bytes).into()
    }

    fn to_vec(&self) -> Vec<F> {
        self.0.to_vec()
    }
}

impl<F: Field> WHashOut<F> {
    pub const ZERO: Self = Wrapper(HashOut::ZERO);

    pub fn read(inputs: &mut core::slice::Iter<F>) -> Self {
        HashOut {
            elements: [
                *inputs.next().unwrap(),
                *inputs.next().unwrap(),
                *inputs.next().unwrap(),
                *inputs.next().unwrap(),
            ],
        }
        .into()
    }

    pub fn write(&self, inputs: &mut Vec<F>) {
        inputs.append(&mut self.0.elements.to_vec())
    }

    pub fn rand() -> Self {
        HashOut::rand().into()
    }
}

pub fn hash256_to_goldilocks_u32(hash: &Hash256) -> [GoldilocksField; 8] {
    let u32s: [u32; 8] = hash.into();
    core::array::from_fn(|i| GoldilocksField::from_canonical_u32(u32s[i]))
}
pub fn hash256_to_goldilocks_hash(hash: &Hash256) -> HashOut<GoldilocksField> {
    let u64s: [u64; 4] = hash.into();
    HashOut {
        elements: core::array::from_fn(|i| GoldilocksField::from_noncanonical_u64(u64s[i])),
    }
}
