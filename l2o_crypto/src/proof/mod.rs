pub mod groth16;
pub mod plonky2;

use serde::Deserialize;
use serde::Serialize;
use strum::EnumIs;
use strum::EnumTryAs;

use crate::proof::groth16::bn128::proof_data::Groth16ProofSerializable;
use crate::proof::groth16::bn128::verifier_data::Groth16VerifierSerializable;

#[derive(EnumIs, EnumTryAs, Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum L2OAVerifierSerializableData {
    Groth16VerifierSerializable(Groth16VerifierSerializable),
}

impl From<Groth16VerifierSerializable> for L2OAVerifierSerializableData {
    fn from(value: Groth16VerifierSerializable) -> Self {
        L2OAVerifierSerializableData::Groth16VerifierSerializable(value)
    }
}

#[derive(EnumIs, EnumTryAs, Clone, PartialEq, Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum L2OAProofSerializableData {
    Groth16ProofSerializable(Groth16ProofSerializable),
}

impl From<Groth16ProofSerializable> for L2OAProofSerializableData {
    fn from(value: Groth16ProofSerializable) -> Self {
        L2OAProofSerializableData::Groth16ProofSerializable(value)
    }
}
