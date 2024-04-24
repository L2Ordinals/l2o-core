use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;

use crate::error::JSONError;
use crate::inscription::inscription_id::InscriptionId;
use crate::operation::deploy::Deploy;
use crate::operation::mint::Mint;
use crate::operation::transfer::Transfer;
use crate::PROTOCOL_LITERAL;

pub mod deploy;
pub mod mint;
pub mod transfer;

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Deploy(Deploy),
    Mint {
        mint: Mint,
        parent: Option<InscriptionId>,
    },
    InscribeTransfer(Transfer),
    Transfer(Transfer),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, strum::Display)]
#[strum(serialize_all = "camelCase")]
pub enum OperationType {
    Deploy,
    Mint,
    InscribeTransfer,
    Transfer,
}

impl Operation {
    pub fn op_type(&self) -> OperationType {
        match self {
            Operation::Deploy(_) => OperationType::Deploy,
            Operation::Mint { .. } => OperationType::Mint,
            Operation::InscribeTransfer(_) => OperationType::InscribeTransfer,
            Operation::Transfer(_) => OperationType::Transfer,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "op")]
pub enum RawOperation {
    #[serde(rename = "deploy")]
    Deploy(Deploy),
    #[serde(rename = "mint")]
    Mint(Mint),
    #[serde(rename = "transfer")]
    Transfer(Transfer),
}

pub fn deserialize_brc20(s: &str) -> Result<RawOperation, JSONError> {
    let value: Value = serde_json::from_str(s).map_err(|_| JSONError::InvalidJson)?;
    if value.get("p") != Some(&json!(PROTOCOL_LITERAL)) {
        return Err(JSONError::NotBRC20Json);
    }

    serde_json::from_value(value).map_err(|e| JSONError::ParseOperationJsonError(e.to_string()))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::okx::datastore::ord::Action;
//
//     #[test]
//     fn test_deploy_deserialize() {
//         let max_supply = "21000000".to_string();
//         let mint_limit = "1000".to_string();
//
//         let json_str = format!(
//             r##"{{
//   "p": "brc-20",
//   "op": "deploy",
//   "tick": "ordi",
//   "max": "{max_supply}",
//   "lim": "{mint_limit}"
// }}"##
//         );
//
//         assert_eq!(
//             deserialize_brc20(&json_str).unwrap(),
//             RawOperation::Deploy(Deploy {
//                 tick: "ordi".to_string(),
//                 max_supply,
//                 mint_limit: Some(mint_limit),
//                 decimals: None,
//                 self_mint: None,
//             })
//         );
//     }
//
//     #[test]
//     fn test_mint_deserialize() {
//         let amount = "1000".to_string();
//
//         let json_str = format!(
//             r##"{{
//   "p": "brc-20",
//   "op": "mint",
//   "tick": "ordi",
//   "amt": "{amount}"
// }}"##
//         );
//
//         assert_eq!(
//             deserialize_brc20(&json_str).unwrap(),
//             RawOperation::Mint(Mint {
//                 tick: "ordi".to_string(),
//                 amount,
//             })
//         );
//     }
//
//     #[test]
//     fn test_transfer_deserialize() {
//         let amount = "100".to_string();
//
//         let json_str = format!(
//             r##"{{
//   "p": "brc-20",
//   "op": "transfer",
//   "tick": "ordi",
//   "amt": "{amount}"
// }}"##
//         );
//
//         assert_eq!(
//             deserialize_brc20(&json_str).unwrap(),
//             RawOperation::Transfer(Transfer {
//                 tick: "ordi".to_string(),
//                 amount,
//             })
//         );
//     }
//     #[test]
//     fn test_json_duplicate_field() {
//         let json_str =
// r#"{"p":"brc-20","op":"mint","tick":"smol","amt":"333","amt":"33"}"#;
//         assert_eq!(
//             deserialize_brc20(json_str).unwrap(),
//             RawOperation::Mint(Mint {
//                 tick: String::from("smol"),
//                 amount: String::from("33"),
//             })
//         )
//     }
//
//     #[test]
//     fn test_json_non_string() {
//         let json_str =
// r#"{"p":"brc-20","op":"mint","tick":"smol","amt":33}"#;         assert!
// (deserialize_brc20(json_str).is_err())     }
//
//     #[test]
//     fn test_deserialize_case_insensitive() {
//         let max_supply = "21000000".to_string();
//         let mint_limit = "1000".to_string();
//
//         let json_str = format!(
//             r##"{{
//   "P": "brc-20",
//   "Op": "deploy",
//   "Tick": "ordi",
//   "mAx": "{max_supply}",
//   "Lim": "{mint_limit}"
// }}"##
//         );
//
//         assert_eq!(deserialize_brc20(&json_str),
// Err(JSONError::NotBRC20Json));     }
//     #[test]
//     fn test_ignore_non_transfer_brc20() {
//         let content_type = "text/plain;charset=utf-8";
//         let inscription = crate::inscription(
//             content_type,
//
// r#"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"
// 11"}"#,         );
//         assert_eq!(
//             deserialize_brc20_operation(
//                 &inscription,
//                 &Action::New {
//                     cursed: false,
//                     unbound: false,
//                     vindicated: false,
//                     parent: None,
//                     inscription: inscription.clone()
//                 },
//             )
//             .unwrap(),
//             Operation::Deploy(Deploy {
//                 tick: "abcd".to_string(),
//                 max_supply: "12000".to_string(),
//                 mint_limit: Some("12".to_string()),
//                 decimals: Some("11".to_string()),
//                 self_mint: None,
//             }),
//         );
//         let inscription = crate::inscription(
//             content_type,
//             r#"{"p":"brc-20","op":"mint","tick":"abcd","amt":"12000"}"#,
//         );
//
//         assert_eq!(
//             deserialize_brc20_operation(
//                 &inscription,
//                 &Action::New {
//                     cursed: false,
//                     unbound: false,
//                     vindicated: false,
//                     parent: None,
//                     inscription: inscription.clone()
//                 },
//             )
//             .unwrap(),
//             Operation::Mint {
//                 mint: Mint {
//                     tick: "abcd".to_string(),
//                     amount: "12000".to_string()
//                 },
//                 parent: None
//             }
//         );
//         let inscription = crate::inscription(
//             content_type,
//             r#"{"p":"brc-20","op":"transfer","tick":"abcd","amt":"12000"}"#,
//         );
//
//         assert_eq!(
//             deserialize_brc20_operation(
//                 &inscription,
//                 &Action::New {
//                     cursed: false,
//                     unbound: false,
//                     vindicated: false,
//                     parent: None,
//                     inscription: inscription.clone()
//                 },
//             )
//             .unwrap(),
//             Operation::InscribeTransfer(Transfer {
//                 tick: "abcd".to_string(),
//                 amount: "12000".to_string()
//             })
//         );
//
//         let inscription = crate::inscription(
//             content_type,
//
// r#"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"
// 11"}"#,         );
//         assert!(deserialize_brc20_operation(&inscription,
// &Action::Transfer).is_err());
//
//         let inscription = crate::inscription(
//             content_type,
//             r#"{"p":"brc-20","op":"mint","tick":"abcd","amt":"12000"}"#,
//         );
//         assert!(deserialize_brc20_operation(&inscription,
// &Action::Transfer).is_err());         let inscription = crate::inscription(
//             content_type,
//             r#"{"p":"brc-20","op":"transfer","tick":"abcd","amt":"12000"}"#,
//         );
//         assert_eq!(
//             deserialize_brc20_operation(&inscription,
// &Action::Transfer).unwrap(),             Operation::Transfer(Transfer {
//                 tick: "abcd".to_string(),
//                 amount: "12000".to_string()
//             })
//         );
//     }
// }
