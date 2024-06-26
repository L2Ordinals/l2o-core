use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Mint {
    #[serde(rename = "tick")]
    pub tick: String,
    #[serde(rename = "amt")]
    pub amount: String,
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::error::JSONError;

    #[test]
    fn test_serialize() {
        let obj = Mint {
            tick: "abcd".to_string(),
            amount: "22".to_string(),
        };
        assert_eq!(
            serde_json::to_string(&obj).unwrap(),
            r#"{"tick":"abcd","amt":"22"}"#
        );
    }

    #[test]
    fn test_deserialize() {
        assert_eq!(
            deserialize_brc20(r#"{"p":"brc-20","op":"mint","tick":"abcd","amt":"12000"}"#).unwrap(),
            RawBRC20Operation::Mint(Mint {
                tick: "abcd".to_string(),
                amount: "12000".to_string()
            })
        );
    }

    #[test]
    fn test_loss_require_key() {
        assert_eq!(
            deserialize_brc20(r#"{"p":"brc-20","op":"mint","tick":"abcd"}"#).unwrap_err(),
            JSONError::ParseOperationJsonError("missing field `amt`".to_string())
        );
    }

    #[test]
    fn test_duplicate_key() {
        let json_str = r#"{"p":"brc-20","op":"mint","tick":"smol","amt":"100","tick":"hhaa","amt":"200","tick":"actt"}"#;
        assert_eq!(
            deserialize_brc20(json_str).unwrap(),
            RawBRC20Operation::Mint(Mint {
                tick: "actt".to_string(),
                amount: "200".to_string(),
            })
        );
    }
}
