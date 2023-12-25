use l2o_crypto::hash::merkle::store::key::KVQMerkleNodeKey;

use super::table_key::L2TableKey;

pub const TABLE_L2_DEPLOYMENTS: u16 = 1;
pub const TABLE_L2_LATEST_BLOCK: u16 = 2;

pub const TABLE_L2_STATE_ROOTS: u16 = 8;
pub const SUB_TABLE_L2_STATE_ROOTS_SHA256: u8 = 1;
pub const SUB_TABLE_L2_STATE_ROOTS_KECCACK256: u8 = 2;
pub const SUB_TABLE_L2_STATE_ROOTS_BLAKE3: u8 = 3;
pub const SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS: u8 = 4;





pub type L2OStateRootsMerkleNodeKey = KVQMerkleNodeKey<TABLE_L2_STATE_ROOTS>;
pub type L2ODeploymentsKey = L2TableKey<TABLE_L2_DEPLOYMENTS>;
pub type L2OLatestBlockKey = L2TableKey<TABLE_L2_DEPLOYMENTS>;


