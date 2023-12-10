use crate::{hash::{merkle::traits::{MerkleHasher, MerkleHasherWithMarkedLeaf}, traits::L2OBlockHasher}, standards::l2o_a::L2OBlockInscriptionV1};
use l2o_common::common::data::hash::Hash256;
use sha3::{Digest, Keccak256};

use super::block_hasher::get_block_payload_bytes;

pub struct Keccack256Hasher;
impl MerkleHasher<Hash256> for Keccack256Hasher {
    fn two_to_one(left: &Hash256, right: &Hash256) -> Hash256 {
        let mut hasher = Keccak256::new();
        hasher.update(&left.0);
        hasher.update(&right.0);

        let result: [u8; 32] = hasher.finalize().into();
        Hash256(result)
    }
}

impl MerkleHasherWithMarkedLeaf<Hash256> for Keccack256Hasher {
    fn two_to_one_marked_leaf(left: &Hash256, right: &Hash256) -> Hash256 {
        let mut hasher = Keccak256::new();
        hasher.update(&left.0);
        hasher.update(&right.0);
        hasher.update(&[1u8]);
        let result: [u8; 32] = hasher.finalize().into();
        Hash256(result)
    }
}

impl L2OBlockHasher for Keccack256Hasher {
    fn get_l2_block_hash(block: &L2OBlockInscriptionV1)->Hash256 {
        let payload = get_block_payload_bytes(block);
        let mut hasher = Keccak256::new();
        hasher.update(&payload);
        let result: [u8; 32] = hasher.finalize().into();
        Hash256(result)
    }
}