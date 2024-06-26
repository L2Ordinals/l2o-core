use l2o_ord::inscription::inscription_id::InscriptionId;
use l2o_ord::script_key::ScriptKey;
use l2o_ord::tick::Tick;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TokenInfo {
    pub tick: Tick,
    pub inscription_id: InscriptionId,
    pub inscription_number: i32,
    #[serde_as(as = "DisplayFromStr")]
    pub supply: u128,
    #[serde_as(as = "DisplayFromStr")]
    pub burned_supply: u128,
    #[serde_as(as = "DisplayFromStr")]
    pub minted: u128,
    #[serde_as(as = "DisplayFromStr")]
    pub limit_per_mint: u128,
    pub decimal: u8,
    pub deploy_by: ScriptKey,
    pub is_self_mint: bool,
    pub deployed_number: u32,
    pub deployed_timestamp: u32,
    pub latest_mint_number: u32,
}
