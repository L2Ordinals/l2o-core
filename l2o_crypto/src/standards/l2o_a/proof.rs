use serde::Deserialize;
use serde::Serialize;

use crate::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use crate::proof::groth16::bn128::verifier_data::Groth16BN128VerifierData;
use crate::proof::plonky2::poseidon_goldilocks::Plonky2PoseidonGoldilocksProofData;
use crate::proof::plonky2::poseidon_goldilocks::Plonky2PoseidonGoldilocksVerifierData;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(tag = "proof_type", content = "proof")]
pub enum L2OAProofData {
    Groth16BN128(Groth16BN128ProofData),
    Plonky2PoseidonGoldilocks(Plonky2PoseidonGoldilocksProofData),
}

impl L2OAProofData {
    pub fn as_groth16_bn128(self) -> Groth16BN128ProofData {
        match self {
            L2OAProofData::Groth16BN128(x) => x,
            L2OAProofData::Plonky2PoseidonGoldilocks(_) => unreachable!(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(tag = "proof_type", content = "verifier_data")]
pub enum L2OAVerifierData {
    Groth16BN128(Groth16BN128VerifierData),
    Plonky2PoseidonGoldilocks(Plonky2PoseidonGoldilocksVerifierData),
}
