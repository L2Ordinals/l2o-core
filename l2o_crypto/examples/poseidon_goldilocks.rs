use kvq::adapters::standard::KVQStandardAdapter;
use kvq::memory::simple::KVQSimpleMemoryBackingStore;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::merkle::store::key::KVQMerkleNodeKey;
use l2o_crypto::hash::merkle::store::model::KVQMerkleTreeModel;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Sample;
use plonky2::hash::hash_types::HashOut;

pub const TREE_HEIGHT: u8 = 30;
type DemoModel<const TABLE_TYPE: u16> = KVQMerkleTreeModel<
    TABLE_TYPE,
    TREE_HEIGHT,
    false,
    KVQSimpleMemoryBackingStore,
    HashOut<GoldilocksField>,
    PoseidonHasher,
    KVQStandardAdapter<
        KVQSimpleMemoryBackingStore,
        KVQMerkleNodeKey<TABLE_TYPE>,
        HashOut<GoldilocksField>,
    >,
>;

fn main() {
    l2o_common::logger::setup_logger();
    const TABLE_TYPE: u16 = 1;

    let mut store = KVQSimpleMemoryBackingStore::new();
    let r = DemoModel::<TABLE_TYPE>::set_leaf(
        &mut store,
        &KVQMerkleNodeKey {
            tree_id: 0,
            primary_id: 0,
            secondary_id: 0,
            level: TREE_HEIGHT,
            index: 3,
            checkpoint_id: 0,
        },
        HashOut::rand(),
    )
    .unwrap();
    tracing::info!("verify: {}", r.verify_marked_if::<PoseidonHasher>(false));

    tracing::info!("delta: {}", serde_json::to_string(&r).unwrap());
}
