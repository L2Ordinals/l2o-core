pub mod cli;
pub mod common;
pub mod error;
pub mod logger;
pub mod standards;

use std::str::FromStr;

use ark_bn254::Fq;
use ark_bn254::Fr;
pub use cli::*;
pub use error::Error;
pub use error::Result;
pub use logger::setup_logger;

pub fn str_to_fq(s: &str) -> Result<Fq> {
    Ok(Fq::from_str(if s.trim().is_empty() { "0" } else { s })
        .map_err(|_| anyhow::anyhow!("str to fq conversion failed"))?)
}

pub fn str_to_fr(s: &str) -> Result<Fr> {
    Ok(Fr::from_str(if s.trim().is_empty() { "0" } else { s })
        .map_err(|_| anyhow::anyhow!("str to fr conversion failed"))?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_fq() {
        let zero = Fq::from(0).to_string();
        assert_eq!(zero, "");
        assert_eq!(str_to_fq(&zero).unwrap(), Fq::from(0));

        let one = Fq::from(1).to_string();
        assert_eq!(one, "1");
        assert_eq!(str_to_fq(&one).unwrap(), Fq::from(1));
    }
}
