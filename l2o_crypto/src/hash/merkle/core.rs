use serde::{Serialize, Deserialize};

use super::traits::{MerkleHasher, MerkleHasherWithMarkedLeaf};


#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleProofCore<Hash: PartialEq + Copy> {
    pub root: Hash,
    pub value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}

impl<Hash: PartialEq + Copy> MerkleProofCore<Hash> {
    pub fn verify<Hasher: MerkleHasher<Hash>>(&self) -> bool {
      verify_merkle_proof_core::<Hash, Hasher>(&self)
    }
    pub fn verify_marked<Hasher: MerkleHasher<Hash>>(&self) -> bool {
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
pub fn verify_merkle_proof_core<Hash: PartialEq + Copy, Hasher: MerkleHasher<Hash>>(
    proof: &MerkleProofCore<Hash>,
) -> bool {
    let mut current = proof.value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if proof.index & (1 << i) == 0 {
            current = Hasher::two_to_one(sibling, &current);
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
            current = Hasher::two_to_one(sibling, &current);
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
            current = Hasher::two_to_one(sibling, &current);
        } else {
            current = Hasher::two_to_one(sibling, &current);
        }
    }
    current == proof.new_root
}

pub fn verify_merkle_proof_marked_leaves_core<
    Hash: PartialEq + Copy,
    Hasher: MerkleHasher<Hash>,
>(
    proof: &MerkleProofCore<Hash>,
) -> bool {
    let mut current = proof.value;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        if proof.index & (1 << i) == 0 {
            current = Hasher::two_to_one(sibling, &current);
        } else {
            current = Hasher::two_to_one(sibling, &current);
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
                current = Hasher::two_to_one_marked_leaf(sibling, &current);
            } else {
                current = Hasher::two_to_one_marked_leaf(sibling, &current);
            }
        } else {
            if proof.index & (1 << i) == 0 {
                current = Hasher::two_to_one(sibling, &current);
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
        if proof.index & (1 << i) == 0 {
            current = Hasher::two_to_one(sibling, &current);
        } else {
            current = Hasher::two_to_one(sibling, &current);
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