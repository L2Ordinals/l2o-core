use kvq::adapters::standard::KVQStandardAdapter;
use kvq::traits::KVQBinaryStore;
use kvq::traits::KVQStoreAdapter;
use l2o_common::common::data::hash::Hash256;
use l2o_common::standards::l2o_a::supported_crypto::L2OAHashFunction;
use l2o_crypto::fields::goldilocks::hash::hash256_to_goldilocks_hash;
use l2o_crypto::fields::goldilocks::hash::GHashOut;
use l2o_crypto::hash::hash_functions::blake3::Blake3Hasher;
use l2o_crypto::hash::hash_functions::keccak256::Keccak256Hasher;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::merkle::core::MerkleProofCore;
use l2o_crypto::hash::merkle::store::key::KVQMerkleNodeKey;
use l2o_crypto::hash::merkle::store::model::KVQMerkleTreeModel;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_crypto::standards::l2o_a::L2ODeployInscriptionV1;

use super::tables::L2ODeploymentsKey;
use super::tables::L2OLatestBlockKey;
use super::tables::L2OStateRootsMerkleNodeKey;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_BLAKE3;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_KECCACK256;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_SHA256;
use super::tables::TABLE_L2_STATE_ROOTS;
use super::traits::L2OStoreV1;

type Sha256StateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    false,
    S,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>,
    Hash256,
    Sha256Hasher,
>;
type Keccack256StateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    false,
    S,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>,
    Hash256,
    Keccak256Hasher,
>;
type Blake3StateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    false,
    S,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>,
    Hash256,
    Blake3Hasher,
>;
type PoseidonGoldilocksStateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    false,
    S,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, GHashOut>,
    GHashOut,
    PoseidonHasher,
>;
const TREE_HEIGHT: usize = 32;

pub struct L2OStoreV1Core<S: KVQBinaryStore> {
    pub store: S,
}
impl<S: KVQBinaryStore> L2OStoreV1Core<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
impl<S: KVQBinaryStore> L2OStoreV1 for L2OStoreV1Core<S> {
    fn get_deploy_inscription(&mut self, l2id: u64) -> anyhow::Result<L2ODeployInscriptionV1> {
        KVQStandardAdapter::<S, L2ODeploymentsKey, L2ODeployInscriptionV1>::get_exact(
            &self.store,
            &L2ODeploymentsKey::new(l2id),
        )
    }

    fn get_last_block_inscription(&mut self, l2id: u64) -> anyhow::Result<L2OBlockInscriptionV1> {
        KVQStandardAdapter::<S, L2OLatestBlockKey, L2OBlockInscriptionV1>::get_exact(
            &self.store,
            &L2OLatestBlockKey::new(l2id),
        )
    }

    fn get_state_root_at_block(&mut self, l2id: u64, block_number: u64) -> anyhow::Result<Hash256> {
        Sha256StateRootTree::<S>::get_node(
            &self.store,
            TREE_HEIGHT,
            &KVQMerkleNodeKey::new(
                SUB_TABLE_L2_STATE_ROOTS_SHA256,
                0,
                0,
                TREE_HEIGHT as u8,
                l2id,
                block_number,
            ),
        )
    }

    fn get_merkle_proof_state_root(
        &mut self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<MerkleProofCore<Hash256>> {
        match hash {
            L2OAHashFunction::Sha256 => Sha256StateRootTree::<S>::get_leaf(
                &mut self.store,
                &KVQMerkleNodeKey::new(
                    SUB_TABLE_L2_STATE_ROOTS_SHA256,
                    0,
                    0,
                    TREE_HEIGHT as u8,
                    l2id,
                    block_number,
                ),
            ),
            L2OAHashFunction::BLAKE3 => Blake3StateRootTree::<S>::get_leaf(
                &mut self.store,
                &KVQMerkleNodeKey::new(
                    SUB_TABLE_L2_STATE_ROOTS_BLAKE3,
                    0,
                    0,
                    TREE_HEIGHT as u8,
                    l2id,
                    block_number,
                ),
            ),
            L2OAHashFunction::Keccak256 => Keccack256StateRootTree::<S>::get_leaf(
                &mut self.store,
                &KVQMerkleNodeKey::new(
                    SUB_TABLE_L2_STATE_ROOTS_KECCACK256,
                    0,
                    0,
                    TREE_HEIGHT as u8,
                    l2id,
                    block_number,
                ),
            ),
            L2OAHashFunction::PoseidonGoldilocks => {
                let p = PoseidonGoldilocksStateRootTree::<S>::get_leaf(
                    &mut self.store,
                    &KVQMerkleNodeKey::new(
                        SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS,
                        0,
                        0,
                        TREE_HEIGHT as u8,
                        l2id,
                        block_number,
                    ),
                )?;
                Ok(p.into())
            }
        }
    }

    fn report_deploy_inscription(
        &mut self,
        deployment: L2ODeployInscriptionV1,
    ) -> anyhow::Result<()> {
        KVQStandardAdapter::<S, L2ODeploymentsKey, L2ODeployInscriptionV1>::set(
            &mut self.store,
            L2ODeploymentsKey::new(deployment.l2id),
            deployment,
        )?;
        Ok(())
    }

    fn set_last_block_inscription(&mut self, block: L2OBlockInscriptionV1) -> anyhow::Result<()> {
        let end_state_root = block.end_state_root;
        let glv = hash256_to_goldilocks_hash(&end_state_root);
        let block_num = block.bitcoin_block_number;
        let l2id = block.l2id;

        KVQStandardAdapter::<S, L2OLatestBlockKey, L2OBlockInscriptionV1>::set(
            &mut self.store,
            L2OLatestBlockKey::new(block.l2id),
            block,
        )?;
        Sha256StateRootTree::<S>::set_leaf(
            &mut self.store,
            &KVQMerkleNodeKey::new(
                SUB_TABLE_L2_STATE_ROOTS_SHA256,
                0,
                0,
                TREE_HEIGHT as u8,
                l2id,
                block_num,
            ),
            end_state_root,
        )?;
        Blake3StateRootTree::<S>::set_leaf(
            &mut self.store,
            &KVQMerkleNodeKey::new(
                SUB_TABLE_L2_STATE_ROOTS_BLAKE3,
                0,
                0,
                TREE_HEIGHT as u8,
                l2id,
                block_num,
            ),
            end_state_root,
        )?;
        Keccack256StateRootTree::<S>::set_leaf(
            &mut self.store,
            &KVQMerkleNodeKey::new(
                SUB_TABLE_L2_STATE_ROOTS_KECCACK256,
                0,
                0,
                TREE_HEIGHT as u8,
                l2id,
                block_num,
            ),
            end_state_root,
        )?;
        PoseidonGoldilocksStateRootTree::<S>::set_leaf(
            &mut self.store,
            &KVQMerkleNodeKey::new(
                SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS,
                0,
                0,
                TREE_HEIGHT as u8,
                l2id,
                block_num,
            ),
            glv,
        )?;

        Ok(())
    }

    fn has_deployed_l2id(&mut self, l2id: u64) -> anyhow::Result<bool> {
        let r = KVQStandardAdapter::<S, L2ODeploymentsKey, L2ODeployInscriptionV1>::get_exact(
            &mut self.store,
            &L2ODeploymentsKey::new(l2id),
        );
        match r {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
