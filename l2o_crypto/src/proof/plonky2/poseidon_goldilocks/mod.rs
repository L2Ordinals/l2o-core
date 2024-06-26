use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::HashOut;
use plonky2::hash::merkle_tree::MerkleCap;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::circuit_data::VerifierOnlyCircuitData;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::proof::ProofWithPublicInputs;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

pub type Plonky2PoseidonGoldilocksProofData =
    ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>;

#[derive(Clone, PartialEq, Debug, Serialize)]
pub struct Plonky2PoseidonGoldilocksVerifierData(
    pub VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>,
);

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct VerifierOnlyCircuitDataDeserializable {
    pub constants_sigmas_cap: MerkleCap<GoldilocksField, PoseidonHash>,
    pub circuit_digest: HashOut<GoldilocksField>,
}

impl<'de> Deserialize<'de> for Plonky2PoseidonGoldilocksVerifierData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = VerifierOnlyCircuitDataDeserializable::deserialize(deserializer)?;
        Ok(Plonky2PoseidonGoldilocksVerifierData(
            VerifierOnlyCircuitData {
                constants_sigmas_cap: raw.constants_sigmas_cap,
                circuit_digest: raw.circuit_digest,
            },
        ))
    }
}
