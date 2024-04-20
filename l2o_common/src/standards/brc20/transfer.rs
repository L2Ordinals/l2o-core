use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BRC20Transfer {
    pub tick: String,
    pub amt: String,
}

impl BRC20Transfer {
    pub fn new(tick: String, amt: String) -> Self {
        BRC20Transfer { tick, amt }
    }
}
