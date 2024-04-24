use std::io;
use std::io::Cursor;

use bitcoin::block::Header;
use bitcoin::consensus::Decodable;
use bitcoin::consensus::Encodable;
use bitcoin::hashes::Hash;
use bitcoin::OutPoint;
use bitcoin::Txid;
use l2o_ord::sat_point::SatPoint;

pub type HeaderValue = [u8; 80];
pub type SatPointValue = [u8; 44];
pub type TxidValue = [u8; 32];
pub type OutPointValue = [u8; 36];

pub trait Entry: Sized {
    type Value;

    fn load(value: Self::Value) -> Self;

    fn store(self) -> Self::Value;
}

impl Entry for Header {
    type Value = HeaderValue;

    fn load(value: Self::Value) -> Self {
        bitcoin::consensus::encode::deserialize(&value).unwrap()
    }

    fn store(self) -> Self::Value {
        let mut buffer = Cursor::new([0; 80]);
        let len = self
            .consensus_encode(&mut buffer)
            .expect("in-memory writers don't error");
        let buffer = buffer.into_inner();
        debug_assert_eq!(len, buffer.len());
        buffer
    }
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

impl Entry for OutPoint {
    type Value = OutPointValue;

    fn load(value: Self::Value) -> Self {
        Decodable::consensus_decode(&mut io::Cursor::new(value)).unwrap()
    }

    fn store(self) -> Self::Value {
        let mut value = [0; 36];
        self.consensus_encode(&mut value.as_mut_slice()).unwrap();
        value
    }
}
