use l2o_common::common::data::hash::Hash256;
use l2o_crypto::hash::merkle::core::MerkleProofCore;
use l2o_ord::operation::l2o_a::L2OABlockV1;
use l2o_ord::operation::l2o_a::L2OADeployV1;
use l2o_ord::operation::l2o_a::L2OAHashFunction;

pub trait L2OStoreReaderV1 {
    fn has_deployed_l2id(&self, l2id: u64) -> anyhow::Result<bool>;
    fn get_brc21_balance(&self, tick: String, address: String) -> anyhow::Result<u64>;
    fn get_deploy_inscription(&self, l2id: u64) -> anyhow::Result<L2OADeployV1>;
    fn get_last_block_inscription(&self, l2id: u64) -> anyhow::Result<L2OABlockV1>;
    fn get_state_root_at_block(
        &self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256>;
    fn get_superchainroot_at_block(
        &self,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256>;
    fn get_merkle_proof_state_root_at_block(
        &self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<MerkleProofCore<Hash256>>;
}

pub trait L2OStoreV1: L2OStoreReaderV1 {
    fn transfer_brc21(
        &mut self,
        tick: String,
        from: String,
        to: String,
        amount: u64,
    ) -> anyhow::Result<()>;
    fn report_deploy_inscription(&mut self, deployment: L2OADeployV1) -> anyhow::Result<()>;
    fn set_last_block_inscription(&mut self, block: L2OABlockV1) -> anyhow::Result<()>;
}
