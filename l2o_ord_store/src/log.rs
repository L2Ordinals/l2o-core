use l2o_ord::inscription::inscription_id::InscriptionId;
use l2o_ord::script_key::ScriptKey;
use l2o_ord::tick::Tick;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

#[serde_as]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TransferableLog {
    pub inscription_id: InscriptionId,
    pub inscription_number: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub amount: u128,
    pub tick: Tick,
    pub owner: ScriptKey,
}
