use kvq::traits::KVQSerializable;
use serde::{Serialize, Deserialize};
use core::hash::Hash;
use crate::common::data::{hash::Hash256, signature::{L2OSignature512, L2OCompactPublicKey}};


fn default_p() -> String {
  "l2o-a".to_string()
}
fn default_op() -> String {
  "Block".to_string()
}


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct L2OBlockInscription<Proof> {
    #[serde(default = "default_p")]
    pub p: String,
    #[serde(default = "default_op")]
    pub op: String,

    pub l2id: u32,
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
