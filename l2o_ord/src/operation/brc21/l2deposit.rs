use kvq::traits::KVQSerializable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct L2Deposit {
    #[serde(rename = "l2id")]
    pub l2id: u64,
    #[serde(rename = "tick")]
    pub tick: String,
    pub to: String,
    #[serde(rename = "amt")]
    pub amount: String,
}

impl L2Deposit {
    pub fn new(l2id: u64, tick: String, to: String, amount: String) -> Self {
        L2Deposit {
            l2id,
            tick,
            amount,
            to,
        }
    }
}

impl KVQSerializable for L2Deposit {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self)?)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
