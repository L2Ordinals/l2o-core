use plonky2::{field::{goldilocks_field::GoldilocksField, types::Field}, hash::hash_types::HashOut};

use super::traits::KVQSerializable;

impl<const SIZE: usize> KVQSerializable for [u8; SIZE] {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut result = [0u8; SIZE];
        result.copy_from_slice(bytes);
        result
    }
}

impl KVQSerializable for Vec<u8> {
    fn to_bytes(&self) -> Vec<u8> {
        self.clone()
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        bytes.to_vec()
    }
}
impl<const SIZE: usize> KVQSerializable for [u64; SIZE] {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(SIZE * 8);
        for i in 0..SIZE {
            result.extend_from_slice(&self[i].to_be_bytes());
        }
        result
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut result = [0u64; SIZE];
        for i in 0..SIZE {
            let mut bytes_u64 = [0u8; 8];
            bytes_u64.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
            result[i] = u64::from_be_bytes(bytes_u64);
        }
        result
    }
}


impl<const SIZE: usize> KVQSerializable for [u32; SIZE] {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(SIZE * 4);
        for i in 0..SIZE {
            result.extend_from_slice(&self[i].to_be_bytes());
        }
        result
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut result = [0u32; SIZE];
        for i in 0..SIZE {
            let mut bytes_u32 = [0u8; 4];
            bytes_u32.copy_from_slice(&bytes[i * 4..(i + 1) * 4]);
            result[i] = u32::from_be_bytes(bytes_u32);
        }
        result
    }
}


impl KVQSerializable for HashOut<GoldilocksField> {
    fn to_bytes(&self) -> Vec<u8> {

        let mut result = Vec::with_capacity(32);
        result.extend_from_slice(&self.elements[3].0.to_be_bytes());
        result.extend_from_slice(&self.elements[2].0.to_be_bytes());
        result.extend_from_slice(&self.elements[1].0.to_be_bytes());
        result.extend_from_slice(&self.elements[0].0.to_be_bytes());
        result
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[0..8]);
        let a = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
        buf.copy_from_slice(&bytes[8..16]);
        let b = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
        buf.copy_from_slice(&bytes[16..24]);
        let c = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
        buf.copy_from_slice(&bytes[24..32]);
        let d = GoldilocksField::from_noncanonical_u64(u64::from_be_bytes(buf));
        HashOut{
          elements: [d,c,b,a]
        }
    }
}