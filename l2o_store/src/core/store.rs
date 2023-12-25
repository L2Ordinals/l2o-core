use kvq::{traits::{KVQBinaryStore, KVQStoreAdapter}, adapters::standard::KVQStandardAdapter};
use l2o_common::{common::data::hash::Hash256, standards::l2o_a::supported_crypto::L2OAHashFunction};
use l2o_crypto::{standards::l2o_a::{L2ODeployInscriptionV1, L2OBlockInscriptionV1}, hash::{merkle::{store::{model::KVQMerkleTreeModel, key::KVQMerkleNodeKey}, core::MerkleProofCore}, hash_functions::{sha256::Sha256Hasher, keccack256::Keccack256Hasher, poseidon_goldilocks::PoseidonHasher, blake3::Blake3Hasher}}, fields::goldilocks::hash::{GHashOut, hash256_to_goldilocks_hash}};

use super::{traits::L2OStoreV1, tables::{L2ODeploymentsKey, L2OLatestBlockKey, TABLE_L2_STATE_ROOTS, L2OStateRootsMerkleNodeKey, SUB_TABLE_L2_STATE_ROOTS_SHA256, SUB_TABLE_L2_STATE_ROOTS_KECCACK256, SUB_TABLE_L2_STATE_ROOTS_BLAKE3, SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS}};

type Sha256StateRootTree<S> = KVQMerkleTreeModel::<TABLE_L2_STATE_ROOTS, false, S, KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>, Hash256, Sha256Hasher>;
type Keccack256StateRootTree<S> = KVQMerkleTreeModel::<TABLE_L2_STATE_ROOTS, false, S, KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>, Hash256, Keccack256Hasher>;
type Blake3StateRootTree<S> = KVQMerkleTreeModel::<TABLE_L2_STATE_ROOTS, false, S, KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>, Hash256, Blake3Hasher>;
type PoseidonGoldilocksStateRootTree<S> = KVQMerkleTreeModel::<TABLE_L2_STATE_ROOTS, false, S, KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, GHashOut>, GHashOut, PoseidonHasher>;
const TREE_HEIGHT: usize = 32;

pub struct L2OStoreV1Core<S: KVQBinaryStore> {
  pub store: S,
}
impl <S: KVQBinaryStore> L2OStoreV1Core<S> {
  pub fn new(store: S) -> Self {
    Self {
      store,
    }
  }
}
impl<S: KVQBinaryStore> L2OStoreV1 for L2OStoreV1Core<S> {
    fn get_deploy_inscription(&mut self, l2id: u64)->anyhow::Result<L2ODeployInscriptionV1> {
        KVQStandardAdapter::<S, L2ODeploymentsKey, L2ODeployInscriptionV1>::get_exact(&self.store, &L2ODeploymentsKey::new(l2id))
     }

    fn get_last_block_inscription(&mut self, l2id: u64)->anyhow::Result<L2OBlockInscriptionV1> {
      KVQStandardAdapter::<S, L2OLatestBlockKey, L2OBlockInscriptionV1>::get_exact(&self.store, &L2OLatestBlockKey::new(l2id))
    }

    fn get_state_root_at_block(&mut self, l2id: u64, block_number: u64)->anyhow::Result<Hash256> {
        Sha256StateRootTree::<S>::get_node(&self.store, TREE_HEIGHT, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_SHA256, 0, 0, TREE_HEIGHT as u8, l2id, block_number))
    }

    fn get_merkle_proof_state_root(&mut self, l2id: u64, block_number: u64, hash: L2OAHashFunction)->anyhow::Result<MerkleProofCore<Hash256>> {
        match hash {
            L2OAHashFunction::Sha256 => {
              Sha256StateRootTree::<S>::get_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_SHA256, 0, 0, TREE_HEIGHT as u8, l2id, block_number))
            },
            L2OAHashFunction::BLAKE3 => {
              Blake3StateRootTree::<S>::get_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_BLAKE3, 0, 0, TREE_HEIGHT as u8, l2id, block_number))
            },
            L2OAHashFunction::Keccack256 => {
              Keccack256StateRootTree::<S>::get_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_KECCACK256, 0, 0, TREE_HEIGHT as u8, l2id, block_number))
            },
            L2OAHashFunction::PoseidonGoldilocks => {
              let p = PoseidonGoldilocksStateRootTree::<S>::get_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS, 0, 0, TREE_HEIGHT as u8, l2id, block_number))?;
              Ok(p.into())
            },
        }
    }

    fn report_deploy_inscription(&mut self, deployment: L2ODeployInscriptionV1)->anyhow::Result<()> {
        let has_deployed = self.has_deployed_l2id(deployment.l2id)?;
        if has_deployed {
          return Err(anyhow::anyhow!("L2ID {} already deployed", deployment.l2id));
        }

        KVQStandardAdapter::<S, L2ODeploymentsKey, L2ODeployInscriptionV1>::set(&mut self.store, L2ODeploymentsKey::new(deployment.l2id), deployment)?;
        Ok(())
    }

    fn set_last_block_inscription(&mut self, block: L2OBlockInscriptionV1)->anyhow::Result<()> {
      let end_state_root = block.end_state_root;
      let glv = hash256_to_goldilocks_hash(&end_state_root);
      let block_num = block.bitcoin_block_number;
      let l2id = block.l2id;

      KVQStandardAdapter::<S, L2OLatestBlockKey, L2OBlockInscriptionV1>::set(&mut self.store, L2OLatestBlockKey::new(block.l2id), block)?;
      Sha256StateRootTree::<S>::set_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_SHA256, 0, 0, TREE_HEIGHT as u8, l2id, block_num), end_state_root)?;
      Blake3StateRootTree::<S>::set_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_BLAKE3, 0, 0, TREE_HEIGHT as u8, l2id, block_num), end_state_root)?;
      Keccack256StateRootTree::<S>::set_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_KECCACK256, 0, 0, TREE_HEIGHT as u8, l2id, block_num), end_state_root)?;
      PoseidonGoldilocksStateRootTree::<S>::set_leaf(&mut self.store, &KVQMerkleNodeKey::new(SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS, 0, 0, TREE_HEIGHT as u8, l2id, block_num), glv)?;
      
      Ok(())
    }

    fn has_deployed_l2id(&mut self, l2id: u64)->anyhow::Result<bool> {
      let r = KVQStandardAdapter::<S, L2ODeploymentsKey, L2ODeployInscriptionV1>::get_exact(&mut self.store, &L2ODeploymentsKey::new(l2id));
      match r {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
      }
    }
}