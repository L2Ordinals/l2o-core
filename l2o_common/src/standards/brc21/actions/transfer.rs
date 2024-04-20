use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BRC21Transfer {
    pub tick: String,
    pub amt: String,
}

impl BRC21Transfer {
    pub fn new(tick: String, amt: String) -> Self {
        BRC21Transfer { tick, amt }
    }
}
