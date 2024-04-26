use kvq::traits::KVQSerializable;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use serde::Deserialize;
use serde::Serialize;

use crate::operation::l2o_a::L2OAHashFunction;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "V: Serialize, for<'de2> V: Deserialize<'de2>")]
pub struct L2OADeployInscription<V>
where
    V: Serialize,
    for<'de2> V: Deserialize<'de2>,
{
    pub l2id: u64,
    pub public_key: L2OCompactPublicKey,

    pub start_state_root: Hash256,

    pub hash_function: L2OAHashFunction,

    #[serde(flatten)]
    pub verifier_data: V,
}

impl<V: Serialize + Clone + PartialEq> KVQSerializable for L2OADeployInscription<V>
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
