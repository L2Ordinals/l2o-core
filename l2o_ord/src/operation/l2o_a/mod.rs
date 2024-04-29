use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::proof::L2OAVerifierData;
use serde::Deserialize;
use serde::Serialize;
use strum::EnumIs;
use strum::EnumString;

use crate::operation::l2o_a::block::Block;
use crate::operation::l2o_a::deploy::Deploy;

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

pub type L2OADeployV1 = Deploy<L2OAVerifierData>;
pub type L2OABlockV1 = Block<L2OAProofData>;

#[derive(Debug, Clone, PartialEq)]
pub enum L2OAOperation {
    Deploy(L2OADeployV1),
    Block(L2OABlockV1),
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum RawL2OAOperation {
    #[serde(rename = "Deploy")]
    Deploy(L2OADeployV1),
    #[serde(rename = "Block")]
    Block(L2OABlockV1),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deploy() {
        let deploy_json = include_str!("../../../../static/deploy.json");
        let p = serde_json::from_str::<RawL2OAOperation>(deploy_json).unwrap();
        assert!(matches!(p, RawL2OAOperation::Deploy(_)));
    }

    #[test]
    fn test_serialize_block() {
        let block_json = include_str!("../../../../static/block.json");
        let p = serde_json::from_str::<RawL2OAOperation>(block_json).unwrap();
        assert!(matches!(p, RawL2OAOperation::Block(_)));
    }
}
