use std::marker::PhantomData;

use kvq::traits::KVQBinaryStore;
use kvq::traits::KVQBinaryStoreReader;
use kvq::traits::KVQPair;
use kvq::traits::KVQSerializable;
use kvq::traits::KVQStoreAdapter;
use kvq::traits::KVQStoreAdapterReader;
use serde::Deserialize;
use serde::Serialize;

use super::key::KVQMerkleNodeKey;
use crate::hash::merkle::core::calc_merkle_path;
use crate::hash::merkle::core::calc_merkle_root_marked_if;
use crate::hash::merkle::core::DeltaMerkleProofCore;
use crate::hash::merkle::core::MerkleProofCore;
use crate::hash::merkle::store::key::KVQAppendOnlyMerkleKey;
use crate::hash::merkle::traits::GeneralMerkleZeroHasher;
use crate::hash::traits::ZeroableHash;

const CHECKPOINT_SIZE: usize = 8;

pub struct KVQMerkleTreeModel<
    const TABLE_TYPE: u16,
    const TREE_HEIGHT: u8,
    const MARK_LEAVES: bool,
    S: KVQBinaryStoreReader,
    Hash: Copy + PartialEq + KVQSerializable + Serialize,
    Hasher: GeneralMerkleZeroHasher<Hash>,
    KVA,
> where
    for<'de2> Hash: Deserialize<'de2>,
{
    _hasher: PhantomData<Hasher>,
    _hash: PhantomData<Hash>,
    _s: PhantomData<S>,
    _kva: PhantomData<KVA>,
}
impl<
        const TABLE_TYPE: u16,
        const TREE_HEIGHT: u8,
        const MARK_LEAVES: bool,
        S: KVQBinaryStoreReader,
        Hash: PartialEq + KVQSerializable + Copy + Serialize,
        Hasher: GeneralMerkleZeroHasher<Hash>,
        KVA: KVQStoreAdapterReader<S, KVQMerkleNodeKey<TABLE_TYPE>, Hash>,
    > KVQMerkleTreeModel<TABLE_TYPE, TREE_HEIGHT, MARK_LEAVES, S, Hash, Hasher, KVA>
where
    for<'de2> Hash: Deserialize<'de2>,
{
    pub fn get_node(store: &S, key: &KVQMerkleNodeKey<TABLE_TYPE>) -> anyhow::Result<Hash> {
        match KVA::get_leq(store, key, CHECKPOINT_SIZE)? {
            Some(v) => Ok(v),
            None => {
                return Ok(Hasher::get_zero_hash_marked_if(
                    (TREE_HEIGHT - key.level).into(),
                    MARK_LEAVES,
                ))
            }
        }
    }
    pub fn get_nodes(
        store: &S,
        keys: &[KVQMerkleNodeKey<TABLE_TYPE>],
    ) -> anyhow::Result<Vec<Hash>> {
        let result = KVA::get_many_leq(store, keys, CHECKPOINT_SIZE)?;
        Ok(result
            .iter()
            .enumerate()
            .map(|(i, v)| match v {
                Some(v) => *v,
                None => Hasher::get_zero_hash((TREE_HEIGHT - (keys[i].level)).into()),
            })
            .collect())
    }
    pub fn get_leaf(
        store: &S,
        key: &KVQMerkleNodeKey<TABLE_TYPE>,
    ) -> anyhow::Result<MerkleProofCore<Hash>> {
        let nodes = Self::get_nodes(
            store,
            &vec![vec![*key], key.siblings(), vec![key.root()]].concat(),
        )?;
        let value = nodes[0];
        let root_ind = nodes.len() - 1;
        let siblings = nodes[1..root_ind].to_vec();
        let root = nodes[root_ind];
        Ok(MerkleProofCore::<Hash> {
            root,
            value,
            siblings,
            index: key.index,
        })
    }
}

impl<
        const TABLE_TYPE: u16,
        const TREE_HEIGHT: u8,
        const MARK_LEAVES: bool,
        S: KVQBinaryStore,
        Hash: PartialEq + KVQSerializable + Copy + Serialize,
        Hasher: GeneralMerkleZeroHasher<Hash>,
        KVA: KVQStoreAdapter<S, KVQMerkleNodeKey<TABLE_TYPE>, Hash>,
    > KVQMerkleTreeModel<TABLE_TYPE, TREE_HEIGHT, MARK_LEAVES, S, Hash, Hasher, KVA>
where
    for<'de2> Hash: Deserialize<'de2>,
{
    fn set_nodes<'a>(
        store: &mut S,
        nodes: &[KVQPair<KVQMerkleNodeKey<TABLE_TYPE>, Hash>],
    ) -> anyhow::Result<()> {
        KVA::set_many(store, nodes)
    }
    pub fn set_leaf(
        store: &mut S,
        key: &KVQMerkleNodeKey<TABLE_TYPE>,
        value: Hash,
    ) -> anyhow::Result<DeltaMerkleProofCore<Hash>> {
        let old_proof = Self::get_leaf(store, key)?;
        let mut current_value = value;
        let mut current_key = *key;

        let mut updates: Vec<KVQPair<KVQMerkleNodeKey<TABLE_TYPE>, Hash>> =
            Vec::with_capacity((key.level as usize) + 1);

        let height = key.level as usize;
        for i in 0..height {
            let index = current_key.index;
            let mark_leaves = i == 0 && MARK_LEAVES;
            updates.push(KVQPair::<KVQMerkleNodeKey<TABLE_TYPE>, Hash> {
                key: current_key,
                value: current_value,
            });
            current_value = if index & 1 == 0 {
                Hasher::two_to_one_marked_if(&current_value, &old_proof.siblings[i], mark_leaves)
            } else {
                Hasher::two_to_one_marked_if(&old_proof.siblings[i], &current_value, mark_leaves)
            };
            current_key = current_key.parent();
        }
        updates.push(KVQPair::<KVQMerkleNodeKey<TABLE_TYPE>, Hash> {
            key: current_key,
            value: current_value,
        });

        Self::set_nodes(store, &updates)?;
        Ok(DeltaMerkleProofCore::<Hash> {
            old_root: old_proof.root,
            old_value: old_proof.value,

            new_root: current_value,
            new_value: value,

            siblings: old_proof.siblings,
            index: key.index,
        })
    }
}

pub struct KVQAppendOnlyMerkleTreeModel<
    const TABLE_TYPE: u16,
    const TREE_HEIGHT: u8,
    S: KVQBinaryStoreReader,
    Hash: Copy + PartialEq + KVQSerializable + Serialize + ZeroableHash,
    Hasher: GeneralMerkleZeroHasher<Hash>,
    KVA,
> where
    for<'de2> Hash: Deserialize<'de2>,
{
    _hasher: PhantomData<Hasher>,
    _hash: PhantomData<Hash>,
    _s: PhantomData<S>,
    _kva: PhantomData<KVA>,
}
impl<
        const TABLE_TYPE: u16,
        const TREE_HEIGHT: u8,
        S: KVQBinaryStoreReader,
        Hash: PartialEq + KVQSerializable + Copy + Serialize + ZeroableHash,
        Hasher: GeneralMerkleZeroHasher<Hash>,
        KVA: KVQStoreAdapterReader<S, KVQAppendOnlyMerkleKey<TABLE_TYPE>, MerkleProofCore<Hash>>,
    > KVQAppendOnlyMerkleTreeModel<TABLE_TYPE, TREE_HEIGHT, S, Hash, Hasher, KVA>
where
    for<'de2> Hash: Deserialize<'de2>,
{
    pub fn get_last_proof(
        store: &S,
        key: &KVQAppendOnlyMerkleKey<TABLE_TYPE>,
    ) -> anyhow::Result<MerkleProofCore<Hash>> {
        KVA::get_exact(&store, &key)
    }
}

impl<
        const TABLE_TYPE: u16,
        const TREE_HEIGHT: u8,
        S: KVQBinaryStore,
        Hash: PartialEq + KVQSerializable + Copy + Serialize + ZeroableHash,
        Hasher: GeneralMerkleZeroHasher<Hash>,
        KVA: KVQStoreAdapter<S, KVQAppendOnlyMerkleKey<TABLE_TYPE>, MerkleProofCore<Hash>>,
    > KVQAppendOnlyMerkleTreeModel<TABLE_TYPE, TREE_HEIGHT, S, Hash, Hasher, KVA>
where
    for<'de2> Hash: Deserialize<'de2>,
{
    pub fn append_leaf(
        store: &mut S,
        key: &KVQAppendOnlyMerkleKey<TABLE_TYPE>,
        value: Hash,
    ) -> anyhow::Result<DeltaMerkleProofCore<Hash>> {
        let mut last_proof = Self::get_last_proof(store, key)?;

        let old_merkle_path = calc_merkle_path::<Hash, Hasher>(
            last_proof.value,
            &last_proof.siblings,
            last_proof.index,
        );
        let old_root = last_proof.root;
        let old_value = Hash::get_zero_value();
        let prev_index = last_proof.index;
        let new_index = prev_index + 1;
        let mut siblings = Vec::new();
        let mut multiplier = 1;

        for level in 0..(TREE_HEIGHT as usize) {
            let prev_level_index = prev_index / multiplier;
            let new_level_index = new_index / multiplier;

            if new_level_index == prev_level_index {
                siblings.push(last_proof.siblings[level]);
            } else {
                if new_level_index & 1 == 0 {
                    siblings.push(Hasher::get_zero_hash_marked_if(
                        TREE_HEIGHT as usize - level,
                        false,
                    ));
                } else {
                    siblings.push(old_merkle_path[level]);
                }
            }
            multiplier *= 2;
        }

        let new_root =
            calc_merkle_root_marked_if::<Hash, Hasher>(value, &siblings, new_index, false);
        last_proof = MerkleProofCore {
            index: new_index,
            siblings: siblings.clone(),
            root: new_root,
            value: value,
        };

        KVA::set_ref(store, key, &last_proof)?;

        Ok(DeltaMerkleProofCore {
            index: last_proof.index,
            siblings,
            old_root: old_root,
            old_value: old_value,
            new_root: new_root,
            new_value: value,
        })
    }
}
