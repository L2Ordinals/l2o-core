use l2o_common::standards::brc21::actions::l2deposit::BRC21L2Deposit;
use l2o_common::standards::brc21::actions::l2withdraw::BRC21L2Withdraw;
use l2o_common::standards::brc21::actions::transfer::BRC21Transfer;
use l2o_crypto::hash::merkle::core::MerkleProofCommonHash256;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "op")]
pub enum BRC21Inscription {
    #[serde(rename = "l2deposit")]
    L2Deposit(BRC21L2Deposit),
    #[serde(rename = "l2withdraw")]
    L2Withdraw(BRC21L2Withdraw<MerkleProofCommonHash256>),
    #[serde(rename = "transfer")]
    Transfer(BRC21Transfer),
}
