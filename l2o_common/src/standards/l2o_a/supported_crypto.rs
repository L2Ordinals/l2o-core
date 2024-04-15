use serde::Deserialize;
use serde::Serialize;
use strum::EnumIs;
use strum::EnumString;

#[derive(EnumIs, EnumString, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum L2OAHashFunction {
    Sha256,
    BLAKE3,
    Keccak256,
    PoseidonGoldilocks,
}

#[derive(EnumIs, EnumString, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum L2OAProofType {
    Groth16BN128,
    Plonky2PoseidonGoldilocks,
}
