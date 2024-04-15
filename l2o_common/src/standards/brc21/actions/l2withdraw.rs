use serde::Deserialize;
use serde::Serialize;

use crate::common::data::hash::MerkleProofCommonHash256;

fn default_p() -> String {
    "brc-21".to_string()
}
fn default_op() -> String {
    "l2withdraw".to_string()
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC21L2Withdraw {
    #[serde(default = "default_p")]
    pub p: String,
    #[serde(default = "default_op")]
    pub op: String,

    pub tick: String,
    pub amt: String,
    pub proof: MerkleProofCommonHash256,
}

impl BRC21L2Withdraw {
    pub fn new(tick: String, amt: String, proof: MerkleProofCommonHash256) -> Self {
        BRC21L2Withdraw {
            p: "brc-21".to_string(),
            op: "l2withdraw".to_string(),
            tick,
            amt,
            proof,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.p == "brc-21" && self.op == "l2withdraw"
    }
}
