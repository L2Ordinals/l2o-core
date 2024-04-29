use serde::Deserialize;
use serde::Serialize;

use crate::error::JSONError;
use crate::inscription::inscription::Inscription;
use crate::inscription::inscription_id::InscriptionId;
use crate::operation::brc20::BRC20Operation;
use crate::operation::brc20::RawBRC20Operation;
use crate::operation::brc21::BRC21Operation;
use crate::operation::brc21::RawBRC21Operation;
use crate::operation::l2o_a::L2OAOperation;
use crate::operation::l2o_a::RawL2OAOperation;
use crate::operation::Operation;
use crate::operation::RawOperation;

// the act of marking an inscription.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Action {
    New {
        cursed: bool,
        unbound: bool,
        #[serde(skip)]
        inscription: Inscription,
        #[serde(skip)]
        vindicated: bool,
        #[serde(skip)]
        parent: Option<InscriptionId>,
    },
    Transfer,
}

pub fn deserialize_operation(
    inscription: &Inscription,
    action: &Action,
) -> anyhow::Result<Operation> {
    let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
    if content_body.len() < 40 {
        return Err(JSONError::NotBRC2XJson.into());
    }

    let content_type = inscription
        .content_type()
        .ok_or(JSONError::InvalidContentType)?;

    if content_type != "text/plain"
        && content_type != "text/plain;charset=utf-8"
        && content_type != "text/plain;charset=UTF-8"
        && content_type != "application/json"
        && !content_type.starts_with("text/plain;")
    {
        return Err(JSONError::UnSupportContentType.into());
    }
    let raw_operation = match serde_json::from_str::<RawOperation>(content_body) {
        Ok(op) => op,
        Err(e) => {
            return Err(e.into());
        }
    };

    match action {
        Action::New { parent, .. } => match raw_operation {
            RawOperation::BRC20(RawBRC20Operation::Deploy(deploy)) => {
                Ok(Operation::BRC20(BRC20Operation::Deploy(deploy)))
            }
            RawOperation::BRC20(RawBRC20Operation::Mint(mint)) => {
                Ok(Operation::BRC20(BRC20Operation::Mint {
                    mint,
                    parent: *parent,
                }))
            }
            RawOperation::BRC20(RawBRC20Operation::Transfer(transfer)) => {
                Ok(Operation::BRC20(BRC20Operation::InscribeTransfer(transfer)))
            }
            RawOperation::BRC21(RawBRC21Operation::Deploy(deploy)) => {
                Ok(Operation::BRC21(BRC21Operation::Deploy(deploy)))
            }
            RawOperation::BRC21(RawBRC21Operation::Mint(mint)) => {
                Ok(Operation::BRC21(BRC21Operation::Mint {
                    mint,
                    parent: *parent,
                }))
            }
            RawOperation::BRC21(RawBRC21Operation::Transfer(transfer)) => {
                Ok(Operation::BRC21(BRC21Operation::InscribeTransfer(transfer)))
            }
            RawOperation::BRC21(RawBRC21Operation::L2Deposit(l2deposit)) => {
                Ok(Operation::BRC21(BRC21Operation::L2Deposit(l2deposit)))
            }
            RawOperation::BRC21(RawBRC21Operation::L2Withdraw(l2withdraw)) => {
                Ok(Operation::BRC21(BRC21Operation::L2Withdraw(l2withdraw)))
            }
            RawOperation::L2OA(RawL2OAOperation::Deploy(deploy)) => {
                Ok(Operation::L2OA(L2OAOperation::Deploy(deploy)))
            }
            RawOperation::L2OA(RawL2OAOperation::Block(block)) => {
                Ok(Operation::L2OA(L2OAOperation::Block(block)))
            }
        },
        Action::Transfer => match raw_operation {
            RawOperation::BRC20(RawBRC20Operation::Transfer(transfer)) => {
                Ok(Operation::BRC20(BRC20Operation::Transfer(transfer)))
            }
            RawOperation::BRC21(RawBRC21Operation::Transfer(transfer)) => {
                Ok(Operation::BRC21(BRC21Operation::Transfer(transfer)))
            }
            _ => Err(JSONError::NotBRC2XJson.into()),
        },
    }
}
