use l2o_ord::tick::Tick;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Balance {
    pub tick: Tick,
    #[serde_as(as = "DisplayFromStr")]
    pub overall_balance: u128,
    #[serde_as(as = "DisplayFromStr")]
    pub transferable_balance: u128,
}

impl Balance {
    pub fn new(tick: &Tick) -> Self {
        Self {
            tick: tick.clone(),
            overall_balance: 0u128,
            transferable_balance: 0u128,
        }
    }
}
