use bitcoin::address::NetworkUnchecked;
use bitcoin::opcodes;
use bitcoin::script::PushBytesBuf;
use bitcoin::script::{self};
use bitcoin::Address;
use bitcoin::Amount;
use bitcoin::BlockHash;
use bitcoin::OutPoint;
use bitcoin::ScriptBuf;
use bitcoin::Sequence;
use bitcoin::TxIn;
use bitcoin::TxOut;
use bitcoin::Txid;
use bitcoin::Witness;

use crate::inscription::inscription::Inscription;
use crate::inscription::inscription_id::InscriptionId;
use crate::sat_point::SatPoint;

#[macro_export]
macro_rules! assert_regex_match {
    ($value:expr, $pattern:expr $(,)?) => {
        let regex = regex::Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
        let string = $value.to_string();

        if !regex.is_match(string.as_ref()) {
            panic!(
                "Regex:\n\n{}\n\n…did not match string:\n\n{}",
                regex, string
            );
        }
    };
}

#[macro_export]
macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}

pub fn blockhash(n: u64) -> BlockHash {
    let hex = format!("{n:x}");

    if hex.is_empty() || hex.len() > 1 {
        panic!();
    }

    hex.repeat(64).parse().unwrap()
}

pub fn txid(n: u64) -> Txid {
    let hex = format!("{n:x}");

    if hex.is_empty() || hex.len() > 1 {
        panic!();
    }

    hex.repeat(64).parse().unwrap()
}

pub fn outpoint(n: u64) -> OutPoint {
    format!("{}:{}", txid(n), n).parse().unwrap()
}

pub fn satpoint(n: u64, offset: u64) -> SatPoint {
    SatPoint {
        outpoint: outpoint(n),
        offset,
    }
}

pub fn address() -> Address {
    "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked()
}

pub fn recipient() -> Address {
    "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked()
}

pub fn change(n: u64) -> Address {
    match n {
        0 => "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww",
        1 => "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l",
        2 => "tb1qxz9yk0td0yye009gt6ayn7jthz5p07a75luryg",
        3 => "tb1qe62s57n77pfhlw2vtqlhm87dwj75l6fguavjjq",
        _ => panic!(),
    }
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .assume_checked()
}

pub fn tx_in(previous_output: OutPoint) -> TxIn {
    TxIn {
        previous_output,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
    }
}

pub fn tx_out(value: u64, address: Address) -> TxOut {
    TxOut {
        value: Amount::from_sat(value),
        script_pubkey: address.script_pubkey(),
    }
}

#[derive(Default, Debug)]
pub struct InscriptionTemplate {
    pub parent: Option<InscriptionId>,
    pub pointer: Option<u64>,
}

impl From<InscriptionTemplate> for Inscription {
    fn from(template: InscriptionTemplate) -> Self {
        Self {
            parent: template.parent.map(|id| id.value()),
            pointer: template.pointer.map(Inscription::pointer_value),
            ..Default::default()
        }
    }
}

pub fn inscription(content_type: &str, body: impl AsRef<[u8]>) -> Inscription {
    Inscription::new(Some(content_type.into()), Some(body.as_ref().into()))
}

pub fn inscription_id(n: u32) -> InscriptionId {
    let hex = format!("{n:x}");

    if hex.is_empty() || hex.len() > 1 {
        panic!();
    }

    format!("{}i{n}", hex.repeat(64)).parse().unwrap()
}

pub fn envelope(payload: &[&[u8]]) -> Witness {
    let mut builder = script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF);

    for data in payload {
        let mut buf = PushBytesBuf::new();
        buf.extend_from_slice(data).unwrap();
        builder = builder.push_slice(buf);
    }

    let script = builder.push_opcode(opcodes::all::OP_ENDIF).into_script();

    Witness::from_slice(&[script.into_bytes(), Vec::new()])
}
