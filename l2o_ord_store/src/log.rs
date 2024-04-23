use l2o_ord::inscription::inscription_id::InscriptionId;
use serde::Deserialize;
use serde::Serialize;

use crate::script_key::ScriptKey;
use crate::tick::Tick;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransferableLog {
    pub inscription_id: InscriptionId,
    pub inscription_number: i32,
    pub amount: u128,
    pub tick: Tick,
    pub owner: ScriptKey,
}
