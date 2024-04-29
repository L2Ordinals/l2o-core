use l2o_common::common::data::hash::Hash256;
use l2o_crypto::hash::hash_functions::blake3::Blake3Hasher;
use l2o_crypto::hash::hash_functions::keccak256::Keccak256Hasher;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::merkle::traits::MerkleHasher;
use l2o_crypto::hash::merkle::traits::MerkleHasherWithMarkedLeaf;
use l2o_crypto::hash::merkle::traits::ZERO_HASH_CACHE_SIZE;
use l2o_crypto::hash::traits::ZeroableHash;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::HashOut;
use plonky2::hash::hash_types::RichField;

fn compute_zero_hashes<Hash: ZeroableHash + PartialEq, Hasher: MerkleHasher<Hash>>(
    count: usize,
) -> Vec<Hash> {
    let mut result = vec![Hash::get_zero_value()];
    for i in 1..count {
        result.push(Hasher::two_to_one(&result[i - 1], &result[i - 1]));
    }
    result
}
fn compute_zero_hashes_leaf_hasher<
    Hash: ZeroableHash + PartialEq,
    Hasher: MerkleHasherWithMarkedLeaf<Hash>,
>(
    count: usize,
) -> Vec<Hash> {
    let mut result = vec![Hash::get_zero_value()];
    if count > 0 {
        result.push(Hasher::two_to_one_marked_leaf(&result[0], &result[0]));
    }
    for i in 2..count {
        result.push(Hasher::two_to_one(&result[i - 1], &result[i - 1]));
    }
    result
}
trait PrintableZeroHash: ZeroableHash + PartialEq {
    fn get_zh_string(&self) -> String;
    fn get_zh_type_string() -> String;
}
impl PrintableZeroHash for Hash256 {
    fn get_zh_string(&self) -> String {
        format!(
                "Hash256 ([{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}])",
                self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7], self.0[8], self.0[9], self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15], self.0[16], self.0[17], self.0[18], self.0[19], self.0[20], self.0[21], self.0[22], self.0[23], self.0[24], self.0[25], self.0[26], self.0[27], self.0[28], self.0[29], self.0[30], self.0[31]
            )
    }

    fn get_zh_type_string() -> String {
        format!("Hash256")
    }
}

trait NamedRicherField: RichField {
    fn get_zh_field_name() -> &'static str;
}
impl NamedRicherField for GoldilocksField {
    fn get_zh_field_name() -> &'static str {
        "GoldilocksField"
    }
}
impl PrintableZeroHash for HashOut<GoldilocksField> {
    fn get_zh_string(&self) -> String {
        format!(
            "HashOut {{ elements: [{}({}), {}({}), {}({}), {}({})] }}",
            GoldilocksField::get_zh_field_name(),
            self.elements[0],
            GoldilocksField::get_zh_field_name(),
            self.elements[1],
            GoldilocksField::get_zh_field_name(),
            self.elements[2],
            GoldilocksField::get_zh_field_name(),
            self.elements[3]
        )
    }

    fn get_zh_type_string() -> String {
        format!("HashOut<{}>", GoldilocksField::get_zh_field_name())
    }
}
trait NamedMerkleHasher {
    fn get_zh_hasher_name() -> &'static str;
}
impl NamedMerkleHasher for Sha256Hasher {
    fn get_zh_hasher_name() -> &'static str {
        "Sha256Hasher"
    }
}
impl NamedMerkleHasher for Blake3Hasher {
    fn get_zh_hasher_name() -> &'static str {
        "Blake3Hasher"
    }
}
impl NamedMerkleHasher for Keccak256Hasher {
    fn get_zh_hasher_name() -> &'static str {
        "Keccak256Hasher"
    }
}
impl NamedMerkleHasher for PoseidonHasher {
    fn get_zh_hasher_name() -> &'static str {
        "PoseidonHasher"
    }
}

fn get_zero_hashes_for_hash_str<
    Hash: PrintableZeroHash,
    Hasher: MerkleHasher<Hash> + NamedMerkleHasher,
>(
    count: usize,
) -> String {
    let mut zh_str = format!(
        "impl MerkleZeroHasherWithCache<{}> for {} {{\n    const CACHED_ZERO_HASHES: [{}; {}]= [\n",
        Hash::get_zh_type_string(),
        Hasher::get_zh_hasher_name(),
        Hash::get_zh_type_string(),
        count,
    );
    let zero_hashes = compute_zero_hashes::<Hash, Hasher>(count);
    for zh in zero_hashes.iter() {
        zh_str.push_str(&format!("        {},\n", zh.get_zh_string()));
    }
    zh_str.push_str("    ];\n}\n\n");
    zh_str
}

fn get_zero_hashes_for_leaf_hash_str<
    Hash: PrintableZeroHash,
    Hasher: MerkleHasherWithMarkedLeaf<Hash> + NamedMerkleHasher,
>(
    count: usize,
) -> String {
    let mut zh_str = format!(
            "impl MerkleZeroHasherWithCacheMarkedLeaf<{}> for {} {{\n    const CACHED_MARKED_LEAF_ZERO_HASHES: [{}; {}]= [\n",
            Hash::get_zh_type_string(),
            Hasher::get_zh_hasher_name(),
            Hash::get_zh_type_string(),
            count,
        );
    let zero_hashes = compute_zero_hashes_leaf_hasher::<Hash, Hasher>(count);
    for zh in zero_hashes.iter() {
        zh_str.push_str(&format!("        {},\n", zh.get_zh_string()));
    }
    zh_str.push_str("    ];\n}\n\n");
    zh_str
}

pub fn print_goldilocks_zero_hashes() {
    let mut result = "".to_string();
    result.push_str(&get_zero_hashes_for_hash_str::<
        HashOut<GoldilocksField>,
        PoseidonHasher,
    >(ZERO_HASH_CACHE_SIZE));
    result.push_str(&get_zero_hashes_for_leaf_hash_str::<
        HashOut<GoldilocksField>,
        PoseidonHasher,
    >(ZERO_HASH_CACHE_SIZE));
    tracing::info!("{}", result);
}
pub fn print_hash256_zero_hashes() {
    let mut result = "".to_string();
    result.push_str(&get_zero_hashes_for_hash_str::<Hash256, Sha256Hasher>(
        ZERO_HASH_CACHE_SIZE,
    ));
    result.push_str(&get_zero_hashes_for_leaf_hash_str::<Hash256, Sha256Hasher>(
        ZERO_HASH_CACHE_SIZE,
    ));
    result.push_str(&get_zero_hashes_for_hash_str::<Hash256, Blake3Hasher>(
        ZERO_HASH_CACHE_SIZE,
    ));
    result.push_str(&get_zero_hashes_for_leaf_hash_str::<Hash256, Blake3Hasher>(
        ZERO_HASH_CACHE_SIZE,
    ));
    result.push_str(&get_zero_hashes_for_hash_str::<Hash256, Keccak256Hasher>(
        ZERO_HASH_CACHE_SIZE,
    ));
    result.push_str(
        &get_zero_hashes_for_leaf_hash_str::<Hash256, Keccak256Hasher>(ZERO_HASH_CACHE_SIZE),
    );
    tracing::info!("{}", result);
}

fn main() {
    l2o_common::logger::setup_logger();
    print_hash256_zero_hashes();
    print_goldilocks_zero_hashes()
}
