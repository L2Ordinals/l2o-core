use kvq::adapters::standard::KVQStandardAdapter;
use kvq::memory::simple::KVQSimpleMemoryBackingStore;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::merkle::store::key::KVQMerkleNodeKey;
use l2o_crypto::hash::merkle::store::model::KVQMerkleTreeModel;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Sample;
use plonky2::hash::hash_types::HashOut;

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
    tracing::info!("verify: {}", r.verify::<PoseidonHasher>());

    tracing::info!("delta: {}", serde_json::to_string(&r).unwrap());
}
