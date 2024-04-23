use l2o_common::standards::brc20::deploy::BRC20Deploy;
use l2o_common::standards::brc20::mint::BRC20Mint;
use l2o_common::standards::brc20::transfer::BRC20Transfer;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "op")]
pub enum BRC20Inscription {
    #[serde(rename = "transfer")]
    Transfer(BRC20Transfer),
    #[serde(rename = "deploy")]
    Deploy(BRC20Deploy),
    #[serde(rename = "mint")]
    Mint(BRC20Mint),
}
