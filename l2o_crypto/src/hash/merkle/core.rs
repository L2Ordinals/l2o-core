use l2o_common::common::data::hash::{Hash256, MerkleProofCommonHash256};
use plonky2::{hash::hash_types::HashOut, field::goldilocks_field::GoldilocksField};
use serde::{Serialize, Deserialize};

use crate::hash::traits::L2OHash;

use super::traits::{MerkleHasher, MerkleHasherWithMarkedLeaf};


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleProofCore<Hash: PartialEq + Copy> {
    pub root: Hash,
    pub value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}

impl Into<MerkleProofCommonHash256> for MerkleProofCore<Hash256> {
    fn into(self) -> MerkleProofCommonHash256 {
        MerkleProofCommonHash256 {
            root: self.root,
            value: self.value,
            index: self.index,
            siblings: self.siblings,
        }
    }
}
impl From<MerkleProofCommonHash256> for MerkleProofCore<Hash256> {
    fn from(proof: MerkleProofCommonHash256) -> Self {
        MerkleProofCore {
            root: proof.root,
            value: proof.value,
            index: proof.index,
            siblings: proof.siblings,
        }
    }
}


impl Into<MerkleProofCommonHash256> for MerkleProofCore<HashOut<GoldilocksField>> {
    fn into(self) -> MerkleProofCommonHash256 {
        MerkleProofCommonHash256 {
            root: self.root.to_hash_256(),
            value: self.value.to_hash_256(),
            index: self.index,
            siblings: self.siblings.into_iter().map(|f|f.to_hash_256()).collect(),
        }
    }
}
impl From<MerkleProofCommonHash256> for MerkleProofCore<HashOut<GoldilocksField>> {
    fn from(proof: MerkleProofCommonHash256) -> Self {
        MerkleProofCore {
            root: HashOut::<GoldilocksField>::from_hash_256(&proof.root),
            value: HashOut::<GoldilocksField>::from_hash_256(&proof.value),
            index: proof.index,
            siblings: proof.siblings.into_iter().map(|f|HashOut::<GoldilocksField>::from_hash_256(&f)).collect(),
        }
    }
}


impl Into<MerkleProofCore<Hash256>> for MerkleProofCore<HashOut<GoldilocksField>> {
    fn into(self) -> MerkleProofCore<Hash256> {
        MerkleProofCore {
            root: self.root.to_hash_256(),
            value: self.value.to_hash_256(),
            index: self.index,
            siblings: self.siblings.into_iter().map(|x| x.to_hash_256()).collect(),
        }
    }
}
impl From<MerkleProofCore<Hash256>> for MerkleProofCore<HashOut<GoldilocksField>> {
    fn from(proof: MerkleProofCore<Hash256>) -> Self {
        MerkleProofCore {
            root: HashOut::<GoldilocksField>::from_hash_256(&proof.root),
            value: HashOut::<GoldilocksField>::from_hash_256(&proof.value),
            index: proof.index,
            siblings: proof.siblings.into_iter().map(|x| HashOut::<GoldilocksField>::from_hash_256(&x)).collect(),
        }
    }
}

impl<Hash: PartialEq + Copy> MerkleProofCore<Hash> {
    pub fn verify<Hasher: MerkleHasher<Hash>>(&self) -> bool {
      verify_merkle_proof_core::<Hash, Hasher>(&self)
    }
    pub fn verify_marked<Hasher: MerkleHasherWithMarkedLeaf<Hash>>(&self) -> bool {
      verify_merkle_proof_marked_leaves_core::<Hash, Hasher>(&self)

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
  pub fn verify<Hasher: MerkleHasher<Hash>>(&self) -> bool {
    verify_delta_merkle_proof_core::<Hash, Hasher>(&self)
  }
  pub fn verify_marked<Hasher: MerkleHasherWithMarkedLeaf<Hash>>(&self) -> bool {
    verify_delta_merkle_proof_marked_leaves_core::<Hash, Hasher>(&self)
  }
}

impl Into<DeltaMerkleProofCore<Hash256>> for DeltaMerkleProofCore<HashOut<GoldilocksField>> {
    fn into(self) -> DeltaMerkleProofCore<Hash256> {
        DeltaMerkleProofCore {
            old_root: self.old_root.to_hash_256(),
            new_root: self.new_root.to_hash_256(),
            old_value: self.old_value.to_hash_256(),
            new_value: self.new_value.to_hash_256(),
            index: self.index,
            siblings: self.siblings.into_iter().map(|x| x.to_hash_256()).collect(),
        }
    }
}
impl From<DeltaMerkleProofCore<Hash256>> for DeltaMerkleProofCore<HashOut<GoldilocksField>> {
    fn from(proof: DeltaMerkleProofCore<Hash256>) -> Self {
        DeltaMerkleProofCore {
            old_root: HashOut::<GoldilocksField>::from_hash_256(&proof.old_root),
            new_root: HashOut::<GoldilocksField>::from_hash_256(&proof.new_root),
            old_value: HashOut::<GoldilocksField>::from_hash_256(&proof.old_value),
            new_value: HashOut::<GoldilocksField>::from_hash_256(&proof.new_value),
            index: proof.index,
            siblings: proof.siblings.into_iter().map(|x| HashOut::<GoldilocksField>::from_hash_256(&x)).collect(),
        }
    }
}
pub fn verify_merkle_proof_core<Hash: PartialEq + Copy, Hasher: MerkleHasher<Hash>>(
    proof: &MerkleProofCore<Hash>,
) -> bool {
    let mut current = proof.value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if proof.index & (1 << i) == 0 {
            current = Hasher::two_to_one(&current, sibling);
        } else {
            current = Hasher::two_to_one(sibling, &current);
        }
    }
    current == proof.root
}
pub fn verify_delta_merkle_proof_core<Hash: PartialEq + Copy, Hasher: MerkleHasher<Hash>>(
    proof: &DeltaMerkleProofCore<Hash>,
) -> bool {
    let mut current = proof.old_value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if proof.index & (1 << i) == 0 {
            current = Hasher::two_to_one(&current, sibling);
        } else {
            current = Hasher::two_to_one(sibling, &current);
        }
    }
    if current != proof.old_root {
        return false;
    }
    current = proof.new_value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if proof.index & (1 << i) == 0 {
            current = Hasher::two_to_one(&current, sibling);
        } else {
            current = Hasher::two_to_one(sibling, &current);
        }
    }
    current == proof.new_root
}

pub fn verify_merkle_proof_marked_leaves_core<
    Hash: PartialEq + Copy,
    Hasher: MerkleHasherWithMarkedLeaf<Hash>,
>(
    proof: &MerkleProofCore<Hash>,
) -> bool {
    let mut current = proof.value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if i == 0 {
            if proof.index & (1 << i) == 0 {
                current = Hasher::two_to_one_marked_leaf(&current, sibling);
            } else {
                current = Hasher::two_to_one_marked_leaf(sibling, &current);
            }
        } else {
            if proof.index & (1 << i) == 0 {
                current = Hasher::two_to_one(&current, sibling);
            } else {
                current = Hasher::two_to_one(sibling, &current);
            }
        }
    }
    current == proof.root
}
pub fn verify_delta_merkle_proof_marked_leaves_core<
    Hash: PartialEq + Copy,
    Hasher: MerkleHasherWithMarkedLeaf<Hash>,
>(
    proof: &DeltaMerkleProofCore<Hash>,
) -> bool {
    let mut current = proof.old_value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if i == 0 {
            if proof.index & (1 << i) == 0 {
                current = Hasher::two_to_one_marked_leaf(&current, sibling);
            } else {
                current = Hasher::two_to_one_marked_leaf(sibling, &current);
            }
        } else {
            if proof.index & (1 << i) == 0 {
                current = Hasher::two_to_one(&current, sibling);
            } else {
                current = Hasher::two_to_one(sibling, &current);
            }
        }
    }
    if current != proof.old_root {
        return false;
    }
    current = proof.new_value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if i == 0 {
            if proof.index & (1 << i) == 0 {
                current = Hasher::two_to_one_marked_leaf(&current, sibling);
            } else {
                current = Hasher::two_to_one_marked_leaf(sibling, &current);
            }
        } else {
            if proof.index & (1 << i) == 0 {
                current = Hasher::two_to_one(&current, sibling);
            } else {
                current = Hasher::two_to_one(sibling, &current);
            }
        }
    }
    current == proof.new_root
}


pub fn calc_merkle_root_from_leaves<Hash: PartialEq + Copy, Hasher: MerkleHasher<Hash>>(leaves: Vec<Hash>) -> Hash {
  let mut current_leaves: Vec<Hash> = leaves.chunks_exact(2).map(|chunk| {
      Hasher::two_to_one(&chunk[0], &chunk[1])
  }).collect();
  let height = (current_leaves.len() as f64).log2().ceil() as usize;
  for _ in 1..height{
      let next_leaves = current_leaves.chunks_exact(2).map(|chunk| {
          Hasher::two_to_one(&chunk[0], &chunk[1])
      }).collect();
      current_leaves = next_leaves;
  }
  current_leaves[0]
}

pub fn calc_merkle_root_from_marked_leaves<Hash: PartialEq + Copy, Hasher: MerkleHasherWithMarkedLeaf<Hash>>(leaves: Vec<Hash>) -> Hash {
    let mut current_leaves: Vec<Hash> = leaves.chunks_exact(2).map(|chunk| {
        Hasher::two_to_one_marked_leaf(&chunk[0], &chunk[1])
    }).collect();
    let height = (current_leaves.len() as f64).log2().ceil() as usize;
    for _ in 1..height{
        let next_leaves = current_leaves.chunks_exact(2).map(|chunk| {
            Hasher::two_to_one(&chunk[0], &chunk[1])
        }).collect();
        current_leaves = next_leaves;
    }
    current_leaves[0]
}