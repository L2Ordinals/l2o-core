use l2o_common::common::data::hash::Hash256;
use sha2::Digest;
use sha2::Sha256;

use crate::hash::merkle::traits::MerkleHasher;
use crate::hash::merkle::traits::MerkleHasherWithMarkedLeaf;

pub struct Sha256Hasher;
impl MerkleHasher<Hash256> for Sha256Hasher {
    fn two_to_one(left: &Hash256, right: &Hash256) -> Hash256 {
        let mut hasher = Sha256::new();
        hasher.update(&left.0);
        hasher.update(&right.0);

        let result: [u8; 32] = hasher.finalize().into();
        Hash256(result)
    }
}

impl MerkleHasherWithMarkedLeaf<Hash256> for Sha256Hasher {
    fn two_to_one_marked_leaf(left: &Hash256, right: &Hash256) -> Hash256 {
        let mut hasher = Sha256::new();
        hasher.update(&left.0);
        hasher.update(&right.0);
        hasher.update(&[1u8]);
        let result: [u8; 32] = hasher.finalize().into();
        Hash256(result)
    }
}

pub fn hash(data: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Hash256(hasher.finalize().into())
}
