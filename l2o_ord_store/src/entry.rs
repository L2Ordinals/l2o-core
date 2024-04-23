use std::io;

use bitcoin::consensus::Decodable;
use bitcoin::consensus::Encodable;
use bitcoin::hashes::Hash;
use bitcoin::Txid;
use l2o_ord::sat_point::SatPoint;

pub type SatPointValue = [u8; 44];
pub type TxidValue = [u8; 32];

pub trait Entry: Sized {
    type Value;

    fn load(value: Self::Value) -> Self;

    fn store(self) -> Self::Value;
}

impl Entry for SatPoint {
    type Value = SatPointValue;

    fn load(value: Self::Value) -> Self {
        Decodable::consensus_decode(&mut io::Cursor::new(value)).unwrap()
    }

    fn store(self) -> Self::Value {
        let mut value = [0; 44];
        self.consensus_encode(&mut value.as_mut_slice()).unwrap();
        value
    }
}

impl Entry for Txid {
    type Value = TxidValue;

    fn load(value: Self::Value) -> Self {
        Txid::from_byte_array(value)
    }

    fn store(self) -> Self::Value {
        Txid::to_byte_array(self)
    }
}
