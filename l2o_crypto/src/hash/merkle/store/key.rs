use kvq::traits::KVQSerializable;
use plonky2::field::types::PrimeField64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KVQTreeNodePosition {
    pub level: u8,
    pub index: u64,
}

impl KVQTreeNodePosition {
    pub const fn new(level: u8, index: u64) -> Self {
        Self { level, index }
    }
    pub const fn root() -> Self {
        Self { level: 0, index: 0 }
    }
    pub fn new_u8_f<F: PrimeField64>(level: u8, index: F) -> Self {
        Self {
            level: level,
            index: index.to_canonical_u64(),
        }
    }
    pub fn new_ff<F: PrimeField64>(level: F, index: F) -> Self {
        Self {
            level: level.to_canonical_u64() as u8,
            index: index.to_canonical_u64(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KVQTreeIdentifier {
    pub tree_id: u8,
    pub primary_id: u64,
    pub secondary_id: u32,
}
impl KVQTreeIdentifier {
    pub const fn new(tree_id: u8, primary_id: u64, secondary_id: u32) -> Self {
        Self {
            tree_id,
            primary_id,
            secondary_id,
        }
    }
    pub fn new_ff<F: PrimeField64>(tree_id: u8, primary_id: F, secondary_id: F) -> Self {
        Self {
            tree_id,
            primary_id: primary_id.to_canonical_u64(),
            secondary_id: secondary_id.to_canonical_u64() as u32,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KVQMerkleNodeKey<const TABLE_TYPE: u16> {
    pub tree_id: u8,
    pub primary_id: u64,
    pub secondary_id: u32,
    pub level: u8,
    pub index: u64,
    pub checkpoint_id: u64,
}
impl<const TABLE_TYPE: u16> KVQMerkleNodeKey<TABLE_TYPE> {
    pub fn sibling(&self) -> Self {
        Self {
            tree_id: self.tree_id,
            primary_id: self.primary_id,
            secondary_id: self.secondary_id,
            level: self.level,
            index: self.index ^ 1,
            checkpoint_id: self.checkpoint_id,
        }
    }
    pub fn siblings(&self) -> Vec<KVQMerkleNodeKey<TABLE_TYPE>> {
        let mut result: Vec<KVQMerkleNodeKey<TABLE_TYPE>> = Vec::with_capacity(self.level as usize);
        let mut current = *self;
        for _ in 0..self.level {
            result.push(current.sibling());
            current = current.parent();
        }
        result
    }
    pub fn parent(&self) -> Self {
        if self.level == 0 {
            return *self;
        }
        Self {
            tree_id: self.tree_id,
            primary_id: self.primary_id,
            secondary_id: self.secondary_id,
            level: self.level - 1,
            index: self.index >> 1,
            checkpoint_id: self.checkpoint_id,
        }
    }
    pub fn root(&self) -> Self {
        if self.level == 0 {
            return *self;
        }
        Self {
            tree_id: self.tree_id,
            primary_id: self.primary_id,
            secondary_id: self.secondary_id,
            level: 0,
            index: 0,
            checkpoint_id: self.checkpoint_id,
        }
    }
}
impl<const TABLE_TYPE: u16> KVQSerializable for KVQMerkleNodeKey<TABLE_TYPE> {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::with_capacity(32);
        result.push(((TABLE_TYPE & 0xFF00) >> 8) as u8); // 1
        result.push((TABLE_TYPE & 0xFF) as u8); // 2
        result.push(self.tree_id); // 3
        result.extend_from_slice(&self.primary_id.to_be_bytes()); // 11
        result.extend_from_slice(&self.secondary_id.to_be_bytes()); // 15
        result.push(self.level); // 16
        result.extend_from_slice(&self.index.to_be_bytes()); // 24
        result.extend_from_slice(&self.checkpoint_id.to_be_bytes()); // 32
        Ok(result)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {
            tree_id: bytes[2],
            primary_id: u64::from_be_bytes(bytes[3..11].try_into()?),
            secondary_id: u32::from_be_bytes(bytes[11..15].try_into()?),
            level: bytes[15],
            index: u64::from_be_bytes(bytes[16..24].try_into()?),
            checkpoint_id: u64::from_be_bytes(bytes[24..32].try_into()?),
        })
    }
}
impl<const TABLE_TYPE: u16> KVQMerkleNodeKey<TABLE_TYPE> {
    #[inline]
    pub const fn new(
        tree_id: u8,
        primary_id: u64,
        secondary_id: u32,
        level: u8,
        index: u64,
        checkpoint_id: u64,
    ) -> Self {
        Self {
            tree_id,
            primary_id,
            secondary_id,
            level,
            index,
            checkpoint_id,
        }
    }
    #[inline]
    pub const fn from_position(
        tree_id: u8,
        primary_id: u64,
        secondary_id: u32,
        checkpoint_id: u64,
        position: KVQTreeNodePosition,
    ) -> Self {
        Self::from_position_ref(tree_id, primary_id, secondary_id, checkpoint_id, &position)
    }
    #[inline]
    pub const fn from_position_ref(
        tree_id: u8,
        primary_id: u64,
        secondary_id: u32,
        checkpoint_id: u64,
        position: &KVQTreeNodePosition,
    ) -> Self {
        Self {
            tree_id,
            primary_id,
            secondary_id,
            level: position.level,
            index: position.index,
            checkpoint_id,
        }
    }
    #[inline]
    pub const fn from_identifier_position_ref(
        identifier: &KVQTreeIdentifier,
        checkpoint_id: u64,
        position: &KVQTreeNodePosition,
    ) -> Self {
        Self {
            tree_id: identifier.tree_id,
            primary_id: identifier.primary_id,
            secondary_id: identifier.secondary_id,
            level: position.level,
            index: position.index,
            checkpoint_id,
        }
    }
    #[inline]
    pub const fn from_identifier_position(
        identifier: &KVQTreeIdentifier,
        checkpoint_id: u64,
        position: KVQTreeNodePosition,
    ) -> Self {
        Self {
            tree_id: identifier.tree_id,
            primary_id: identifier.primary_id,
            secondary_id: identifier.secondary_id,
            level: position.level,
            index: position.index,
            checkpoint_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KVQAppendOnlyMerkleKey<const TABLE_TYPE: u16> {
    pub tree_id: u8,
    pub primary_id: u64,
    pub secondary_id: u32,
    pub checkpoint_id: u64,
    pub tick: String,
}
impl<const TABLE_TYPE: u16> KVQSerializable for KVQAppendOnlyMerkleKey<TABLE_TYPE> {
    fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::with_capacity(32);
        result.push(((TABLE_TYPE & 0xFF00) >> 8) as u8); // 1
        result.push((TABLE_TYPE & 0xFF) as u8); // 2
        result.push(self.tree_id); // 3
        result.extend_from_slice(&self.primary_id.to_be_bytes()); // 11
        result.extend_from_slice(&self.secondary_id.to_be_bytes()); // 15
        result.extend_from_slice(&self.checkpoint_id.to_be_bytes()); // 23
        result.extend_from_slice(self.tick.as_bytes());
        Ok(result)
    }

    fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(Self {
            tree_id: bytes[2],
            primary_id: u64::from_be_bytes(bytes[3..11].try_into()?),
            secondary_id: u32::from_be_bytes(bytes[11..15].try_into()?),
            checkpoint_id: u64::from_be_bytes(bytes[15..23].try_into()?),
            tick: String::from_utf8(bytes[23..].to_vec())?,
        })
    }
}

impl<const TABLE_TYPE: u16> KVQAppendOnlyMerkleKey<TABLE_TYPE> {
    #[inline]
    pub const fn new(
        tree_id: u8,
        primary_id: u64,
        secondary_id: u32,
        checkpoint_id: u64,
        tick: String,
    ) -> Self {
        Self {
            tree_id,
            primary_id,
            secondary_id,
            checkpoint_id,
            tick,
        }
    }
    #[inline]
    pub const fn from_identifier_ref(
        identifier: &KVQTreeIdentifier,
        checkpoint_id: u64,
        tick: String,
    ) -> Self {
        Self {
            tree_id: identifier.tree_id,
            primary_id: identifier.primary_id,
            secondary_id: identifier.secondary_id,
            checkpoint_id,
            tick,
        }
    }
}
