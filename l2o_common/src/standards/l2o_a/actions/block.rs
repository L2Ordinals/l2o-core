use kvq::traits::KVQSerializable;
use serde::Deserialize;
use serde::Serialize;

use crate::common::data::hash::Hash256;
use crate::common::data::signature::L2OCompactPublicKey;
use crate::common::data::signature::L2OSignature512;

fn default_p() -> String {
    "l2o-a".to_string()
}

fn default_op() -> String {
    "Block".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "Proof: Serialize, for<'de2> Proof: Deserialize<'de2>")]
pub struct L2OBlockInscription<Proof>
where
    Proof: Serialize,
    for<'de2> Proof: Deserialize<'de2>,
{
    #[serde(default = "default_p")]
    pub p: String,
    #[serde(default = "default_op")]
    pub op: String,

    pub l2id: u64,
    pub l2_block_number: u64,

    pub bitcoin_block_number: u64,
    pub bitcoin_block_hash: Hash256,

    pub public_key: L2OCompactPublicKey,

    pub start_state_root: Hash256,
    pub end_state_root: Hash256,

    pub deposit_state_root: Hash256,

    pub start_withdrawal_state_root: Hash256,
    pub end_withdrawal_state_root: Hash256,

    pub proof: Proof,

    pub superchain_root: Hash256,

    pub signature: L2OSignature512,
}

impl<V: Serialize + Clone + PartialEq> KVQSerializable for L2OBlockInscription<V>
where
    for<'de2> V: Deserialize<'de2>,
{
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
