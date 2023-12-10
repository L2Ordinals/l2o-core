use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum L2OAHashFunction {
  Sha256,
  BLAKE3,
  Keccack256,
  PoseidonGoldilocks,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum L2OAProofType {
  Groth16BN128,
  Plonky2PoseidonGoldilocks,
}