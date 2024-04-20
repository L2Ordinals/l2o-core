use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BRC20Deploy {
    pub tick: String,
    pub lim: String,
    pub max: String,
}

impl BRC20Deploy {
    pub fn new(tick: String, lim: String, max: String) -> Self {
        BRC20Deploy { tick, lim, max }
    }
}
