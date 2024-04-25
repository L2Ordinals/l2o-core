use std::io;
use std::io::Cursor;

use bitcoin::block::Header;
use bitcoin::consensus::Decodable;
use bitcoin::consensus::Encodable;
use bitcoin::hashes::Hash;
use bitcoin::OutPoint;
use bitcoin::Txid;
use l2o_ord::inscription::inscription_id::InscriptionId;
use l2o_ord::sat::Sat;
use l2o_ord::sat_point::SatPoint;

pub type HeaderValue = [u8; 80];
pub type SatPointValue = [u8; 44];
pub type TxidValue = [u8; 32];
pub type OutPointValue = [u8; 36];
pub type InscriptionIdValue = (u128, u128, u32);

#[derive(Debug)]
pub struct InscriptionEntry {
    pub charms: u16,
    pub fee: u64,
    pub height: u32,
    pub id: InscriptionId,
    pub inscription_number: i32,
    pub parent: Option<u32>,
    pub sat: Option<Sat>,
    pub sequence_number: u32,
    pub timestamp: u32,
}

pub type InscriptionEntryValue = (
    u16,                // charms
    u64,                // fee
    u32,                // height
    InscriptionIdValue, // inscription id
    i32,                // inscription number
    Option<u32>,        // parent
    Option<u64>,        // sat
    u32,                // sequence number
    u32,                // timestamp
);

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

impl Entry for InscriptionId {
    type Value = InscriptionIdValue;

    fn load(value: Self::Value) -> Self {
        let (head, tail, index) = value;
        let head_array = head.to_le_bytes();
        let tail_array = tail.to_le_bytes();
        let array = [
            head_array[0],
            head_array[1],
            head_array[2],
            head_array[3],
            head_array[4],
            head_array[5],
            head_array[6],
            head_array[7],
            head_array[8],
            head_array[9],
            head_array[10],
            head_array[11],
            head_array[12],
            head_array[13],
            head_array[14],
            head_array[15],
            tail_array[0],
            tail_array[1],
            tail_array[2],
            tail_array[3],
            tail_array[4],
            tail_array[5],
            tail_array[6],
            tail_array[7],
            tail_array[8],
            tail_array[9],
            tail_array[10],
            tail_array[11],
            tail_array[12],
            tail_array[13],
            tail_array[14],
            tail_array[15],
        ];

        Self {
            txid: Txid::from_byte_array(array),
            index,
        }
    }

    fn store(self) -> Self::Value {
        let txid_entry = self.txid.store();
        let little_end = u128::from_le_bytes(txid_entry[..16].try_into().unwrap());
        let big_end = u128::from_le_bytes(txid_entry[16..].try_into().unwrap());
        (little_end, big_end, self.index)
    }
}

impl Entry for InscriptionEntry {
    type Value = InscriptionEntryValue;

  #[rustfmt::skip]
    fn load(
    (
      charms,
      fee,
      height,
      id,
      inscription_number,
      parent,
      sat,
      sequence_number,
      timestamp,
    ): InscriptionEntryValue,
  ) -> Self {
    Self {
      charms,
      fee,
      height,
      id: InscriptionId::load(id),
      inscription_number,
      parent,
      sat: sat.map(Sat),
      sequence_number,
      timestamp,
    }
  }

    fn store(self) -> Self::Value {
        (
            self.charms,
            self.fee,
            self.height,
            self.id.store(),
            self.inscription_number,
            self.parent,
            self.sat.map(Sat::n),
            self.sequence_number,
            self.timestamp,
        )
    }
}
