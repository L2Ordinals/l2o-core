use l2o_common::common::data::hash::Hash256;
use plonky2::{hash::hash_types::HashOut, field::{goldilocks_field::GoldilocksField, types::{Field, PrimeField64}}};

use crate::{standards::l2o_a::L2OBlockInscriptionV1, fields::goldilocks::hash::{WHashOut, Wrapper}};


pub trait ZeroableHash: Sized + Copy + Clone{
  fn get_zero_value() -> Self;
}

pub trait L2OHash: ZeroableHash {
  fn to_hash_256(&self) -> Hash256;
  fn from_hash_256(hash: &Hash256) -> Self;
}
impl ZeroableHash for Hash256 {
  fn get_zero_value() -> Self {
      Hash256([0; 32])
  }
}
impl ZeroableHash for HashOut<GoldilocksField> {
    fn get_zero_value() -> Self {
        HashOut::ZERO
    }
}
impl ZeroableHash for WHashOut<GoldilocksField> {
    fn get_zero_value() -> Self {
        Wrapper(HashOut::ZERO)
    }
}

impl L2OHash for Hash256 {
  fn to_hash_256(&self) -> Hash256 {
      self.clone()
  }

  fn from_hash_256(hash: &Hash256) -> Self {
      hash.clone()
  }
}

impl L2OHash for WHashOut<GoldilocksField> {
    fn to_hash_256(&self) -> Hash256 {
        self.0.to_hash_256()
    }

    fn from_hash_256(hash: &Hash256) -> Self {
      Self(HashOut::<GoldilocksField>::from_hash_256(hash))
    }
}
impl L2OHash for HashOut<GoldilocksField> {
  fn to_hash_256(&self) -> Hash256 {
    let mut p = [0u8;32];
    let d = self.elements[3].to_canonical_u64().to_be_bytes();
    p[0..8].copy_from_slice(&d);
    let d = self.elements[2].to_canonical_u64().to_be_bytes();
    p[8..16].copy_from_slice(&d);
    let d = self.elements[1].to_canonical_u64().to_be_bytes();
    p[16..24].copy_from_slice(&d);
    let d = self.elements[0].to_canonical_u64().to_be_bytes();
    p[24..32].copy_from_slice(&d);
    Hash256(p)
  }

  fn from_hash_256(hash: &Hash256) -> Self {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&hash.0[0..8]);
    let a = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
    buf.copy_from_slice(&hash.0[8..16]);
    let b = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
    buf.copy_from_slice(&hash.0[16..24]);
    let c = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
    buf.copy_from_slice(&hash.0[24..32]);
    let d = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
    HashOut{
      elements: [d,c,b,a]
    }
  }
}

pub trait L2OBlockHasher {
  fn get_l2_block_hash(block: &L2OBlockInscriptionV1)->Hash256;
}

