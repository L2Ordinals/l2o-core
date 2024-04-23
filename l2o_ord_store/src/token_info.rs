use l2o_ord::inscription::inscription_id::InscriptionId;
use serde::Deserialize;
use serde::Serialize;

use crate::script_key::ScriptKey;
use crate::tick::Tick;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TokenInfo {
    pub tick: Tick,
    pub inscription_id: InscriptionId,
    pub inscription_number: i32,
    pub supply: u128,
    pub burned_supply: u128,
    pub minted: u128,
    pub limit_per_mint: u128,
    pub decimal: u8,
    pub deploy_by: ScriptKey,
    pub is_self_mint: bool,
    pub deployed_number: u32,
    pub deployed_timestamp: u32,
    pub latest_mint_number: u32,
}
