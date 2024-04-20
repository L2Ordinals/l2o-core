use kvq::traits::KVQSerializable;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct L2TableKey<const TABLE_TYPE: u16> {
    pub l2id: u64,
}
impl<const TABLE_TYPE: u16> L2TableKey<TABLE_TYPE> {
    pub fn new(l2id: u64) -> Self {
        Self { l2id }
    }
}
impl<const TABLE_TYPE: u16> KVQSerializable for L2TableKey<TABLE_TYPE> {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::with_capacity(10);
        result.push(((TABLE_TYPE & 0xFF00) >> 8) as u8); // 1
        result.push((TABLE_TYPE & 0xFF) as u8); // 2
        result.extend_from_slice(&self.l2id.to_be_bytes()); // 11
        Ok(result)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {
            l2id: u64::from_be_bytes(bytes[2..10].try_into()?),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BRC21TableKey<const TABLE_TYPE: u16> {
    pub tick: String,
    pub address: String,
}
impl<const TABLE_TYPE: u16> BRC21TableKey<TABLE_TYPE> {
    pub const fn new(tick: String, address: String) -> Self {
        Self { tick, address }
    }
}
impl<const TABLE_TYPE: u16> KVQSerializable for BRC21TableKey<TABLE_TYPE> {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::with_capacity(10);
        result.push(((TABLE_TYPE & 0xFF00) >> 8) as u8); // 1
        result.push((TABLE_TYPE & 0xFF) as u8); // 2
        result.extend_from_slice(&serde_json::to_vec(&self)?);
        Ok(result)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(&bytes[2..])?)
    }
}
