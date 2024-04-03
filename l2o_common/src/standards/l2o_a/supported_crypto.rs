use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum L2OAHashFunction {
    Sha256,
    BLAKE3,
    Keccack256,
    PoseidonGoldilocks,
}

impl FromStr for L2OAHashFunction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sha256" => Ok(L2OAHashFunction::Sha256),
            "blake3"=> Ok(L2OAHashFunction::BLAKE3),
            "keccack256" => Ok(L2OAHashFunction::Keccack256),
            "poseidon_goldilocks" => Ok(L2OAHashFunction::PoseidonGoldilocks),
            _ => Err(s.to_string())
        }
    }
}


#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum L2OAProofType {
    Groth16BN128,
    Plonky2PoseidonGoldilocks,
}

impl FromStr for L2OAProofType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "groth16_bn128" => Ok(L2OAProofType::Groth16BN128),
            "plonky2_poseidon_goldilocks" => Ok(L2OAProofType::Plonky2PoseidonGoldilocks),
            _ => Err(s.to_string())
        }
    }
}
