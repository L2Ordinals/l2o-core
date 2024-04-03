use serde::Deserialize;
use serde::Serialize;

use crate::proof::snarkjs::ProofJson;
use crate::proof::snarkjs::VerifyingKeyJson;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "op")]
pub enum L2OInscription {
    Deploy(L2OInscriptionDeploy),
    Block(L2OInscriptionBlock),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct L2OInscriptionDeploy {
    pub p: String,
    pub l2id: u32,
    pub start_state_root: String,
    pub public_key: String,
    pub vk: VerifyingKeyJson,
    pub hash_function: String,
    pub proof_type: String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct L2OInscriptionBlockParameters {
    pub state_root: String,
    pub public_key: String,
    pub deposits_root: String,
    pub withdrawals_root: String,
    pub block_number: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct L2OInscriptionBlock {
    pub p: String,
    pub l2id: u32,
    pub block_parameters: L2OInscriptionBlockParameters,
    pub proof: ProofJson,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_verify() {
        let deploy_json = include_str!("../../assets/deploy.json");
        let p = serde_json::from_str::<L2OInscription>(deploy_json).unwrap();
        tracing::info!("{:?}", p);
    }
}
