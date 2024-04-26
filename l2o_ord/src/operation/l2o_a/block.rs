use kvq::traits::KVQSerializable;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "Proof: Serialize, for<'de2> Proof: Deserialize<'de2>")]
pub struct L2OABlockInscription<Proof>
where
    Proof: Serialize,
    for<'de2> Proof: Deserialize<'de2>,
{
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

    #[serde(flatten)]
    pub proof: Proof,

    pub superchain_root: Hash256,

    pub signature: L2OSignature512,
}

impl<V: Serialize + Clone + PartialEq> KVQSerializable for L2OABlockInscription<V>
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
