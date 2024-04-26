use kvq::traits::KVQSerializable;
use l2o_common::common::data::hash::Hash256;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::HashOut;
use serde::Deserialize;
use serde::Serialize;

use crate::hash::merkle::traits::GeneralMerkleZeroHasher;
use crate::hash::traits::L2OHash;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound = "Hash: Serialize, for<'de2> Hash: Deserialize<'de2>")]
pub struct MerkleProofCore<Hash>
where
    Hash: PartialEq + Copy + Serialize,
    for<'de2> Hash: Deserialize<'de2>,
{
    pub root: Hash,
    pub value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}

pub type MerkleProofHash256 = MerkleProofCore<Hash256>;

impl From<&MerkleProofCore<HashOut<GoldilocksField>>> for MerkleProofCore<Hash256> {
    fn from(proof: &MerkleProofCore<HashOut<GoldilocksField>>) -> MerkleProofCore<Hash256> {
        MerkleProofCore {
            root: proof.root.to_hash_256(),
            value: proof.value.to_hash_256(),
            index: proof.index,
            siblings: proof.siblings.iter().map(|x| x.to_hash_256()).collect(),
        }
    }
}
impl From<&MerkleProofCore<Hash256>> for MerkleProofCore<HashOut<GoldilocksField>> {
    fn from(proof: &MerkleProofCore<Hash256>) -> Self {
        MerkleProofCore {
            root: HashOut::<GoldilocksField>::from_hash_256(&proof.root),
            value: HashOut::<GoldilocksField>::from_hash_256(&proof.value),
            index: proof.index,
            siblings: proof
                .siblings
                .iter()
                .map(|x| HashOut::<GoldilocksField>::from_hash_256(x))
                .collect(),
        }
    }
}

impl<Hash> MerkleProofCore<Hash>
where
    Hash: PartialEq + Copy + Serialize,
    for<'de2> Hash: Deserialize<'de2>,
{
    pub fn verify_marked_if<Hasher: GeneralMerkleZeroHasher<Hash>>(&self, marked: bool) -> bool {
        verify_merkle_proof_core_marked_if::<Hash, Hasher>(&self, marked)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeltaMerkleProofCorePartial<Hash: PartialEq + Copy> {
    pub old_value: Hash,
    pub new_value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeltaMerkleProofCore<Hash: PartialEq + Copy> {
    pub old_root: Hash,
    pub old_value: Hash,

    pub new_root: Hash,
    pub new_value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}
impl<Hash: PartialEq + Copy> DeltaMerkleProofCore<Hash> {
    pub fn verify_marked_if<Hasher: GeneralMerkleZeroHasher<Hash>>(&self, marked: bool) -> bool {
        verify_delta_merkle_proof_core_marked_if::<Hash, Hasher>(&self, marked)
    }
}

impl From<&DeltaMerkleProofCore<HashOut<GoldilocksField>>> for DeltaMerkleProofCore<Hash256> {
    fn from(proof: &DeltaMerkleProofCore<HashOut<GoldilocksField>>) -> Self {
        DeltaMerkleProofCore {
            old_root: proof.old_root.to_hash_256(),
            new_root: proof.new_root.to_hash_256(),
            old_value: proof.old_value.to_hash_256(),
            new_value: proof.new_value.to_hash_256(),
            index: proof.index,
            siblings: proof.siblings.iter().map(|x| x.to_hash_256()).collect(),
        }
    }
}
impl From<&DeltaMerkleProofCore<Hash256>> for DeltaMerkleProofCore<HashOut<GoldilocksField>> {
    fn from(proof: &DeltaMerkleProofCore<Hash256>) -> Self {
        DeltaMerkleProofCore {
            old_root: HashOut::<GoldilocksField>::from_hash_256(&proof.old_root),
            new_root: HashOut::<GoldilocksField>::from_hash_256(&proof.new_root),
            old_value: HashOut::<GoldilocksField>::from_hash_256(&proof.old_value),
            new_value: HashOut::<GoldilocksField>::from_hash_256(&proof.new_value),
            index: proof.index,
            siblings: proof
                .siblings
                .iter()
                .map(|x| HashOut::<GoldilocksField>::from_hash_256(x))
                .collect(),
        }
    }
}
pub fn verify_merkle_proof_core_marked_if<Hash, Hasher: GeneralMerkleZeroHasher<Hash>>(
    proof: &MerkleProofCore<Hash>,
    marked: bool,
) -> bool
where
    Hash: PartialEq + Copy + Serialize,
    for<'de2> Hash: Deserialize<'de2>,
{
    proof.root
        == calc_merkle_root_marked_if::<Hash, Hasher>(
            proof.value,
            &proof.siblings,
            proof.index,
            marked,
        )
}

pub fn verify_delta_merkle_proof_core_marked_if<
    Hash: PartialEq + Copy,
    Hasher: GeneralMerkleZeroHasher<Hash>,
>(
    proof: &DeltaMerkleProofCore<Hash>,
    marked: bool,
) -> bool {
    proof.old_root
        == calc_merkle_root_marked_if::<Hash, Hasher>(
            proof.old_value,
            &proof.siblings,
            proof.index,
            marked,
        )
        && proof.new_root
            == calc_merkle_root_marked_if::<Hash, Hasher>(
                proof.new_value,
                &proof.siblings,
                proof.index,
                marked,
            )
}

pub fn calc_merkle_path<Hash: PartialEq + Copy, Hasher: GeneralMerkleZeroHasher<Hash>>(
    value: Hash,
    siblings: &[Hash],
    index: u64,
) -> Vec<Hash> {
    let mut current = value;
    let mut merkle_path = vec![current];
    for (i, sibling) in siblings.iter().enumerate() {
        if index & (1 << i) == 0 {
            current = Hasher::two_to_one(&current, &sibling);
        } else {
            current = Hasher::two_to_one(&sibling, &current);
        }
        merkle_path.push(current);
    }
    merkle_path
}

pub fn calc_merkle_root_marked_if<Hash: PartialEq + Copy, Hasher: GeneralMerkleZeroHasher<Hash>>(
    value: Hash,
    siblings: &[Hash],
    index: u64,
    marked: bool,
) -> Hash {
    let mut current = value;
    for (i, sibling) in siblings.iter().enumerate() {
        let marked = i == 0 && marked;
        if index & (1 << i) == 0 {
            current = Hasher::two_to_one_marked_if(&current, &sibling, marked);
        } else {
            current = Hasher::two_to_one_marked_if(&sibling, &current, marked);
        }
    }
    current
}

impl<Hash> KVQSerializable for MerkleProofCore<Hash>
where
    Hash: PartialEq + Copy + Serialize,
    for<'de2> Hash: Deserialize<'de2>,
{
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
