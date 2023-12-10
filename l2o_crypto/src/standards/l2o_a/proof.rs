use serde::{Serialize, Deserialize};

use crate::proof::{groth16::bn128::{proof_data::Groth16BN128ProofData, verifier_data::Groth16BN128VerifierData}, plonky2::poseidon_goldilocks::{Plonky2PoseidonGoldilocksProofData, Plonky2PoseidonGoldilocksVerifierData}};


#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(tag = "proof_type", content = "proof")]
pub enum L2OAProofData {
    Groth16BN128(Groth16BN128ProofData),
    Plonky2PoseidonGoldilocks(Plonky2PoseidonGoldilocksProofData),
}


#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(tag = "proof_type", content = "verifier_data")]
pub enum L2OAVerifierData {
    Groth16BN128(Groth16BN128VerifierData),
    Plonky2PoseidonGoldilocks(Plonky2PoseidonGoldilocksVerifierData),
}