use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::proof::L2OAVerifierData;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumIs;
use strum::EnumString;

use crate::operation::l2o_a::block::L2OABlockInscription;
use crate::operation::l2o_a::deploy::L2OADeployInscription;

pub mod block;
pub mod deploy;

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

#[derive(Debug, Clone, PartialEq)]
pub enum L2OAOperation {
    Deploy(L2OADeployInscription<serde_json::Value>),
    Block(L2OABlockInscription<serde_json::Value>),
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum RawL2OAOperation {
    #[serde(rename = "deploy")]
    Deploy(L2OADeployInscription<serde_json::Value>),
    #[serde(rename = "block")]
    Block(L2OABlockInscription<serde_json::Value>),
}

pub type L2OADeployInscriptionV1 = L2OADeployInscription<L2OAVerifierData>;
pub type L2OABlockInscriptionV1 = L2OABlockInscription<L2OAProofData>;

use l2o_crypto::proof::L2OAProofSerializableData;
use l2o_crypto::proof::L2OAVerifierSerializableData;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "op")]
pub enum L2OAInscription {
    Deploy(L2OAInscriptionDeploy),
    Block(L2OAInscriptionBlock),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct L2OAInscriptionDeploy {
    pub l2id: u32,
    pub start_state_root: String,
    pub public_key: String,
    pub vk: L2OAVerifierSerializableData,
    pub hash_function: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct L2OAInscriptionBlockParameters {
    pub state_root: String,
    pub public_key: String,
    pub deposits_root: String,
    pub withdrawals_root: String,
    pub block_number: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct L2OAInscriptionBlock {
    pub l2id: u32,
    pub block_parameters: L2OAInscriptionBlockParameters,
    pub proof: L2OAProofSerializableData,
    pub bitcoin_block_number: u64,
    pub bitcoin_block_hash: String,
    pub superchain_root: String,
    pub signature: String,
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_serialize_deploy() {
//         let deploy_json = include_str!("../../../assets/deploy.json");
//         let p =
// serde_json::from_str::<L2OAInscription>(deploy_json).unwrap();
//         assert!(matches!(p, L2OAInscription::Deploy(_)));
//         tracing::info!("{:?}", p);
//     }
//
//     #[test]
//     fn test_serialize_block() {
//         let block_json = include_str!("../../../assets/block.json");
//         let p = serde_json::from_str::<L2OAInscription>(block_json).unwrap();
//         assert!(matches!(p, L2OAInscription::Block(_)));
//         tracing::info!("{:?}", p);
//     }
// }
