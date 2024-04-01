use kvq::adapters::standard::KVQStandardAdapter;
use kvq::memory::simple::KVQSimpleMemoryBackingStore;
use l2o_common::common::data::hash::Hash256;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::merkle::store::key::KVQMerkleNodeKey;
use l2o_crypto::hash::merkle::store::model::KVQMerkleTreeModel;

type DemoModel<const TABLE_TYPE: u16> = KVQMerkleTreeModel<
    TABLE_TYPE,
    false,
    KVQSimpleMemoryBackingStore,
    KVQStandardAdapter<KVQSimpleMemoryBackingStore, KVQMerkleNodeKey<TABLE_TYPE>, Hash256>,
    Hash256,
    Sha256Hasher,
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
        Hash256::rand(),
    )
    .unwrap();
    println!("verify: {}", r.verify::<Sha256Hasher>());

    println!("delta: {}", serde_json::to_string(&r).unwrap());
}
