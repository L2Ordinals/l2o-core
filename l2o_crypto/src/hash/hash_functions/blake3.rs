use l2o_common::common::data::hash::Hash256;

use super::block_hasher::get_block_payload_bytes;
use crate::hash::merkle::traits::MerkleHasher;
use crate::hash::merkle::traits::MerkleHasherWithMarkedLeaf;
use crate::hash::traits::L2OBlockHasher;
use crate::standards::l2o_a::L2OBlockInscriptionV1;

pub struct Blake3Hasher;

impl MerkleHasher<Hash256> for Blake3Hasher {
    fn two_to_one(left: &Hash256, right: &Hash256) -> Hash256 {
        let mut data = [0; 64];
        data[..32].copy_from_slice(&left.0);
        data[32..].copy_from_slice(&right.0);
        let op = blake3::hash(&data);
        Hash256(*op.as_bytes())
    }
}
impl MerkleHasherWithMarkedLeaf<Hash256> for Blake3Hasher {
    fn two_to_one_marked_leaf(left: &Hash256, right: &Hash256) -> Hash256 {
        let mut data = [0; 65];
        data[..32].copy_from_slice(&left.0);
        data[32..64].copy_from_slice(&right.0);
        data[64] = 1;
        let op = blake3::hash(&data);
        Hash256(*op.as_bytes())
    }
}

impl L2OBlockHasher for Blake3Hasher {
    fn get_l2_block_hash(block: &L2OBlockInscriptionV1) -> Hash256 {
        let payload = get_block_payload_bytes(block);
        let op = blake3::hash(&payload);
        Hash256(*op.as_bytes())
    }
}
