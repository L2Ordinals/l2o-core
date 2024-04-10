use kvq::traits::KVQSerializable;
use serde::Deserialize;
use serde::Serialize;

use crate::common::data::hash::Hash256;
use crate::common::data::signature::L2OCompactPublicKey;
use crate::standards::l2o_a::supported_crypto::L2OAHashFunction;
use crate::standards::l2o_a::supported_crypto::L2OAProofType;

fn default_p() -> String {
    "l2o-a".to_string()
}

fn default_op() -> String {
    "Deploy".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "V: Serialize, for<'de2> V: Deserialize<'de2>")]
pub struct L2ODeployInscription<V>
where
    V: Serialize,
    for<'de2> V: Deserialize<'de2>,
{
    #[serde(default = "default_p")]
    pub p: String,
    #[serde(default = "default_op")]
    pub op: String,
    pub l2id: u64,
    pub public_key: L2OCompactPublicKey,

    pub start_state_root: Hash256,

    pub hash_function: L2OAHashFunction,
    pub proof_type: L2OAProofType,
    pub verifier_data: V,
}

impl<V: Serialize + Clone + PartialEq> KVQSerializable for L2ODeployInscription<V>
where
    for<'de2> V: Deserialize<'de2>,
{
    fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        serde_json::from_slice(bytes).unwrap()
    }
}
