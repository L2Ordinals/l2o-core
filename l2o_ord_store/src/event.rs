use l2o_ord::error::BRC2XError;
use l2o_ord::inscription::inscription_id::InscriptionId;
use l2o_ord::operation::OperationType;
use l2o_ord::sat_point::SatPoint;
use l2o_ord::script_key::ScriptKey;
use l2o_ord::tick::Tick;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Receipt {
    pub inscription_id: InscriptionId,
    pub inscription_number: i32,
    pub old_satpoint: SatPoint,
    pub new_satpoint: SatPoint,
    pub op: OperationType,
    pub from: ScriptKey,
    pub to: ScriptKey,
    pub result: Result<Event, BRC2XError>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Event {
    Deploy(DeployEvent),
    Mint(MintEvent),
    InscribeTransfer(InscribeTransferEvent),
    Transfer(TransferEvent),
    L2Deposit(L2DepositEvent),
    L2Withdraw(L2WithdrawEvent),
    L2OADeploy,
    L2OABlock,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployEvent {
    pub supply: u128,
    pub limit_per_mint: u128,
    pub decimal: u8,
    pub tick: Tick,
    pub self_mint: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
    pub tick: Tick,
    pub amount: u128,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InscribeTransferEvent {
    pub tick: Tick,
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferEvent {
    pub tick: Tick,
    pub amount: u128,
    pub msg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct L2DepositEvent {
    pub l2id: u64,
    pub tick: String,
    pub to: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct L2WithdrawEvent {
    pub l2id: u64,
    pub tick: String,
    pub to: String,
    pub amount: String,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::Address;

    use super::*;

    #[test]
    fn action_receipt_serialize() {
        let action_receipt = Receipt {
            inscription_id: InscriptionId::from_str(
                "9991111111111111111111111111111111111111111111111111111111111111i1",
            )
            .unwrap(),
            inscription_number: 1,
            old_satpoint: SatPoint::from_str(
                "1111111111111111111111111111111111111111111111111111111111111111:1:1",
            )
            .unwrap(),
            new_satpoint: SatPoint::from_str(
                "2111111111111111111111111111111111111111111111111111111111111111:1:1",
            )
            .unwrap(),
            op: OperationType::BRC20Deploy,
            from: ScriptKey::from_address(
                Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
                    .unwrap()
                    .assume_checked(),
            ),
            to: ScriptKey::from_address(
                Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
                    .unwrap()
                    .assume_checked(),
            ),
            result: Err(BRC2XError::InvalidTickLen("abcde".to_string())),
        };
        println!("{}", serde_json::to_string_pretty(&action_receipt).unwrap());
        assert_eq!(
            serde_json::to_string_pretty(&action_receipt).unwrap(),
            r#"{
  "inscription_id": "9991111111111111111111111111111111111111111111111111111111111111i1",
  "inscription_number": 1,
  "old_satpoint": "1111111111111111111111111111111111111111111111111111111111111111:1:1",
  "new_satpoint": "2111111111111111111111111111111111111111111111111111111111111111:1:1",
  "op": "BRC20Deploy",
  "from": {
    "Address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "Address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "result": {
    "Err": {
      "InvalidTickLen": "abcde"
    }
  }
}"#
        );
    }
}
