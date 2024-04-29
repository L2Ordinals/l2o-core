use kvq::adapters::standard::KVQStandardAdapter;
use kvq::memory::simple::KVQSimpleMemoryBackingStore;
use l2o_common::common::data::hash::Hash256;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::merkle::store::key::KVQMerkleNodeKey;
use l2o_crypto::hash::merkle::store::model::KVQMerkleTreeModel;

pub const TREE_HEIGHT: u8 = 30;
type DemoModel<const TABLE_TYPE: u16> = KVQMerkleTreeModel<
    TABLE_TYPE,
    TREE_HEIGHT,
    false,
    KVQSimpleMemoryBackingStore,
    Hash256,
    Sha256Hasher,
    KVQStandardAdapter<KVQSimpleMemoryBackingStore, KVQMerkleNodeKey<TABLE_TYPE>, Hash256>,
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
        Hash256::rand(),
    )
    .unwrap();
    tracing::info!("verify: {}", r.verify_marked_if::<Sha256Hasher>(false));

    tracing::info!("delta: {}", serde_json::to_string(&r).unwrap());
}
