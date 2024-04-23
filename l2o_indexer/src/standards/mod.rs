use serde::Deserialize;
use serde::Serialize;

pub mod brc20;
pub mod brc21;
pub mod l2o_a;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(tag = "p")]
pub enum L2OInscription {
    #[serde(rename = "brc-20")]
    BRC20(brc20::inscription::BRC20Inscription),
    #[serde(rename = "brc-21")]
    BRC21(brc21::inscription::BRC21Inscription),
    #[serde(rename = "l2o-a")]
    L2OA(l2o_a::inscription::L2OAInscription),
}
