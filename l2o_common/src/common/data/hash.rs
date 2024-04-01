use std::fmt::Display;

use kvq::traits::KVQSerializable;
use rand::RngCore;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;

#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Eq, Copy, Hash, Debug)]
pub struct Hash256(#[serde_as(as = "serde_with::hex::Hex")] pub [u8; 32]);

#[derive(Clone, Debug, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub struct MerkleProofCommonHash256 {
    pub root: Hash256,
    pub value: Hash256,
    pub index: u64,
    pub siblings: Vec<Hash256>,
}

impl Hash256 {
    pub fn from_str(s: &str) -> Result<Self, ()> {
        let bytes = hex::decode(s).unwrap();
        assert_eq!(bytes.len(), 32);
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
    pub fn rand() -> Self {
        let mut data = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut data);
        Self(data)
    }
}

impl Display for Hash256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

pub trait L2OHash: Sized + Copy + Clone + PartialEq + Serialize + for<'a> Deserialize<'a> {
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn zero() -> Self;
}

impl L2OHash for Hash256 {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut result = [0u8; 32];
        result.copy_from_slice(bytes);
        Hash256(result)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn zero() -> Self {
        Hash256([0u8; 32])
    }
}

impl Into<[u32; 8]> for &Hash256 {
    fn into(self) -> [u32; 8] {
        let mut result = [0u32; 8];
        for i in 0..8 {
            result[7 - i] = u32::from_be_bytes([
                self.0[i * 4],
                self.0[i * 4 + 1],
                self.0[i * 4 + 2],
                self.0[i * 4 + 3],
            ]);
        }
        result
    }
}

impl Into<[u64; 4]> for &Hash256 {
    fn into(self) -> [u64; 4] {
        let mut result = [0u64; 4];
        for i in 0..4 {
            result[7 - i] = u64::from_be_bytes([
                self.0[i * 4],
                self.0[i * 4 + 1],
                self.0[i * 4 + 2],
                self.0[i * 4 + 3],
                self.0[i * 4 + 4],
                self.0[i * 4 + 5],
                self.0[i * 4 + 6],
                self.0[i * 4 + 7],
            ]);
        }
        result
    }
}

impl KVQSerializable for Hash256 {
    fn to_bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut result = [0u8; 32];
        result.copy_from_slice(bytes);
        Hash256(result)
    }
}
