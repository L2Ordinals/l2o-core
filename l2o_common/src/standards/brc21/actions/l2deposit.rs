use serde::{Deserialize, Serialize};

fn default_p() -> String {
  "brc-21".to_string()
}
fn default_op() -> String {
  "l2deposit".to_string()
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC21L2Deposit {
  #[serde(default = "default_p")]
  pub p: String,
  #[serde(default = "default_op")]
  pub op: String,

  pub tick: String,
  pub amt: String,
}

impl BRC21L2Deposit {
  pub fn new(tick: String, amt: String) -> Self {
    BRC21L2Deposit {
      p: "brc-21".to_string(),
      op: "l2deposit".to_string(),
      tick,
      amt,
    }
  }
  pub fn is_valid(&self) -> bool {
    self.p == "brc-21" && self.op == "l2deposit"
  }
}