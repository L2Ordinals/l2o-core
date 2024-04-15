use serde::Deserialize;
use serde::Serialize;
use strum::EnumIs;
use strum::EnumTryAs;

use crate::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use crate::proof::groth16::bn128::verifier_data::Groth16BN128VerifierData;
use crate::proof::plonky2::poseidon_goldilocks::Plonky2PoseidonGoldilocksProofData;
use crate::proof::plonky2::poseidon_goldilocks::Plonky2PoseidonGoldilocksVerifierData;

#[derive(EnumIs, EnumTryAs, Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(tag = "proof_type", content = "proof")]
pub enum L2OAProofData {
    Groth16BN128(Groth16BN128ProofData),
    Plonky2PoseidonGoldilocks(Plonky2PoseidonGoldilocksProofData),
}

impl From<Groth16BN128ProofData> for L2OAProofData {
    fn from(value: Groth16BN128ProofData) -> Self {
        L2OAProofData::Groth16BN128(value)
    }
}

impl From<Plonky2PoseidonGoldilocksProofData> for L2OAProofData {
    fn from(value: Plonky2PoseidonGoldilocksProofData) -> Self {
        L2OAProofData::Plonky2PoseidonGoldilocks(value)
    }
}

#[derive(EnumIs, EnumTryAs, Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(tag = "proof_type", content = "verifier_data")]
pub enum L2OAVerifierData {
    Groth16BN128(Groth16BN128VerifierData),
    Plonky2PoseidonGoldilocks(Plonky2PoseidonGoldilocksVerifierData),
}

impl From<Groth16BN128VerifierData> for L2OAVerifierData {
    fn from(value: Groth16BN128VerifierData) -> Self {
        L2OAVerifierData::Groth16BN128(value)
    }
}

impl From<Plonky2PoseidonGoldilocksVerifierData> for L2OAVerifierData {
    fn from(value: Plonky2PoseidonGoldilocksVerifierData) -> Self {
        L2OAVerifierData::Plonky2PoseidonGoldilocks(value)
    }
}
