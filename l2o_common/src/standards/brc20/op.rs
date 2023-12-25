use serde::{Serialize, Deserialize};


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct BRC20Operation {
  pub p: String,
  pub op: String,
  pub tick: String,
  pub amt: String,
}

impl BRC20Operation {
  pub fn new_transfer(tick: String, amt: String) -> Self {
    BRC20Operation {
      p: "brc-20".to_string(),
      op: "transfer".to_string(),
      tick,
      amt,
    }
  }
  pub fn new_mint(tick: String, amt: String) -> Self {
    BRC20Operation {
      p: "brc-20".to_string(),
      op: "mint".to_string(),
      tick,
      amt,
    }
  }
  pub fn is_transfer(&self) -> bool {
    self.p == "brc-20" && self.op == "transfer"
  }
  pub fn is_mint(&self) -> bool {
    self.p == "brc-20" && self.op == "mint"
  }
  pub fn is_valid(&self) -> bool {
    self.p == "brc-20" && (self.op == "transfer" || self.op == "mint")
  }
}