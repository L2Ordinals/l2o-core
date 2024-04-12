use l2o_crypto::proof::groth16::bn128::proof_data::Groth16ProofSerializable;
use l2o_crypto::proof::groth16::bn128::verifier_data::Groth16VerifierDataSerializable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "op")]
pub enum L2OInscription {
    Deploy(L2OInscriptionDeploy),
    Block(L2OInscriptionBlock),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct L2OInscriptionDeploy {
    pub l2id: u32,
    pub start_state_root: String,
    pub public_key: String,
    pub vk: Groth16VerifierDataSerializable,
    pub hash_function: String,
    pub proof_type: String,
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
    pub l2id: u32,
    pub block_parameters: L2OInscriptionBlockParameters,
    pub proof: Groth16ProofSerializable,
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deploy() {
        let deploy_json = include_str!("../../assets/deploy.json");
        let p = serde_json::from_str::<L2OInscription>(deploy_json).unwrap();
        assert!(matches!(p, L2OInscription::Deploy(_)));
        tracing::info!("{:?}", p);
    }

    #[test]
    fn test_serialize_block() {
        let block_json = include_str!("../../assets/block.json");
        let p = serde_json::from_str::<L2OInscription>(block_json).unwrap();
        assert!(matches!(p, L2OInscription::Block(_)));
        tracing::info!("{:?}", p);
    }
}
