use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct L2Deposit {
    #[serde(rename = "tick")]
    pub tick: String,
    pub to: String,
    #[serde(rename = "amt")]
    pub amount: String,
}

impl L2Deposit {
    pub fn new(tick: String, to: String, amount: String) -> Self {
        L2Deposit { tick, amount, to }
    }
}
