use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BRC21L2Deposit {
    pub tick: String,
    pub to: String,
    pub amt: String,
}

impl BRC21L2Deposit {
    pub fn new(tick: String, to: String, amt: String) -> Self {
        BRC21L2Deposit { tick, amt, to }
    }
}
