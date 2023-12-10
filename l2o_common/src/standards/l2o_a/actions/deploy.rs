use serde::{Serialize, Deserialize};

use crate::{common::data::{hash::Hash256, signature::{L2OSignature512, L2OCompactPublicKey}}, standards::l2o_a::supported_crypto::{L2OAHashFunction, L2OAProofType}};

/* 
fn default_p() -> String {
  "l2o-a".to_string()
}
fn default_op() -> String {
  "Deploy".to_string()
}
*/

#[derive(Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct L2ODeployInscription<V> {
  /*
    #[serde(default = "default_p")]
    pub p: String,
    #[serde(default = "default_op")]
    pub op: String,*/

    pub l2id: u64,
    pub public_key: L2OCompactPublicKey,

    pub start_state_root: Hash256,
    
    pub hash_function: L2OAHashFunction,
    pub proof_type: L2OAProofType,
    pub verifier_data: V,

    pub signature: L2OSignature512,
}
