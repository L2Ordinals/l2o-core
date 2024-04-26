use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct L2Deposit {
    #[serde(rename = "l2id")]
    pub l2id: String,
    #[serde(rename = "tick")]
    pub tick: String,
    pub to: String,
    #[serde(rename = "amt")]
    pub amount: String,
}

impl L2Deposit {
    pub fn new(l2id: String, tick: String, to: String, amount: String) -> Self {
        L2Deposit {
            l2id,
            tick,
            amount,
            to,
        }
    }
}
