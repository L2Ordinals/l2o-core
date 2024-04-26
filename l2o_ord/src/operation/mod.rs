use serde::Deserialize;
use serde::Serialize;

use crate::operation::brc20::BRC20Operation;
use crate::operation::brc20::RawBRC20Operation;
use crate::operation::brc21::BRC21Operation;
use crate::operation::brc21::RawBRC21Operation;
use crate::operation::l2o_a::L2OAOperation;
use crate::operation::l2o_a::RawL2OAOperation;

pub mod brc20;
pub mod brc21;
pub mod l2o_a;

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    BRC20(BRC20Operation),
    BRC21(BRC21Operation),
    L2OA(L2OAOperation),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, strum::Display)]
#[strum(serialize_all = "camelCase")]
pub enum OperationType {
    BRC20Deploy,
    BRC20Mint,
    BRC20InscribeTransfer,
    BRC20Transfer,
    BRC21Deploy,
    BRC21Mint,
    BRC21InscribeTransfer,
    BRC21Transfer,
    BRC21L2Deposit,
    BRC21L2Withdraw,
    L2OABlock,
    L2ODeploy,
}

impl Operation {
    pub fn op_type(&self) -> OperationType {
        match self {
            Operation::BRC20(BRC20Operation::Deploy(_)) => OperationType::BRC20Deploy,
            Operation::BRC20(BRC20Operation::Mint { .. }) => OperationType::BRC20Mint,
            Operation::BRC20(BRC20Operation::InscribeTransfer(_)) => {
                OperationType::BRC20InscribeTransfer
            }
            Operation::BRC20(BRC20Operation::Transfer(_)) => OperationType::BRC20Transfer,
            Operation::BRC21(BRC21Operation::Deploy(_)) => OperationType::BRC21Deploy,
            Operation::BRC21(BRC21Operation::Mint { .. }) => OperationType::BRC21Mint,
            Operation::BRC21(BRC21Operation::InscribeTransfer(_)) => {
                OperationType::BRC21InscribeTransfer
            }
            Operation::BRC21(BRC21Operation::Transfer(_)) => OperationType::BRC21Transfer,
            Operation::BRC21(BRC21Operation::L2Deposit(_)) => OperationType::BRC21L2Deposit,
            Operation::BRC21(BRC21Operation::L2Withdraw(_)) => OperationType::BRC21L2Withdraw,
            Operation::L2OA(L2OAOperation::Deploy(_)) => OperationType::L2ODeploy,
            Operation::L2OA(L2OAOperation::Block(_)) => OperationType::L2OABlock,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "p")]
pub enum RawOperation {
    #[serde(rename = "brc-20")]
    BRC20(RawBRC20Operation),
    #[serde(rename = "brc-21")]
    BRC21(RawBRC21Operation),
    #[serde(rename = "l2o-a")]
    L2OA(RawL2OAOperation),
}
