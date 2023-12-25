use l2o_common::{common::data::hash::Hash256, standards::l2o_a::supported_crypto::L2OAHashFunction};
use l2o_crypto::{standards::l2o_a::{L2ODeployInscriptionV1, L2OBlockInscriptionV1}, hash::merkle::core::MerkleProofCore};

pub trait L2OStoreV1 {
  fn has_deployed_l2id(&mut self, l2id: u64)->anyhow::Result<bool>;
  fn get_deploy_inscription(&mut self, l2id: u64)->anyhow::Result<L2ODeployInscriptionV1>;
  fn get_last_block_inscription(&mut self, l2id: u64)->anyhow::Result<L2OBlockInscriptionV1>;
  fn get_state_root_at_block(&mut self, l2id: u64, block_number: u64)->anyhow::Result<Hash256>;
  fn get_merkle_proof_state_root(&mut self, l2id: u64, block_number: u64, hash: L2OAHashFunction)->anyhow::Result<MerkleProofCore<Hash256>>;

  fn report_deploy_inscription(&mut self, deployment: L2ODeployInscriptionV1)->anyhow::Result<()>;
  fn set_last_block_inscription(&mut self, block: L2OBlockInscriptionV1)->anyhow::Result<()>;
}