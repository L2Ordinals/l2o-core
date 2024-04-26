use kvq::adapters::standard::KVQStandardAdapter;
use kvq::traits::KVQBinaryStore;
use kvq::traits::KVQPair;
use kvq::traits::KVQStoreAdapter;
use l2o_common::common::data::hash::Hash256;
use l2o_crypto::fields::goldilocks::hash::hash256_to_goldilocks_hash;
use l2o_crypto::fields::goldilocks::hash::GHashOut;
use l2o_crypto::hash::hash_functions::blake3::Blake3Hasher;
use l2o_crypto::hash::hash_functions::keccak256::Keccak256Hasher;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::merkle::core::MerkleProofCore;
use l2o_crypto::hash::merkle::store::key::KVQMerkleNodeKey;
use l2o_crypto::hash::merkle::store::key::KVQTreeIdentifier;
use l2o_crypto::hash::merkle::store::key::KVQTreeNodePosition;
use l2o_crypto::hash::merkle::store::model::KVQMerkleTreeModel;
use l2o_crypto::hash::traits::L2OHash;
use l2o_macros::get_state;
use l2o_macros::set_state;
use l2o_ord::operation::l2o_a::L2OABlockInscriptionV1;
use l2o_ord::operation::l2o_a::L2OADeployInscriptionV1;
use l2o_ord::operation::l2o_a::L2OAHashFunction;

use super::tables::L2ODeploymentsKey;
use super::tables::L2OLatestBlockKey;
use super::tables::L2OStateRootsMerkleNodeKey;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_BLAKE3;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_KECCACK256;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS;
use super::tables::SUB_TABLE_L2_STATE_ROOTS_SHA256;
use super::tables::TABLE_L2_STATE_ROOTS;
use super::traits::L2OStoreV1;
use crate::core::tables::L2OBRC21BalancesKey;

const TREE_HEIGHT: u8 = 32;

pub const BRC20_BURN_ADDRESS: &'static str =
    "bc1p11111111111111111111111111111111111111111111111111114olvt2";

pub const SHA256_STATE_ROOT_TREE_ID: KVQTreeIdentifier =
    KVQTreeIdentifier::new(SUB_TABLE_L2_STATE_ROOTS_SHA256, 0, 0);
type Sha256StateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    TREE_HEIGHT,
    false,
    S,
    Hash256,
    Sha256Hasher,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>,
>;
pub const KECCAK256_STATE_ROOT_TREE_ID: KVQTreeIdentifier =
    KVQTreeIdentifier::new(SUB_TABLE_L2_STATE_ROOTS_KECCACK256, 0, 0);
type Keccak256StateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    TREE_HEIGHT,
    false,
    S,
    Hash256,
    Keccak256Hasher,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>,
>;
pub const BLAKE3_STATE_ROOT_TREE_ID: KVQTreeIdentifier =
    KVQTreeIdentifier::new(SUB_TABLE_L2_STATE_ROOTS_BLAKE3, 0, 0);
type Blake3StateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    TREE_HEIGHT,
    false,
    S,
    Hash256,
    Blake3Hasher,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, Hash256>,
>;
pub const POSEIDONGOLDILOCKS_STATE_ROOT_TREE_ID: KVQTreeIdentifier =
    KVQTreeIdentifier::new(SUB_TABLE_L2_STATE_ROOTS_POSEIDON_GOLDILOCKS, 0, 0);
type PoseidonGoldilocksStateRootTree<S> = KVQMerkleTreeModel<
    TABLE_L2_STATE_ROOTS,
    TREE_HEIGHT,
    false,
    S,
    GHashOut,
    PoseidonHasher,
    KVQStandardAdapter<S, L2OStateRootsMerkleNodeKey, GHashOut>,
>;

pub struct L2OStoreV1Core<S: KVQBinaryStore> {
    pub store: S,
}
impl<S: KVQBinaryStore> L2OStoreV1Core<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
impl<S: KVQBinaryStore> L2OStoreV1 for L2OStoreV1Core<S> {
    fn get_deploy_inscription(&mut self, l2id: u64) -> anyhow::Result<L2OADeployInscriptionV1> {
        KVQStandardAdapter::<S, L2ODeploymentsKey, L2OADeployInscriptionV1>::get_exact(
            &self.store,
            &L2ODeploymentsKey::new(l2id),
        )
    }

    fn get_last_block_inscription(&mut self, l2id: u64) -> anyhow::Result<L2OABlockInscriptionV1> {
        KVQStandardAdapter::<S, L2OLatestBlockKey, L2OABlockInscriptionV1>::get_exact(
            &self.store,
            &L2OLatestBlockKey::new(l2id),
        )
    }

    fn get_state_root_at_block(
        &mut self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256> {
        let checkpoint_id = block_number;
        let pos = KVQTreeNodePosition::new(TREE_HEIGHT, l2id);
        get_state!(
            hash,
            self.store,
            checkpoint_id,
            pos,
            get_node,
            L2OHash::to_hash_256
        )
    }

    fn get_superchainroot_at_block(
        &mut self,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256> {
        let checkpoint_id = block_number;
        let pos = KVQTreeNodePosition::root();
        get_state!(
            hash,
            self.store,
            checkpoint_id,
            pos,
            get_node,
            L2OHash::to_hash_256
        )
    }

    fn get_merkle_proof_state_root_at_block(
        &mut self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<MerkleProofCore<Hash256>> {
        let checkpoint_id = block_number;
        let pos = KVQTreeNodePosition::new(TREE_HEIGHT, l2id);
        get_state!(
            hash,
            self.store,
            checkpoint_id,
            pos,
            get_leaf,
            MerkleProofCore::<Hash256>::from
        )
    }

    fn report_deploy_inscription(
        &mut self,
        deployment: L2OADeployInscriptionV1,
    ) -> anyhow::Result<()> {
        KVQStandardAdapter::<S, L2ODeploymentsKey, L2OADeployInscriptionV1>::set(
            &mut self.store,
            L2ODeploymentsKey::new(deployment.l2id),
            deployment,
        )?;
        Ok(())
    }

    fn set_last_block_inscription(&mut self, block: L2OABlockInscriptionV1) -> anyhow::Result<()> {
        let end_state_root = block.end_state_root;
        let checkpoint_id = block.bitcoin_block_number;
        let pos = KVQTreeNodePosition::new(TREE_HEIGHT, block.l2id);

        KVQStandardAdapter::<S, L2OLatestBlockKey, L2OABlockInscriptionV1>::set(
            &mut self.store,
            L2OLatestBlockKey::new(block.l2id),
            block,
        )?;

        set_state!(self.store, checkpoint_id, pos, end_state_root);

        Ok(())
    }

    fn has_deployed_l2id(&mut self, l2id: u64) -> anyhow::Result<bool> {
        let r = KVQStandardAdapter::<S, L2ODeploymentsKey, L2OADeployInscriptionV1>::get_exact(
            &mut self.store,
            &L2ODeploymentsKey::new(l2id),
        );
        match r {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn get_brc21_balance(&mut self, tick: String, address: String) -> anyhow::Result<u64> {
        KVQStandardAdapter::<S, L2OBRC21BalancesKey, u64>::get_exact(
            &self.store,
            &L2OBRC21BalancesKey::new(tick, address),
        )
    }
    fn transfer_brc21(
        &mut self,
        tick: String,
        from: String,
        to: String,
        amount: u64,
    ) -> anyhow::Result<()> {
        let from_old_balance = Self::get_brc21_balance(self, tick.clone(), from.clone())?;
        let to_old_balance = Self::get_brc21_balance(self, tick.clone(), to.clone())?;
        let mut updates = vec![KVQPair {
            key: L2OBRC21BalancesKey::new(tick.clone(), to),
            value: to_old_balance
                .checked_add(amount)
                .ok_or(anyhow::anyhow!("Arithmetic: Overflow"))?,
        }];
        if from.as_str() != BRC20_BURN_ADDRESS {
            updates.push(KVQPair {
                key: L2OBRC21BalancesKey::new(tick.clone(), from),
                value: from_old_balance
                    .checked_sub(amount)
                    .ok_or(anyhow::anyhow!("Arithmetic: Underflow"))?,
            })
        }
        KVQStandardAdapter::<S, L2OBRC21BalancesKey, u64>::set_many(&mut self.store, &updates)
    }
}
