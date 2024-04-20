use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BRC20Mint {
    pub tick: String,
    pub amt: String,
}

impl BRC20Mint {
    pub fn new(tick: String, amt: String) -> Self {
        BRC20Mint { tick, amt }
    }
}
