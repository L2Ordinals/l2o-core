use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound = "Proof: Serialize, for<'de2> Proof: Deserialize<'de2>")]
pub struct L2Withdraw<Proof>
where
    Proof: Serialize,
    for<'de2> Proof: Deserialize<'de2>,
{
    #[serde(rename = "l2id")]
    pub l2id: String,
    #[serde(rename = "tick")]
    pub tick: String,
    pub to: String,
    #[serde(rename = "amt")]
    pub amount: String,
    pub proof: Proof,
}

impl<Proof> L2Withdraw<Proof>
where
    Proof: Serialize,
    for<'de2> Proof: Deserialize<'de2>,
{
    pub fn new(l2id: String, tick: String, to: String, amount: String, proof: Proof) -> Self {
        L2Withdraw {
            l2id,
            tick,
            to,
            amount,
            proof,
        }
    }
}
