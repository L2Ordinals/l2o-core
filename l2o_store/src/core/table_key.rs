use kvq::traits::KVQSerializable;

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
    fn to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::with_capacity(10);
        result.push(((TABLE_TYPE & 0xFF00) >> 8) as u8); // 1
        result.push((TABLE_TYPE & 0xFF) as u8); // 2
        result.extend_from_slice(&self.l2id.to_be_bytes()); // 11
        result
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            l2id: u64::from_be_bytes(bytes[2..10].try_into().unwrap()),
        }
    }
}
