use kvq::{adapters::standard::KVQStandardAdapter, memory::simple::KVQSimpleMemoryBackingStore};
use l2o_common::common::data::hash::Hash256;
use l2o_crypto::hash::{
    hash_functions::{
        poseidon_goldilocks::{PoseidonGoldilocksHasher, PoseidonHasher},
        sha256::Sha256Hasher,
    },
    merkle::store::{key::KVQMerkleNodeKey, model::KVQMerkleTreeModel},
};
use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Sample},
    hash::hash_types::HashOut,
};

type DemoModel<const TABLE_TYPE: u16> = KVQMerkleTreeModel<
    TABLE_TYPE,
    false,
    KVQSimpleMemoryBackingStore,
    KVQStandardAdapter<
        KVQSimpleMemoryBackingStore,
        KVQMerkleNodeKey<TABLE_TYPE>,
        HashOut<GoldilocksField>,
    >,
    HashOut<GoldilocksField>,
    PoseidonHasher,
>;

fn main() {
    const TABLE_TYPE: u16 = 1;

    const HEIGHT: usize = 30;
    let mut store = KVQSimpleMemoryBackingStore::new();
    let r = DemoModel::<TABLE_TYPE>::set_leaf(
        &mut store,
        &KVQMerkleNodeKey {
            tree_id: 0,
            primary_id: 0,
            secondary_id: 0,
            level: HEIGHT as u8,
            index: 3,
            checkpoint_id: 0,
        },
        HashOut::rand(),
    )
    .unwrap();
    println!("verify: {}", r.verify::<PoseidonHasher>());

    println!("delta: {}", serde_json::to_string(&r).unwrap());
}
